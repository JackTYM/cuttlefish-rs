//! API key database operations.

use chrono::Utc;
use sqlx::SqlitePool;

use crate::models::ApiKey;

/// Create the api_keys table and indexes.
pub async fn create_api_keys_table(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS api_keys (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    name TEXT NOT NULL,
    key_hash TEXT NOT NULL UNIQUE,
    key_prefix TEXT NOT NULL,
    scopes TEXT NOT NULL DEFAULT '[]',
    created_at TEXT NOT NULL,
    last_used_at TEXT,
    expires_at TEXT,
    revoked_at TEXT,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
)"#,
    )
    .execute(pool)
    .await?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_api_keys_user ON api_keys(user_id)")
        .execute(pool)
        .await?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_api_keys_hash ON api_keys(key_hash)")
        .execute(pool)
        .await?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_api_keys_prefix ON api_keys(key_prefix)")
        .execute(pool)
        .await?;

    Ok(())
}

/// Create a new API key.
#[allow(clippy::too_many_arguments)]
pub async fn create_api_key(
    pool: &SqlitePool,
    id: &str,
    user_id: &str,
    name: &str,
    key_hash: &str,
    key_prefix: &str,
    scopes: &[String],
    expires_at: Option<&str>,
) -> Result<ApiKey, sqlx::Error> {
    let now = Utc::now().to_rfc3339();
    let scopes_json = serde_json::to_string(scopes).unwrap_or_else(|_| "[]".to_string());

    sqlx::query_as::<_, ApiKey>(
        r#"INSERT INTO api_keys (id, user_id, name, key_hash, key_prefix, scopes, created_at, expires_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        RETURNING *"#,
    )
    .bind(id)
    .bind(user_id)
    .bind(name)
    .bind(key_hash)
    .bind(key_prefix)
    .bind(&scopes_json)
    .bind(&now)
    .bind(expires_at)
    .fetch_one(pool)
    .await
}

/// Get an API key by its hash.
pub async fn get_api_key_by_hash(
    pool: &SqlitePool,
    key_hash: &str,
) -> Result<Option<ApiKey>, sqlx::Error> {
    sqlx::query_as::<_, ApiKey>("SELECT * FROM api_keys WHERE key_hash = ? AND revoked_at IS NULL")
        .bind(key_hash)
        .fetch_optional(pool)
        .await
}

/// Get an API key by ID.
pub async fn get_api_key_by_id(pool: &SqlitePool, id: &str) -> Result<Option<ApiKey>, sqlx::Error> {
    sqlx::query_as::<_, ApiKey>("SELECT * FROM api_keys WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await
}

/// List all API keys for a user.
pub async fn list_user_api_keys(
    pool: &SqlitePool,
    user_id: &str,
) -> Result<Vec<ApiKey>, sqlx::Error> {
    sqlx::query_as::<_, ApiKey>(
        "SELECT * FROM api_keys WHERE user_id = ? AND revoked_at IS NULL ORDER BY created_at DESC",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
}

/// Revoke an API key.
pub async fn revoke_api_key(pool: &SqlitePool, id: &str) -> Result<bool, sqlx::Error> {
    let now = Utc::now().to_rfc3339();
    let result =
        sqlx::query("UPDATE api_keys SET revoked_at = ? WHERE id = ? AND revoked_at IS NULL")
            .bind(&now)
            .bind(id)
            .execute(pool)
            .await?;

    Ok(result.rows_affected() > 0)
}

/// Revoke all API keys for a user.
pub async fn revoke_all_user_api_keys(
    pool: &SqlitePool,
    user_id: &str,
) -> Result<u64, sqlx::Error> {
    let now = Utc::now().to_rfc3339();
    let result =
        sqlx::query("UPDATE api_keys SET revoked_at = ? WHERE user_id = ? AND revoked_at IS NULL")
            .bind(&now)
            .bind(user_id)
            .execute(pool)
            .await?;

    Ok(result.rows_affected())
}

/// Update the last_used_at timestamp for an API key.
pub async fn update_api_key_last_used(pool: &SqlitePool, id: &str) -> Result<(), sqlx::Error> {
    let now = Utc::now().to_rfc3339();
    sqlx::query("UPDATE api_keys SET last_used_at = ? WHERE id = ?")
        .bind(&now)
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

/// Delete expired API keys.
pub async fn cleanup_expired_api_keys(pool: &SqlitePool) -> Result<u64, sqlx::Error> {
    let now = Utc::now().to_rfc3339();
    let result =
        sqlx::query("DELETE FROM api_keys WHERE expires_at IS NOT NULL AND expires_at < ?")
            .bind(&now)
            .execute(pool)
            .await?;

    Ok(result.rows_affected())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::create_users_table;
    use tempfile::TempDir;

    async fn test_pool() -> (SqlitePool, TempDir) {
        let dir = TempDir::new().expect("temp dir");
        let db_path = dir.path().join("test.db");
        let url = format!("sqlite://{}?mode=rwc", db_path.to_string_lossy());
        let pool = SqlitePool::connect(&url).await.expect("connect");
        create_users_table(&pool).await.expect("create users table");
        create_api_keys_table(&pool)
            .await
            .expect("create api_keys table");

        sqlx::query(
            "INSERT INTO users (id, email, password_hash, created_at, updated_at) VALUES ('user-1', 'test@example.com', 'hash', datetime('now'), datetime('now'))",
        )
        .execute(&pool)
        .await
        .expect("create test user");

        (pool, dir)
    }

    #[tokio::test]
    async fn test_create_and_get_api_key() {
        let (pool, _dir) = test_pool().await;

        let key = create_api_key(
            &pool,
            "key-1",
            "user-1",
            "My API Key",
            "hash-abc123",
            "cfish_abc",
            &["read".to_string(), "write".to_string()],
            None,
        )
        .await
        .expect("create");

        assert_eq!(key.id, "key-1");
        assert_eq!(key.name, "My API Key");
        assert_eq!(key.key_prefix, "cfish_abc");
        assert!(!key.is_revoked());

        let fetched = get_api_key_by_id(&pool, "key-1")
            .await
            .expect("get")
            .expect("exists");
        assert_eq!(fetched.user_id, "user-1");
    }

    #[tokio::test]
    async fn test_get_api_key_by_hash() {
        let (pool, _dir) = test_pool().await;

        create_api_key(
            &pool,
            "key-2",
            "user-1",
            "Test Key",
            "unique-hash-xyz",
            "cfish_xyz",
            &["read".to_string()],
            None,
        )
        .await
        .expect("create");

        let key = get_api_key_by_hash(&pool, "unique-hash-xyz")
            .await
            .expect("get")
            .expect("exists");
        assert_eq!(key.id, "key-2");

        let not_found = get_api_key_by_hash(&pool, "nonexistent")
            .await
            .expect("get");
        assert!(not_found.is_none());
    }

    #[tokio::test]
    async fn test_list_user_api_keys() {
        let (pool, _dir) = test_pool().await;

        for i in 0..3 {
            create_api_key(
                &pool,
                &format!("key-{i}"),
                "user-1",
                &format!("Key {i}"),
                &format!("hash-{i}"),
                &format!("cfish_{i}"),
                &["read".to_string()],
                None,
            )
            .await
            .expect("create");
        }

        let keys = list_user_api_keys(&pool, "user-1").await.expect("list");
        assert_eq!(keys.len(), 3);
    }

    #[tokio::test]
    async fn test_revoke_api_key() {
        let (pool, _dir) = test_pool().await;

        create_api_key(
            &pool,
            "key-revoke",
            "user-1",
            "Revoke Test",
            "hash-revoke",
            "cfish_rev",
            &["read".to_string()],
            None,
        )
        .await
        .expect("create");

        let revoked = revoke_api_key(&pool, "key-revoke").await.expect("revoke");
        assert!(revoked);

        let key = get_api_key_by_id(&pool, "key-revoke")
            .await
            .expect("get")
            .expect("exists");
        assert!(key.is_revoked());

        let by_hash = get_api_key_by_hash(&pool, "hash-revoke")
            .await
            .expect("get");
        assert!(by_hash.is_none());
    }

    #[tokio::test]
    async fn test_revoke_all_user_api_keys() {
        let (pool, _dir) = test_pool().await;

        for i in 0..3 {
            create_api_key(
                &pool,
                &format!("revoke-all-{i}"),
                "user-1",
                &format!("Key {i}"),
                &format!("hash-all-{i}"),
                &format!("cfish_a{i}"),
                &["read".to_string()],
                None,
            )
            .await
            .expect("create");
        }

        let count = revoke_all_user_api_keys(&pool, "user-1")
            .await
            .expect("revoke all");
        assert_eq!(count, 3);

        let keys = list_user_api_keys(&pool, "user-1").await.expect("list");
        assert!(keys.is_empty());
    }

    #[tokio::test]
    async fn test_update_last_used() {
        let (pool, _dir) = test_pool().await;

        create_api_key(
            &pool,
            "key-used",
            "user-1",
            "Used Key",
            "hash-used",
            "cfish_use",
            &["read".to_string()],
            None,
        )
        .await
        .expect("create");

        let key = get_api_key_by_id(&pool, "key-used")
            .await
            .expect("get")
            .expect("exists");
        assert!(key.last_used_at.is_none());

        update_api_key_last_used(&pool, "key-used")
            .await
            .expect("update");

        let key = get_api_key_by_id(&pool, "key-used")
            .await
            .expect("get")
            .expect("exists");
        assert!(key.last_used_at.is_some());
    }

    #[tokio::test]
    async fn test_cleanup_expired_api_keys() {
        let (pool, _dir) = test_pool().await;

        create_api_key(
            &pool,
            "key-expired",
            "user-1",
            "Expired Key",
            "hash-expired",
            "cfish_exp",
            &["read".to_string()],
            Some("2020-01-01T00:00:00Z"),
        )
        .await
        .expect("create expired");

        create_api_key(
            &pool,
            "key-valid",
            "user-1",
            "Valid Key",
            "hash-valid",
            "cfish_val",
            &["read".to_string()],
            Some("2099-01-01T00:00:00Z"),
        )
        .await
        .expect("create valid");

        let cleaned = cleanup_expired_api_keys(&pool).await.expect("cleanup");
        assert_eq!(cleaned, 1);

        let expired = get_api_key_by_id(&pool, "key-expired").await.expect("get");
        assert!(expired.is_none());

        let valid = get_api_key_by_id(&pool, "key-valid").await.expect("get");
        assert!(valid.is_some());
    }

    #[tokio::test]
    async fn test_api_key_scopes() {
        let (pool, _dir) = test_pool().await;

        let key = create_api_key(
            &pool,
            "key-scopes",
            "user-1",
            "Scoped Key",
            "hash-scopes",
            "cfish_scp",
            &["read".to_string(), "write".to_string(), "admin".to_string()],
            None,
        )
        .await
        .expect("create");

        let scopes = key.scopes();
        assert_eq!(scopes.len(), 3);
        assert!(scopes.contains(&"read".to_string()));
        assert!(scopes.contains(&"write".to_string()));
        assert!(scopes.contains(&"admin".to_string()));
    }
}
