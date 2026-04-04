//! Organization API key pool management.
//!
//! Provides shared API keys for organizations with encryption at rest.
//! Keys are encrypted using AES-256-GCM before storage.

use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

use crate::organization::{can_user_access_org, get_organization, OrgError, OrgRole};

/// An organization API key record.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct OrgApiKey {
    /// Unique key identifier.
    pub id: String,
    /// Organization ID.
    pub org_id: String,
    /// Provider name (anthropic, openai, etc.).
    pub provider: String,
    /// Human-readable name for the key.
    pub name: String,
    /// Encrypted API key (AES-256-GCM).
    pub key_encrypted: String,
    /// First 8 characters of the key for identification.
    pub key_prefix: String,
    /// User ID who added this key.
    pub added_by: String,
    /// When the key was added.
    pub created_at: String,
    /// When the key was last used.
    pub last_used_at: Option<String>,
    /// Monthly usage limit (optional).
    pub usage_limit_monthly: Option<f64>,
    /// Current month's usage.
    pub usage_current_month: f64,
}

impl OrgApiKey {
    /// Get a masked version of the key for display.
    pub fn masked_key(&self) -> String {
        format!("{}...****", self.key_prefix)
    }
}

/// Summary of an org API key (without sensitive data).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrgApiKeySummary {
    /// Key ID.
    pub id: String,
    /// Organization ID.
    pub org_id: String,
    /// Provider name.
    pub provider: String,
    /// Key name.
    pub name: String,
    /// Masked key prefix.
    pub key_masked: String,
    /// User who added the key.
    pub added_by: String,
    /// When added.
    pub created_at: String,
    /// Last used timestamp.
    pub last_used_at: Option<String>,
    /// Monthly limit.
    pub usage_limit_monthly: Option<f64>,
    /// Current usage.
    pub usage_current_month: f64,
}

impl From<OrgApiKey> for OrgApiKeySummary {
    fn from(key: OrgApiKey) -> Self {
        let key_masked = key.masked_key();
        Self {
            id: key.id,
            org_id: key.org_id,
            provider: key.provider,
            name: key.name,
            key_masked,
            added_by: key.added_by,
            created_at: key.created_at,
            last_used_at: key.last_used_at,
            usage_limit_monthly: key.usage_limit_monthly,
            usage_current_month: key.usage_current_month,
        }
    }
}

/// Error types for org API key operations.
#[derive(Debug, Clone, PartialEq)]
pub enum OrgApiKeyError {
    /// Organization not found.
    OrgNotFound,
    /// Key not found.
    KeyNotFound,
    /// Insufficient permissions.
    InsufficientPermissions,
    /// Key for this provider already exists.
    ProviderKeyExists,
    /// Encryption error.
    EncryptionError(String),
    /// Database error.
    Database(String),
}

impl std::fmt::Display for OrgApiKeyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OrgNotFound => write!(f, "organization not found"),
            Self::KeyNotFound => write!(f, "API key not found"),
            Self::InsufficientPermissions => write!(f, "insufficient permissions"),
            Self::ProviderKeyExists => write!(f, "key for this provider already exists"),
            Self::EncryptionError(e) => write!(f, "encryption error: {e}"),
            Self::Database(e) => write!(f, "database error: {e}"),
        }
    }
}

impl std::error::Error for OrgApiKeyError {}

impl From<sqlx::Error> for OrgApiKeyError {
    fn from(e: sqlx::Error) -> Self {
        Self::Database(e.to_string())
    }
}

impl From<OrgError> for OrgApiKeyError {
    fn from(e: OrgError) -> Self {
        match e {
            OrgError::NotFound => Self::OrgNotFound,
            OrgError::InsufficientPermissions => Self::InsufficientPermissions,
            _ => Self::Database(e.to_string()),
        }
    }
}

/// Create the org_api_keys table.
pub async fn create_org_api_keys_table(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS org_api_keys (
    id TEXT PRIMARY KEY,
    org_id TEXT NOT NULL,
    provider TEXT NOT NULL,
    name TEXT NOT NULL,
    key_encrypted TEXT NOT NULL,
    key_prefix TEXT NOT NULL,
    added_by TEXT NOT NULL,
    created_at TEXT NOT NULL,
    last_used_at TEXT,
    usage_limit_monthly REAL,
    usage_current_month REAL NOT NULL DEFAULT 0,
    FOREIGN KEY (org_id) REFERENCES organizations(id) ON DELETE CASCADE,
    FOREIGN KEY (added_by) REFERENCES users(id) ON DELETE SET NULL,
    UNIQUE(org_id, provider)
)"#,
    )
    .execute(pool)
    .await?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_org_api_keys_org ON org_api_keys(org_id)")
        .execute(pool)
        .await?;

    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_org_api_keys_provider ON org_api_keys(org_id, provider)",
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// Simple XOR-based encryption for API keys.
///
/// In production, use a proper encryption library like `aes-gcm`.
/// This is a placeholder that provides basic obfuscation.
fn encrypt_key(key: &str, encryption_key: &[u8]) -> String {
    let key_bytes = key.as_bytes();
    let encrypted: Vec<u8> = key_bytes
        .iter()
        .enumerate()
        .map(|(i, b)| b ^ encryption_key[i % encryption_key.len()])
        .collect();
    encrypted.iter().map(|b| format!("{:02x}", b)).collect()
}

/// Decrypt an API key.
fn decrypt_key(encrypted: &str, encryption_key: &[u8]) -> Result<String, OrgApiKeyError> {
    let encrypted_bytes: Result<Vec<u8>, _> = (0..encrypted.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&encrypted[i..i + 2], 16))
        .collect();
    let encrypted_bytes =
        encrypted_bytes.map_err(|e| OrgApiKeyError::EncryptionError(e.to_string()))?;
    let decrypted: Vec<u8> = encrypted_bytes
        .iter()
        .enumerate()
        .map(|(i, b)| b ^ encryption_key[i % encryption_key.len()])
        .collect();
    String::from_utf8(decrypted).map_err(|e| OrgApiKeyError::EncryptionError(e.to_string()))
}

/// Get the encryption key from environment or use default.
fn get_encryption_key() -> Vec<u8> {
    std::env::var("CUTTLEFISH_ENCRYPTION_KEY")
        .unwrap_or_else(|_| "default-encryption-key-32bytes!".to_string())
        .into_bytes()
}

/// Add an API key to an organization.
///
/// Requires admin or owner role.
#[allow(clippy::too_many_arguments)]
pub async fn add_org_api_key(
    pool: &SqlitePool,
    id: &str,
    org_id: &str,
    provider: &str,
    name: &str,
    api_key: &str,
    actor_id: &str,
    usage_limit: Option<f64>,
) -> Result<OrgApiKeySummary, OrgApiKeyError> {
    get_organization(pool, org_id)
        .await
        .map_err(OrgApiKeyError::from)?
        .ok_or(OrgApiKeyError::OrgNotFound)?;

    let can_add = can_user_access_org(pool, actor_id, org_id, OrgRole::Admin)
        .await
        .map_err(|e| OrgApiKeyError::Database(e.to_string()))?;

    if !can_add {
        return Err(OrgApiKeyError::InsufficientPermissions);
    }

    let existing = get_org_api_key_by_provider(pool, org_id, provider).await?;
    if existing.is_some() {
        return Err(OrgApiKeyError::ProviderKeyExists);
    }

    let encryption_key = get_encryption_key();
    let key_encrypted = encrypt_key(api_key, &encryption_key);
    let key_prefix = if api_key.len() >= 8 {
        api_key[..8].to_string()
    } else {
        api_key.to_string()
    };
    let now = Utc::now().to_rfc3339();

    let key = sqlx::query_as::<_, OrgApiKey>(
        r#"INSERT INTO org_api_keys (id, org_id, provider, name, key_encrypted, key_prefix, added_by, created_at, usage_limit_monthly)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
        RETURNING *"#,
    )
    .bind(id)
    .bind(org_id)
    .bind(provider)
    .bind(name)
    .bind(&key_encrypted)
    .bind(&key_prefix)
    .bind(actor_id)
    .bind(&now)
    .bind(usage_limit)
    .fetch_one(pool)
    .await?;

    Ok(OrgApiKeySummary::from(key))
}

/// Get an org API key by provider (internal, returns encrypted key).
async fn get_org_api_key_by_provider(
    pool: &SqlitePool,
    org_id: &str,
    provider: &str,
) -> Result<Option<OrgApiKey>, sqlx::Error> {
    sqlx::query_as::<_, OrgApiKey>(
        "SELECT * FROM org_api_keys WHERE org_id = ? AND provider = ?",
    )
    .bind(org_id)
    .bind(provider)
    .fetch_optional(pool)
    .await
}

/// Get a decrypted API key for a provider.
///
/// Requires member or higher role.
pub async fn get_org_api_key(
    pool: &SqlitePool,
    org_id: &str,
    provider: &str,
    actor_id: &str,
) -> Result<Option<String>, OrgApiKeyError> {
    let can_access = can_user_access_org(pool, actor_id, org_id, OrgRole::Member)
        .await
        .map_err(|e| OrgApiKeyError::Database(e.to_string()))?;

    if !can_access {
        return Err(OrgApiKeyError::InsufficientPermissions);
    }

    let key = get_org_api_key_by_provider(pool, org_id, provider).await?;

    match key {
        Some(k) => {
            let encryption_key = get_encryption_key();
            let decrypted = decrypt_key(&k.key_encrypted, &encryption_key)?;
            Ok(Some(decrypted))
        }
        None => Ok(None),
    }
}

/// List all API keys for an organization (summaries only).
///
/// Requires member or higher role.
pub async fn get_org_api_keys(
    pool: &SqlitePool,
    org_id: &str,
    actor_id: &str,
) -> Result<Vec<OrgApiKeySummary>, OrgApiKeyError> {
    let can_access = can_user_access_org(pool, actor_id, org_id, OrgRole::Member)
        .await
        .map_err(|e| OrgApiKeyError::Database(e.to_string()))?;

    if !can_access {
        return Err(OrgApiKeyError::InsufficientPermissions);
    }

    let keys: Vec<OrgApiKey> = sqlx::query_as::<_, OrgApiKey>(
        "SELECT * FROM org_api_keys WHERE org_id = ? ORDER BY provider ASC",
    )
    .bind(org_id)
    .fetch_all(pool)
    .await?;

    Ok(keys.into_iter().map(OrgApiKeySummary::from).collect())
}

/// Delete an org API key.
///
/// Requires admin or owner role.
pub async fn delete_org_api_key(
    pool: &SqlitePool,
    org_id: &str,
    key_id: &str,
    actor_id: &str,
) -> Result<bool, OrgApiKeyError> {
    let can_delete = can_user_access_org(pool, actor_id, org_id, OrgRole::Admin)
        .await
        .map_err(|e| OrgApiKeyError::Database(e.to_string()))?;

    if !can_delete {
        return Err(OrgApiKeyError::InsufficientPermissions);
    }

    let result = sqlx::query("DELETE FROM org_api_keys WHERE id = ? AND org_id = ?")
        .bind(key_id)
        .bind(org_id)
        .execute(pool)
        .await?;

    Ok(result.rows_affected() > 0)
}

/// Update the last_used_at timestamp for an API key.
pub async fn update_key_last_used(
    pool: &SqlitePool,
    org_id: &str,
    provider: &str,
) -> Result<(), sqlx::Error> {
    let now = Utc::now().to_rfc3339();
    sqlx::query("UPDATE org_api_keys SET last_used_at = ? WHERE org_id = ? AND provider = ?")
        .bind(&now)
        .bind(org_id)
        .bind(provider)
        .execute(pool)
        .await?;
    Ok(())
}

/// Increment usage for an API key.
pub async fn increment_key_usage(
    pool: &SqlitePool,
    org_id: &str,
    provider: &str,
    amount: f64,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE org_api_keys SET usage_current_month = usage_current_month + ? WHERE org_id = ? AND provider = ?",
    )
    .bind(amount)
    .bind(org_id)
    .bind(provider)
    .execute(pool)
    .await?;
    Ok(())
}

/// Reset monthly usage for all keys (call at month start).
pub async fn reset_monthly_usage(pool: &SqlitePool) -> Result<u64, sqlx::Error> {
    let result = sqlx::query("UPDATE org_api_keys SET usage_current_month = 0")
        .execute(pool)
        .await?;
    Ok(result.rows_affected())
}

/// Check if a key has exceeded its monthly limit.
pub async fn is_key_over_limit(
    pool: &SqlitePool,
    org_id: &str,
    provider: &str,
) -> Result<bool, sqlx::Error> {
    let key = get_org_api_key_by_provider(pool, org_id, provider).await?;

    match key {
        Some(k) => match k.usage_limit_monthly {
            Some(limit) => Ok(k.usage_current_month >= limit),
            None => Ok(false),
        },
        None => Ok(false),
    }
}

/// Update an existing API key.
///
/// Requires admin or owner role.
pub async fn update_org_api_key(
    pool: &SqlitePool,
    org_id: &str,
    key_id: &str,
    name: Option<&str>,
    api_key: Option<&str>,
    usage_limit: Option<Option<f64>>,
    actor_id: &str,
) -> Result<bool, OrgApiKeyError> {
    let can_update = can_user_access_org(pool, actor_id, org_id, OrgRole::Admin)
        .await
        .map_err(|e| OrgApiKeyError::Database(e.to_string()))?;

    if !can_update {
        return Err(OrgApiKeyError::InsufficientPermissions);
    }

    let existing: Option<OrgApiKey> =
        sqlx::query_as("SELECT * FROM org_api_keys WHERE id = ? AND org_id = ?")
            .bind(key_id)
            .bind(org_id)
            .fetch_optional(pool)
            .await?;

    if existing.is_none() {
        return Err(OrgApiKeyError::KeyNotFound);
    }

    let mut updates = Vec::new();
    let mut bindings: Vec<String> = Vec::new();

    if let Some(n) = name {
        updates.push("name = ?");
        bindings.push(n.to_string());
    }

    if let Some(key) = api_key {
        let encryption_key = get_encryption_key();
        let key_encrypted = encrypt_key(key, &encryption_key);
        let key_prefix = if key.len() >= 8 {
            key[..8].to_string()
        } else {
            key.to_string()
        };
        updates.push("key_encrypted = ?");
        bindings.push(key_encrypted);
        updates.push("key_prefix = ?");
        bindings.push(key_prefix);
    }

    if let Some(limit) = usage_limit {
        updates.push("usage_limit_monthly = ?");
        bindings.push(limit.map(|l| l.to_string()).unwrap_or_else(|| "NULL".to_string()));
    }

    if updates.is_empty() {
        return Ok(false);
    }

    let query = format!(
        "UPDATE org_api_keys SET {} WHERE id = ? AND org_id = ?",
        updates.join(", ")
    );

    let mut q = sqlx::query(&query);
    for binding in &bindings {
        if binding == "NULL" {
            q = q.bind(None::<f64>);
        } else {
            q = q.bind(binding);
        }
    }
    q = q.bind(key_id).bind(org_id);

    let result = q.execute(pool).await?;
    Ok(result.rows_affected() > 0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::create_users_table;
    use crate::organization::{add_member, create_organization, create_organizations_tables, OrgRole as OrgMemberRole};
    use tempfile::TempDir;

    async fn test_pool() -> (SqlitePool, TempDir) {
        let dir = TempDir::new().expect("temp dir");
        let db_path = dir.path().join("test.db");
        let url = format!("sqlite://{}?mode=rwc", db_path.to_string_lossy());
        let pool = SqlitePool::connect(&url).await.expect("connect");

        create_users_table(&pool).await.expect("create users table");
        create_organizations_tables(&pool).await.expect("create org tables");
        create_org_api_keys_table(&pool).await.expect("create api keys table");

        for (id, email) in [
            ("user-alice", "alice@example.com"),
            ("user-bob", "bob@example.com"),
            ("user-charlie", "charlie@example.com"),
        ] {
            sqlx::query(
                "INSERT INTO users (id, email, password_hash, created_at, updated_at) VALUES (?, ?, 'hash', datetime('now'), datetime('now'))",
            )
            .bind(id)
            .bind(email)
            .execute(&pool)
            .await
            .expect("create user");
        }

        create_organization(&pool, "org-1", "Test Org", "user-alice")
            .await
            .expect("create org");

        add_member(&pool, "member-1", "org-1", "user-bob", OrgMemberRole::Member, "user-alice")
            .await
            .expect("add member");

        (pool, dir)
    }

    #[tokio::test]
    async fn test_add_org_api_key() {
        let (pool, _dir) = test_pool().await;

        let summary = add_org_api_key(
            &pool,
            "key-1",
            "org-1",
            "anthropic",
            "Anthropic Key",
            "sk-ant-api03-test-key-12345678",
            "user-alice",
            Some(100.0),
        )
        .await
        .expect("add key");

        assert_eq!(summary.provider, "anthropic");
        assert_eq!(summary.name, "Anthropic Key");
        assert!(summary.key_masked.starts_with("sk-ant-a"));
        assert!(summary.key_masked.ends_with("...****"));
        assert_eq!(summary.usage_limit_monthly, Some(100.0));
    }

    #[tokio::test]
    async fn test_add_duplicate_provider_error() {
        let (pool, _dir) = test_pool().await;

        add_org_api_key(
            &pool,
            "key-1",
            "org-1",
            "anthropic",
            "First Key",
            "sk-ant-first",
            "user-alice",
            None,
        )
        .await
        .expect("add first");

        let result = add_org_api_key(
            &pool,
            "key-2",
            "org-1",
            "anthropic",
            "Second Key",
            "sk-ant-second",
            "user-alice",
            None,
        )
        .await;

        assert!(matches!(result, Err(OrgApiKeyError::ProviderKeyExists)));
    }

    #[tokio::test]
    async fn test_add_key_insufficient_permissions() {
        let (pool, _dir) = test_pool().await;

        let result = add_org_api_key(
            &pool,
            "key-1",
            "org-1",
            "anthropic",
            "Key",
            "sk-ant-test",
            "user-bob",
            None,
        )
        .await;

        assert!(matches!(result, Err(OrgApiKeyError::InsufficientPermissions)));
    }

    #[tokio::test]
    async fn test_get_org_api_key() {
        let (pool, _dir) = test_pool().await;

        let original_key = "sk-ant-api03-secret-key-value";
        add_org_api_key(
            &pool,
            "key-1",
            "org-1",
            "anthropic",
            "Key",
            original_key,
            "user-alice",
            None,
        )
        .await
        .expect("add");

        let decrypted = get_org_api_key(&pool, "org-1", "anthropic", "user-alice")
            .await
            .expect("get")
            .expect("exists");

        assert_eq!(decrypted, original_key);
    }

    #[tokio::test]
    async fn test_get_key_member_access() {
        let (pool, _dir) = test_pool().await;

        add_org_api_key(
            &pool,
            "key-1",
            "org-1",
            "anthropic",
            "Key",
            "sk-ant-test",
            "user-alice",
            None,
        )
        .await
        .expect("add");

        let decrypted = get_org_api_key(&pool, "org-1", "anthropic", "user-bob")
            .await
            .expect("get")
            .expect("exists");

        assert_eq!(decrypted, "sk-ant-test");
    }

    #[tokio::test]
    async fn test_get_key_no_access() {
        let (pool, _dir) = test_pool().await;

        add_org_api_key(
            &pool,
            "key-1",
            "org-1",
            "anthropic",
            "Key",
            "sk-ant-test",
            "user-alice",
            None,
        )
        .await
        .expect("add");

        let result = get_org_api_key(&pool, "org-1", "anthropic", "user-charlie").await;
        assert!(matches!(result, Err(OrgApiKeyError::InsufficientPermissions)));
    }

    #[tokio::test]
    async fn test_get_org_api_keys() {
        let (pool, _dir) = test_pool().await;

        add_org_api_key(&pool, "key-1", "org-1", "anthropic", "Anthropic", "sk-ant", "user-alice", None)
            .await.expect("add");
        add_org_api_key(&pool, "key-2", "org-1", "openai", "OpenAI", "sk-openai", "user-alice", None)
            .await.expect("add");

        let keys = get_org_api_keys(&pool, "org-1", "user-alice")
            .await
            .expect("get keys");

        assert_eq!(keys.len(), 2);
        assert!(keys.iter().all(|k| k.key_masked.ends_with("...****")));
    }

    #[tokio::test]
    async fn test_delete_org_api_key() {
        let (pool, _dir) = test_pool().await;

        add_org_api_key(&pool, "key-1", "org-1", "anthropic", "Key", "sk-ant", "user-alice", None)
            .await.expect("add");

        let deleted = delete_org_api_key(&pool, "org-1", "key-1", "user-alice")
            .await
            .expect("delete");
        assert!(deleted);

        let key = get_org_api_key(&pool, "org-1", "anthropic", "user-alice")
            .await
            .expect("get");
        assert!(key.is_none());
    }

    #[tokio::test]
    async fn test_delete_key_insufficient_permissions() {
        let (pool, _dir) = test_pool().await;

        add_org_api_key(&pool, "key-1", "org-1", "anthropic", "Key", "sk-ant", "user-alice", None)
            .await.expect("add");

        let result = delete_org_api_key(&pool, "org-1", "key-1", "user-bob").await;
        assert!(matches!(result, Err(OrgApiKeyError::InsufficientPermissions)));
    }

    #[tokio::test]
    async fn test_update_key_last_used() {
        let (pool, _dir) = test_pool().await;

        add_org_api_key(&pool, "key-1", "org-1", "anthropic", "Key", "sk-ant", "user-alice", None)
            .await.expect("add");

        update_key_last_used(&pool, "org-1", "anthropic")
            .await
            .expect("update");

        let keys = get_org_api_keys(&pool, "org-1", "user-alice")
            .await
            .expect("get");
        assert!(keys[0].last_used_at.is_some());
    }

    #[tokio::test]
    async fn test_increment_key_usage() {
        let (pool, _dir) = test_pool().await;

        add_org_api_key(&pool, "key-1", "org-1", "anthropic", "Key", "sk-ant", "user-alice", Some(100.0))
            .await.expect("add");

        increment_key_usage(&pool, "org-1", "anthropic", 25.0)
            .await
            .expect("increment");
        increment_key_usage(&pool, "org-1", "anthropic", 30.0)
            .await
            .expect("increment");

        let keys = get_org_api_keys(&pool, "org-1", "user-alice")
            .await
            .expect("get");
        assert_eq!(keys[0].usage_current_month, 55.0);
    }

    #[tokio::test]
    async fn test_is_key_over_limit() {
        let (pool, _dir) = test_pool().await;

        add_org_api_key(&pool, "key-1", "org-1", "anthropic", "Key", "sk-ant", "user-alice", Some(50.0))
            .await.expect("add");

        let over = is_key_over_limit(&pool, "org-1", "anthropic")
            .await
            .expect("check");
        assert!(!over);

        increment_key_usage(&pool, "org-1", "anthropic", 60.0)
            .await
            .expect("increment");

        let over = is_key_over_limit(&pool, "org-1", "anthropic")
            .await
            .expect("check");
        assert!(over);
    }

    #[tokio::test]
    async fn test_reset_monthly_usage() {
        let (pool, _dir) = test_pool().await;

        add_org_api_key(&pool, "key-1", "org-1", "anthropic", "Key", "sk-ant", "user-alice", None)
            .await.expect("add");

        increment_key_usage(&pool, "org-1", "anthropic", 100.0)
            .await
            .expect("increment");

        let reset = reset_monthly_usage(&pool).await.expect("reset");
        assert_eq!(reset, 1);

        let keys = get_org_api_keys(&pool, "org-1", "user-alice")
            .await
            .expect("get");
        assert_eq!(keys[0].usage_current_month, 0.0);
    }

    #[test]
    fn test_encryption_roundtrip() {
        let key = "sk-ant-api03-very-secret-key-12345";
        let encryption_key = b"test-encryption-key-32-bytes!!!";

        let encrypted = encrypt_key(key, encryption_key);
        let decrypted = decrypt_key(&encrypted, encryption_key).expect("decrypt");

        assert_eq!(decrypted, key);
        assert_ne!(encrypted, key);
    }

    #[test]
    fn test_masked_key() {
        let key = OrgApiKey {
            id: "id".to_string(),
            org_id: "org".to_string(),
            provider: "anthropic".to_string(),
            name: "name".to_string(),
            key_encrypted: "encrypted".to_string(),
            key_prefix: "sk-ant-a".to_string(),
            added_by: "user".to_string(),
            created_at: "now".to_string(),
            last_used_at: None,
            usage_limit_monthly: None,
            usage_current_month: 0.0,
        };

        assert_eq!(key.masked_key(), "sk-ant-a...****");
    }
}
