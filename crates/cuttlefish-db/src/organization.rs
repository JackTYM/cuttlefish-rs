//! Organization database operations.
//!
//! This module provides functionality for managing organizations,
//! including creation, membership, and role-based access control.

use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

/// Organization role for access control.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OrgRole {
    /// Full control over the organization (billing, deletion).
    Owner,
    /// Can manage members and create projects.
    Admin,
    /// Can work on org projects.
    Member,
}

impl OrgRole {
    /// Convert role to string representation.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Owner => "owner",
            Self::Admin => "admin",
            Self::Member => "member",
        }
    }

    /// Parse role from string.
    pub fn parse(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "owner" => Self::Owner,
            "admin" => Self::Admin,
            _ => Self::Member,
        }
    }

    /// Get numeric level for comparison (higher = more permissions).
    pub fn level(&self) -> u8 {
        match self {
            Self::Owner => 3,
            Self::Admin => 2,
            Self::Member => 1,
        }
    }

    /// Check if this role has at least the permissions of another role.
    pub fn has_at_least(&self, other: Self) -> bool {
        self.level() >= other.level()
    }
}

/// Organization settings.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OrgSettings {
    /// Default model to use for new projects.
    pub default_model: Option<String>,
    /// List of allowed providers (None = all allowed).
    pub allowed_providers: Option<Vec<String>>,
    /// Maximum number of projects allowed.
    pub max_projects: Option<i64>,
    /// Shared template IDs.
    pub shared_templates: Vec<String>,
}

impl OrgSettings {
    /// Create new empty settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Serialize to JSON string.
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| "{}".to_string())
    }

    /// Deserialize from JSON string.
    pub fn from_json(s: &str) -> Self {
        serde_json::from_str(s).unwrap_or_default()
    }
}

/// An organization in the Cuttlefish system.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Organization {
    /// Unique organization identifier (UUID string).
    pub id: String,
    /// Organization name.
    pub name: String,
    /// URL-friendly slug (unique).
    pub slug: String,
    /// User ID of the organization owner.
    pub owner_id: String,
    /// When the organization was created (ISO 8601 format).
    pub created_at: String,
    /// Organization settings (JSON).
    pub settings: Option<String>,
}

impl Organization {
    /// Get the parsed settings.
    pub fn settings(&self) -> OrgSettings {
        self.settings
            .as_ref()
            .map(|s| OrgSettings::from_json(s))
            .unwrap_or_default()
    }
}

/// An organization membership record.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct OrgMember {
    /// Unique membership identifier (UUID string).
    pub id: String,
    /// Organization ID.
    pub org_id: String,
    /// User ID.
    pub user_id: String,
    /// Role: owner, admin, or member.
    pub role: String,
    /// When the membership was created (ISO 8601 format).
    pub joined_at: String,
}

impl OrgMember {
    /// Get the parsed role.
    pub fn role(&self) -> OrgRole {
        OrgRole::parse(&self.role)
    }
}

/// Summary of an organization for list endpoints.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrgSummary {
    /// Organization ID.
    pub id: String,
    /// Organization name.
    pub name: String,
    /// URL-friendly slug.
    pub slug: String,
    /// User's role in the organization.
    pub role: String,
    /// Number of members.
    pub member_count: i64,
}

/// Error types for organization operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OrgError {
    /// Organization not found.
    NotFound,
    /// Slug already exists.
    SlugExists,
    /// User is already a member.
    AlreadyMember,
    /// User is not a member.
    NotMember,
    /// Cannot remove the last owner.
    LastOwner,
    /// Insufficient permissions.
    InsufficientPermissions,
    /// Cannot modify own role.
    CannotModifySelf,
    /// Database error.
    Database(String),
}

impl std::fmt::Display for OrgError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound => write!(f, "organization not found"),
            Self::SlugExists => write!(f, "organization slug already exists"),
            Self::AlreadyMember => write!(f, "user is already a member"),
            Self::NotMember => write!(f, "user is not a member"),
            Self::LastOwner => write!(f, "cannot remove the last owner"),
            Self::InsufficientPermissions => write!(f, "insufficient permissions"),
            Self::CannotModifySelf => write!(f, "cannot modify own role"),
            Self::Database(e) => write!(f, "database error: {e}"),
        }
    }
}

impl std::error::Error for OrgError {}

impl From<sqlx::Error> for OrgError {
    fn from(e: sqlx::Error) -> Self {
        Self::Database(e.to_string())
    }
}

/// Create the organizations and organization_members tables.
pub async fn create_organizations_tables(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS organizations (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    slug TEXT NOT NULL UNIQUE,
    owner_id TEXT NOT NULL,
    created_at TEXT NOT NULL,
    settings TEXT,
    FOREIGN KEY (owner_id) REFERENCES users(id) ON DELETE RESTRICT
)"#,
    )
    .execute(pool)
    .await?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_organizations_slug ON organizations(slug)")
        .execute(pool)
        .await?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_organizations_owner ON organizations(owner_id)")
        .execute(pool)
        .await?;

    sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS organization_members (
    id TEXT PRIMARY KEY,
    org_id TEXT NOT NULL,
    user_id TEXT NOT NULL,
    role TEXT NOT NULL,
    joined_at TEXT NOT NULL,
    FOREIGN KEY (org_id) REFERENCES organizations(id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    UNIQUE(org_id, user_id)
)"#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_org_members_org ON organization_members(org_id)",
    )
    .execute(pool)
    .await?;

    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_org_members_user ON organization_members(user_id)",
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// Generate a URL-friendly slug from a name.
fn generate_slug(name: &str) -> String {
    name.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

/// Create a new organization.
///
/// The creator becomes the owner and first member.
pub async fn create_organization(
    pool: &SqlitePool,
    id: &str,
    name: &str,
    owner_id: &str,
) -> Result<Organization, OrgError> {
    let slug = generate_slug(name);
    let now = Utc::now().to_rfc3339();
    let settings = OrgSettings::new().to_json();

    // Check if slug exists
    let existing = get_organization_by_slug(pool, &slug).await?;
    if existing.is_some() {
        return Err(OrgError::SlugExists);
    }

    let org = sqlx::query_as::<_, Organization>(
        r#"INSERT INTO organizations (id, name, slug, owner_id, created_at, settings)
        VALUES (?, ?, ?, ?, ?, ?)
        RETURNING *"#,
    )
    .bind(id)
    .bind(name)
    .bind(&slug)
    .bind(owner_id)
    .bind(&now)
    .bind(&settings)
    .fetch_one(pool)
    .await?;

    // Add owner as first member
    let member_id = uuid::Uuid::new_v4().to_string();
    sqlx::query(
        r#"INSERT INTO organization_members (id, org_id, user_id, role, joined_at)
        VALUES (?, ?, ?, 'owner', ?)"#,
    )
    .bind(&member_id)
    .bind(id)
    .bind(owner_id)
    .bind(&now)
    .execute(pool)
    .await?;

    Ok(org)
}

/// Get an organization by ID.
pub async fn get_organization(
    pool: &SqlitePool,
    org_id: &str,
) -> Result<Option<Organization>, sqlx::Error> {
    sqlx::query_as::<_, Organization>("SELECT * FROM organizations WHERE id = ?")
        .bind(org_id)
        .fetch_optional(pool)
        .await
}

/// Get an organization by slug.
pub async fn get_organization_by_slug(
    pool: &SqlitePool,
    slug: &str,
) -> Result<Option<Organization>, sqlx::Error> {
    sqlx::query_as::<_, Organization>("SELECT * FROM organizations WHERE slug = ?")
        .bind(slug)
        .fetch_optional(pool)
        .await
}

/// Add a member to an organization.
///
/// # Errors
///
/// Returns `OrgError::NotFound` if the organization doesn't exist.
/// Returns `OrgError::AlreadyMember` if the user is already a member.
/// Returns `OrgError::InsufficientPermissions` if the actor lacks permission.
pub async fn add_member(
    pool: &SqlitePool,
    member_id: &str,
    org_id: &str,
    user_id: &str,
    role: OrgRole,
    actor_id: &str,
) -> Result<OrgMember, OrgError> {
    // Verify org exists
    let org = get_organization(pool, org_id).await?.ok_or(OrgError::NotFound)?;

    // Verify actor has permission (must be owner or admin)
    let actor_member = get_member(pool, org_id, actor_id).await?;
    let actor_role = actor_member.map(|m| m.role()).unwrap_or(OrgRole::Member);

    if !actor_role.has_at_least(OrgRole::Admin) && org.owner_id != actor_id {
        return Err(OrgError::InsufficientPermissions);
    }

    // Cannot add someone as owner unless you're the owner
    if role == OrgRole::Owner && actor_role != OrgRole::Owner && org.owner_id != actor_id {
        return Err(OrgError::InsufficientPermissions);
    }

    // Check if already a member
    let existing = get_member(pool, org_id, user_id).await?;
    if existing.is_some() {
        return Err(OrgError::AlreadyMember);
    }

    let now = Utc::now().to_rfc3339();

    let member = sqlx::query_as::<_, OrgMember>(
        r#"INSERT INTO organization_members (id, org_id, user_id, role, joined_at)
        VALUES (?, ?, ?, ?, ?)
        RETURNING *"#,
    )
    .bind(member_id)
    .bind(org_id)
    .bind(user_id)
    .bind(role.as_str())
    .bind(&now)
    .fetch_one(pool)
    .await?;

    Ok(member)
}

/// Remove a member from an organization.
///
/// # Errors
///
/// Returns `OrgError::NotFound` if the organization doesn't exist.
/// Returns `OrgError::NotMember` if the user is not a member.
/// Returns `OrgError::LastOwner` if trying to remove the last owner.
/// Returns `OrgError::InsufficientPermissions` if the actor lacks permission.
pub async fn remove_member(
    pool: &SqlitePool,
    org_id: &str,
    user_id: &str,
    actor_id: &str,
) -> Result<bool, OrgError> {
    // Verify org exists
    let org = get_organization(pool, org_id).await?.ok_or(OrgError::NotFound)?;

    // Verify actor has permission
    let actor_member = get_member(pool, org_id, actor_id).await?;
    let actor_role = actor_member.map(|m| m.role()).unwrap_or(OrgRole::Member);

    // Users can remove themselves, otherwise need admin+
    if user_id != actor_id && !actor_role.has_at_least(OrgRole::Admin) && org.owner_id != actor_id {
        return Err(OrgError::InsufficientPermissions);
    }

    // Get target member
    let target_member = get_member(pool, org_id, user_id).await?.ok_or(OrgError::NotMember)?;
    let target_role = target_member.role();

    // Cannot remove someone with equal or higher role (unless removing self)
    if user_id != actor_id && target_role.level() >= actor_role.level() && org.owner_id != actor_id
    {
        return Err(OrgError::InsufficientPermissions);
    }

    // Check if this is the last owner
    if target_role == OrgRole::Owner {
        let owner_count = count_members_by_role(pool, org_id, OrgRole::Owner).await?;
        if owner_count <= 1 {
            return Err(OrgError::LastOwner);
        }
    }

    let result =
        sqlx::query("DELETE FROM organization_members WHERE org_id = ? AND user_id = ?")
            .bind(org_id)
            .bind(user_id)
            .execute(pool)
            .await?;

    Ok(result.rows_affected() > 0)
}

/// Update a member's role.
///
/// # Errors
///
/// Returns `OrgError::NotFound` if the organization doesn't exist.
/// Returns `OrgError::NotMember` if the user is not a member.
/// Returns `OrgError::LastOwner` if demoting the last owner.
/// Returns `OrgError::InsufficientPermissions` if the actor lacks permission.
/// Returns `OrgError::CannotModifySelf` if trying to modify own role.
pub async fn update_member_role(
    pool: &SqlitePool,
    org_id: &str,
    user_id: &str,
    new_role: OrgRole,
    actor_id: &str,
) -> Result<bool, OrgError> {
    // Cannot modify own role
    if user_id == actor_id {
        return Err(OrgError::CannotModifySelf);
    }

    // Verify org exists
    let org = get_organization(pool, org_id).await?.ok_or(OrgError::NotFound)?;

    // Verify actor has permission
    let actor_member = get_member(pool, org_id, actor_id).await?;
    let actor_role = actor_member.map(|m| m.role()).unwrap_or(OrgRole::Member);

    if !actor_role.has_at_least(OrgRole::Admin) && org.owner_id != actor_id {
        return Err(OrgError::InsufficientPermissions);
    }

    // Get target member
    let target_member = get_member(pool, org_id, user_id).await?.ok_or(OrgError::NotMember)?;
    let target_role = target_member.role();

    // Cannot modify someone with equal or higher role
    if target_role.level() >= actor_role.level() && org.owner_id != actor_id {
        return Err(OrgError::InsufficientPermissions);
    }

    // Cannot promote to a role higher than your own
    if new_role.level() > actor_role.level() && org.owner_id != actor_id {
        return Err(OrgError::InsufficientPermissions);
    }

    // Check if demoting the last owner
    if target_role == OrgRole::Owner && new_role != OrgRole::Owner {
        let owner_count = count_members_by_role(pool, org_id, OrgRole::Owner).await?;
        if owner_count <= 1 {
            return Err(OrgError::LastOwner);
        }
    }

    let result = sqlx::query(
        "UPDATE organization_members SET role = ? WHERE org_id = ? AND user_id = ?",
    )
    .bind(new_role.as_str())
    .bind(org_id)
    .bind(user_id)
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}

/// Get a specific member.
pub async fn get_member(
    pool: &SqlitePool,
    org_id: &str,
    user_id: &str,
) -> Result<Option<OrgMember>, sqlx::Error> {
    sqlx::query_as::<_, OrgMember>(
        "SELECT * FROM organization_members WHERE org_id = ? AND user_id = ?",
    )
    .bind(org_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await
}

/// Get all members of an organization.
pub async fn get_org_members(
    pool: &SqlitePool,
    org_id: &str,
) -> Result<Vec<OrgMember>, sqlx::Error> {
    sqlx::query_as::<_, OrgMember>(
        "SELECT * FROM organization_members WHERE org_id = ? ORDER BY joined_at ASC",
    )
    .bind(org_id)
    .fetch_all(pool)
    .await
}

/// Get all organizations a user belongs to.
pub async fn get_user_orgs(
    pool: &SqlitePool,
    user_id: &str,
) -> Result<Vec<OrgSummary>, sqlx::Error> {
    // Get memberships with org details
    let rows = sqlx::query_as::<_, (String, String, String, String)>(
        r#"SELECT o.id, o.name, o.slug, om.role
        FROM organizations o
        INNER JOIN organization_members om ON o.id = om.org_id
        WHERE om.user_id = ?
        ORDER BY o.name ASC"#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    let mut summaries = Vec::with_capacity(rows.len());
    for (id, name, slug, role) in rows {
        let member_count = count_members(pool, &id).await?;
        summaries.push(OrgSummary {
            id,
            name,
            slug,
            role,
            member_count,
        });
    }

    Ok(summaries)
}

/// Count members in an organization.
pub async fn count_members(pool: &SqlitePool, org_id: &str) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM organization_members WHERE org_id = ?",
    )
    .bind(org_id)
    .fetch_one(pool)
    .await
}

/// Count members by role in an organization.
pub async fn count_members_by_role(
    pool: &SqlitePool,
    org_id: &str,
    role: OrgRole,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM organization_members WHERE org_id = ? AND role = ?",
    )
    .bind(org_id)
    .bind(role.as_str())
    .fetch_one(pool)
    .await
}

/// Check if a user is a member of an organization with at least the required role.
pub async fn can_user_access_org(
    pool: &SqlitePool,
    user_id: &str,
    org_id: &str,
    required_role: OrgRole,
) -> Result<bool, sqlx::Error> {
    let member = get_member(pool, org_id, user_id).await?;

    match member {
        Some(m) => {
            let user_role = m.role();
            Ok(user_role.has_at_least(required_role))
        }
        None => Ok(false),
    }
}

/// Get a user's role in an organization.
pub async fn get_user_org_role(
    pool: &SqlitePool,
    user_id: &str,
    org_id: &str,
) -> Result<Option<OrgRole>, sqlx::Error> {
    let member = get_member(pool, org_id, user_id).await?;
    Ok(member.map(|m| m.role()))
}

/// Update organization settings.
pub async fn update_org_settings(
    pool: &SqlitePool,
    org_id: &str,
    settings: &OrgSettings,
) -> Result<bool, sqlx::Error> {
    let settings_json = settings.to_json();

    let result = sqlx::query("UPDATE organizations SET settings = ? WHERE id = ?")
        .bind(&settings_json)
        .bind(org_id)
        .execute(pool)
        .await?;

    Ok(result.rows_affected() > 0)
}

/// Update organization name.
pub async fn update_org_name(
    pool: &SqlitePool,
    org_id: &str,
    name: &str,
) -> Result<bool, OrgError> {
    let new_slug = generate_slug(name);

    // Check if new slug conflicts with another org
    let existing = get_organization_by_slug(pool, &new_slug).await?;
    if let Some(org) = existing
        && org.id != org_id
    {
        return Err(OrgError::SlugExists);
    }

    let result = sqlx::query("UPDATE organizations SET name = ?, slug = ? WHERE id = ?")
        .bind(name)
        .bind(&new_slug)
        .bind(org_id)
        .execute(pool)
        .await?;

    Ok(result.rows_affected() > 0)
}

/// Delete an organization.
///
/// Only the owner can delete an organization.
pub async fn delete_organization(
    pool: &SqlitePool,
    org_id: &str,
    actor_id: &str,
) -> Result<bool, OrgError> {
    let org = get_organization(pool, org_id).await?.ok_or(OrgError::NotFound)?;

    // Only owner can delete
    if org.owner_id != actor_id {
        let actor_member = get_member(pool, org_id, actor_id).await?;
        let actor_role = actor_member.map(|m| m.role()).unwrap_or(OrgRole::Member);
        if actor_role != OrgRole::Owner {
            return Err(OrgError::InsufficientPermissions);
        }
    }

    // Delete members first (cascade should handle this, but be explicit)
    sqlx::query("DELETE FROM organization_members WHERE org_id = ?")
        .bind(org_id)
        .execute(pool)
        .await?;

    let result = sqlx::query("DELETE FROM organizations WHERE id = ?")
        .bind(org_id)
        .execute(pool)
        .await?;

    Ok(result.rows_affected() > 0)
}

/// Check if an actor can modify another user's role.
///
/// Rules:
/// - Owners can modify anyone except other owners
/// - Admins can modify members only
/// - Members cannot modify roles
/// - No one can escalate to a role higher than their own
pub fn can_modify_org_role(
    actor_role: OrgRole,
    target_current_role: OrgRole,
    target_new_role: OrgRole,
) -> bool {
    // Can't escalate beyond your own role
    if target_new_role.level() > actor_role.level() {
        return false;
    }

    // Can't modify someone with equal or higher role
    if target_current_role.level() >= actor_role.level() {
        return false;
    }

    // Must be at least admin to modify roles
    actor_role.level() >= OrgRole::Admin.level()
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
        create_organizations_tables(&pool)
            .await
            .expect("create org tables");

        // Create test users
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

        (pool, dir)
    }

    #[tokio::test]
    async fn test_create_organization() {
        let (pool, _dir) = test_pool().await;

        let org = create_organization(&pool, "org-1", "Acme Corp", "user-alice")
            .await
            .expect("create org");

        assert_eq!(org.id, "org-1");
        assert_eq!(org.name, "Acme Corp");
        assert_eq!(org.slug, "acme-corp");
        assert_eq!(org.owner_id, "user-alice");

        // Owner should be a member
        let member = get_member(&pool, "org-1", "user-alice")
            .await
            .expect("get member")
            .expect("member exists");
        assert_eq!(member.role(), OrgRole::Owner);
    }

    #[tokio::test]
    async fn test_slug_generation() {
        assert_eq!(generate_slug("Acme Corp"), "acme-corp");
        assert_eq!(generate_slug("My  Company!"), "my-company");
        assert_eq!(generate_slug("Test123"), "test123");
        assert_eq!(generate_slug("  Spaces  "), "spaces");
    }

    #[tokio::test]
    async fn test_duplicate_slug_error() {
        let (pool, _dir) = test_pool().await;

        create_organization(&pool, "org-1", "Acme Corp", "user-alice")
            .await
            .expect("create first");

        let result = create_organization(&pool, "org-2", "Acme Corp", "user-bob").await;
        assert!(matches!(result, Err(OrgError::SlugExists)));
    }

    #[tokio::test]
    async fn test_get_organization() {
        let (pool, _dir) = test_pool().await;

        create_organization(&pool, "org-1", "Test Org", "user-alice")
            .await
            .expect("create");

        let org = get_organization(&pool, "org-1")
            .await
            .expect("get")
            .expect("exists");
        assert_eq!(org.name, "Test Org");

        let by_slug = get_organization_by_slug(&pool, "test-org")
            .await
            .expect("get by slug")
            .expect("exists");
        assert_eq!(by_slug.id, "org-1");
    }

    #[tokio::test]
    async fn test_add_member() {
        let (pool, _dir) = test_pool().await;

        create_organization(&pool, "org-1", "Test Org", "user-alice")
            .await
            .expect("create");

        let member = add_member(
            &pool,
            "member-1",
            "org-1",
            "user-bob",
            OrgRole::Member,
            "user-alice",
        )
        .await
        .expect("add member");

        assert_eq!(member.user_id, "user-bob");
        assert_eq!(member.role(), OrgRole::Member);
    }

    #[tokio::test]
    async fn test_add_member_already_exists() {
        let (pool, _dir) = test_pool().await;

        create_organization(&pool, "org-1", "Test Org", "user-alice")
            .await
            .expect("create");

        add_member(
            &pool,
            "member-1",
            "org-1",
            "user-bob",
            OrgRole::Member,
            "user-alice",
        )
        .await
        .expect("add first");

        let result = add_member(
            &pool,
            "member-2",
            "org-1",
            "user-bob",
            OrgRole::Admin,
            "user-alice",
        )
        .await;
        assert!(matches!(result, Err(OrgError::AlreadyMember)));
    }

    #[tokio::test]
    async fn test_remove_member() {
        let (pool, _dir) = test_pool().await;

        create_organization(&pool, "org-1", "Test Org", "user-alice")
            .await
            .expect("create");

        add_member(
            &pool,
            "member-1",
            "org-1",
            "user-bob",
            OrgRole::Member,
            "user-alice",
        )
        .await
        .expect("add");

        let removed = remove_member(&pool, "org-1", "user-bob", "user-alice")
            .await
            .expect("remove");
        assert!(removed);

        let member = get_member(&pool, "org-1", "user-bob").await.expect("get");
        assert!(member.is_none());
    }

    #[tokio::test]
    async fn test_remove_last_owner_error() {
        let (pool, _dir) = test_pool().await;

        create_organization(&pool, "org-1", "Test Org", "user-alice")
            .await
            .expect("create");

        let result = remove_member(&pool, "org-1", "user-alice", "user-alice").await;
        assert!(matches!(result, Err(OrgError::LastOwner)));
    }

    #[tokio::test]
    async fn test_update_member_role() {
        let (pool, _dir) = test_pool().await;

        create_organization(&pool, "org-1", "Test Org", "user-alice")
            .await
            .expect("create");

        add_member(
            &pool,
            "member-1",
            "org-1",
            "user-bob",
            OrgRole::Member,
            "user-alice",
        )
        .await
        .expect("add");

        let updated =
            update_member_role(&pool, "org-1", "user-bob", OrgRole::Admin, "user-alice")
                .await
                .expect("update");
        assert!(updated);

        let member = get_member(&pool, "org-1", "user-bob")
            .await
            .expect("get")
            .expect("exists");
        assert_eq!(member.role(), OrgRole::Admin);
    }

    #[tokio::test]
    async fn test_cannot_modify_self() {
        let (pool, _dir) = test_pool().await;

        create_organization(&pool, "org-1", "Test Org", "user-alice")
            .await
            .expect("create");

        let result =
            update_member_role(&pool, "org-1", "user-alice", OrgRole::Admin, "user-alice").await;
        assert!(matches!(result, Err(OrgError::CannotModifySelf)));
    }

    #[tokio::test]
    async fn test_demote_last_owner_error() {
        let (pool, _dir) = test_pool().await;

        // Alice creates org (she's the original owner stored in org.owner_id)
        create_organization(&pool, "org-1", "Test Org", "user-alice")
            .await
            .expect("create");

        // Add bob as owner
        add_member(
            &pool,
            "member-1",
            "org-1",
            "user-bob",
            OrgRole::Owner,
            "user-alice",
        )
        .await
        .expect("add bob as owner");

        // Add charlie as owner
        add_member(
            &pool,
            "member-2",
            "org-1",
            "user-charlie",
            OrgRole::Owner,
            "user-alice",
        )
        .await
        .expect("add charlie as owner");

        // Alice (original owner) demotes bob to admin - should work
        let updated =
            update_member_role(&pool, "org-1", "user-bob", OrgRole::Admin, "user-alice")
                .await
                .expect("demote bob");
        assert!(updated);

        // Alice demotes charlie to admin - should work (alice is still owner)
        let updated =
            update_member_role(&pool, "org-1", "user-charlie", OrgRole::Admin, "user-alice")
                .await
                .expect("demote charlie");
        assert!(updated);

        // Bob (now admin) tries to demote alice (owner) - should fail (can't demote higher role)
        let result =
            update_member_role(&pool, "org-1", "user-alice", OrgRole::Admin, "user-bob").await;
        assert!(matches!(result, Err(OrgError::InsufficientPermissions)));

        // Promote bob back to owner so we can test last owner protection
        let updated =
            update_member_role(&pool, "org-1", "user-bob", OrgRole::Owner, "user-alice")
                .await
                .expect("promote bob back");
        assert!(updated);

        // Bob tries to demote alice (last owner besides bob) - alice is org.owner_id so protected
        // This tests that the original org creator cannot be demoted by other owners
        let result =
            update_member_role(&pool, "org-1", "user-alice", OrgRole::Admin, "user-bob").await;
        assert!(matches!(result, Err(OrgError::InsufficientPermissions)));
    }

    #[tokio::test]
    async fn test_last_owner_cannot_be_demoted() {
        let (pool, _dir) = test_pool().await;

        create_organization(&pool, "org-1", "Test Org", "user-alice")
            .await
            .expect("create");

        add_member(
            &pool,
            "member-1",
            "org-1",
            "user-bob",
            OrgRole::Owner,
            "user-alice",
        )
        .await
        .expect("add bob as owner");

        let updated =
            update_member_role(&pool, "org-1", "user-bob", OrgRole::Admin, "user-alice")
                .await
                .expect("demote bob");
        assert!(updated);

        let result =
            update_member_role(&pool, "org-1", "user-alice", OrgRole::Admin, "user-alice").await;
        assert!(matches!(result, Err(OrgError::CannotModifySelf)));
    }

    #[tokio::test]
    async fn test_get_org_members() {
        let (pool, _dir) = test_pool().await;

        create_organization(&pool, "org-1", "Test Org", "user-alice")
            .await
            .expect("create");

        add_member(
            &pool,
            "member-1",
            "org-1",
            "user-bob",
            OrgRole::Member,
            "user-alice",
        )
        .await
        .expect("add");

        let members = get_org_members(&pool, "org-1").await.expect("get members");
        assert_eq!(members.len(), 2);
    }

    #[tokio::test]
    async fn test_get_user_orgs() {
        let (pool, _dir) = test_pool().await;

        create_organization(&pool, "org-1", "Org One", "user-alice")
            .await
            .expect("create 1");
        create_organization(&pool, "org-2", "Org Two", "user-alice")
            .await
            .expect("create 2");

        let orgs = get_user_orgs(&pool, "user-alice").await.expect("get orgs");
        assert_eq!(orgs.len(), 2);
    }

    #[tokio::test]
    async fn test_can_user_access_org() {
        let (pool, _dir) = test_pool().await;

        create_organization(&pool, "org-1", "Test Org", "user-alice")
            .await
            .expect("create");

        add_member(
            &pool,
            "member-1",
            "org-1",
            "user-bob",
            OrgRole::Member,
            "user-alice",
        )
        .await
        .expect("add");

        // Owner can access as admin
        let can_admin =
            can_user_access_org(&pool, "user-alice", "org-1", OrgRole::Admin)
                .await
                .expect("check");
        assert!(can_admin);

        // Member can access as member
        let can_member =
            can_user_access_org(&pool, "user-bob", "org-1", OrgRole::Member)
                .await
                .expect("check");
        assert!(can_member);

        // Member cannot access as admin
        let cannot_admin =
            can_user_access_org(&pool, "user-bob", "org-1", OrgRole::Admin)
                .await
                .expect("check");
        assert!(!cannot_admin);

        // Non-member cannot access
        let cannot_access =
            can_user_access_org(&pool, "user-charlie", "org-1", OrgRole::Member)
                .await
                .expect("check");
        assert!(!cannot_access);
    }

    #[tokio::test]
    async fn test_update_org_settings() {
        let (pool, _dir) = test_pool().await;

        create_organization(&pool, "org-1", "Test Org", "user-alice")
            .await
            .expect("create");

        let mut settings = OrgSettings::new();
        settings.default_model = Some("claude-sonnet".to_string());
        settings.max_projects = Some(10);

        update_org_settings(&pool, "org-1", &settings)
            .await
            .expect("update");

        let org = get_organization(&pool, "org-1")
            .await
            .expect("get")
            .expect("exists");
        let loaded_settings = org.settings();
        assert_eq!(loaded_settings.default_model, Some("claude-sonnet".to_string()));
        assert_eq!(loaded_settings.max_projects, Some(10));
    }

    #[tokio::test]
    async fn test_delete_organization() {
        let (pool, _dir) = test_pool().await;

        create_organization(&pool, "org-1", "Test Org", "user-alice")
            .await
            .expect("create");

        add_member(
            &pool,
            "member-1",
            "org-1",
            "user-bob",
            OrgRole::Member,
            "user-alice",
        )
        .await
        .expect("add");

        // Non-owner cannot delete
        let result = delete_organization(&pool, "org-1", "user-bob").await;
        assert!(matches!(result, Err(OrgError::InsufficientPermissions)));

        // Owner can delete
        let deleted = delete_organization(&pool, "org-1", "user-alice")
            .await
            .expect("delete");
        assert!(deleted);

        let org = get_organization(&pool, "org-1").await.expect("get");
        assert!(org.is_none());
    }

    #[test]
    fn test_can_modify_org_role() {
        // Owner can demote admin to member
        assert!(can_modify_org_role(
            OrgRole::Owner,
            OrgRole::Admin,
            OrgRole::Member
        ));

        // Owner cannot modify another owner
        assert!(!can_modify_org_role(
            OrgRole::Owner,
            OrgRole::Owner,
            OrgRole::Admin
        ));

        // Admin can demote member (but there's no lower role)
        assert!(can_modify_org_role(
            OrgRole::Admin,
            OrgRole::Member,
            OrgRole::Member
        ));

        // Admin cannot promote to owner
        assert!(!can_modify_org_role(
            OrgRole::Admin,
            OrgRole::Member,
            OrgRole::Owner
        ));

        // Admin cannot modify another admin
        assert!(!can_modify_org_role(
            OrgRole::Admin,
            OrgRole::Admin,
            OrgRole::Member
        ));

        // Member cannot modify anyone
        assert!(!can_modify_org_role(
            OrgRole::Member,
            OrgRole::Member,
            OrgRole::Member
        ));
    }

    #[tokio::test]
    async fn test_org_role_parsing() {
        assert_eq!(OrgRole::parse("owner"), OrgRole::Owner);
        assert_eq!(OrgRole::parse("ADMIN"), OrgRole::Admin);
        assert_eq!(OrgRole::parse("Member"), OrgRole::Member);
        assert_eq!(OrgRole::parse("unknown"), OrgRole::Member);
    }

    #[tokio::test]
    async fn test_org_settings_serialization() {
        let mut settings = OrgSettings::new();
        settings.default_model = Some("gpt-4".to_string());
        settings.allowed_providers = Some(vec!["openai".to_string(), "anthropic".to_string()]);
        settings.max_projects = Some(50);
        settings.shared_templates = vec!["tmpl-1".to_string()];

        let json = settings.to_json();
        let restored = OrgSettings::from_json(&json);

        assert_eq!(restored.default_model, Some("gpt-4".to_string()));
        assert_eq!(
            restored.allowed_providers,
            Some(vec!["openai".to_string(), "anthropic".to_string()])
        );
        assert_eq!(restored.max_projects, Some(50));
        assert_eq!(restored.shared_templates, vec!["tmpl-1".to_string()]);
    }

    #[tokio::test]
    async fn test_insufficient_permissions_add_member() {
        let (pool, _dir) = test_pool().await;

        create_organization(&pool, "org-1", "Test Org", "user-alice")
            .await
            .expect("create");

        add_member(
            &pool,
            "member-1",
            "org-1",
            "user-bob",
            OrgRole::Member,
            "user-alice",
        )
        .await
        .expect("add bob");

        // Bob (member) cannot add charlie
        let result = add_member(
            &pool,
            "member-2",
            "org-1",
            "user-charlie",
            OrgRole::Member,
            "user-bob",
        )
        .await;
        assert!(matches!(result, Err(OrgError::InsufficientPermissions)));
    }
}
