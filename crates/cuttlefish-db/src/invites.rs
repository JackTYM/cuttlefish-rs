//! Project invite database operations.
//!
//! Handles inviting users to projects via email with secure tokens.

use chrono::{Duration, Utc};
use sqlx::SqlitePool;

use crate::models::{ProjectInvite, ProjectRole};

const INVITE_EXPIRY_DAYS: i64 = 7;
const TOKEN_BYTES: usize = 32;

/// Create the project_invites table and indexes.
pub async fn create_project_invites_table(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS project_invites (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL,
    email TEXT NOT NULL,
    role TEXT NOT NULL,
    token TEXT NOT NULL UNIQUE,
    invited_by TEXT NOT NULL,
    created_at TEXT NOT NULL,
    expires_at TEXT NOT NULL,
    accepted_at TEXT,
    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE,
    FOREIGN KEY (invited_by) REFERENCES users(id) ON DELETE SET NULL
)"#,
    )
    .execute(pool)
    .await?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_project_invites_token ON project_invites(token)")
        .execute(pool)
        .await?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_project_invites_email ON project_invites(email)")
        .execute(pool)
        .await?;

    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_project_invites_project ON project_invites(project_id)",
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// Generate a URL-safe random token.
pub fn generate_invite_token() -> String {
    use std::fmt::Write;
    let mut bytes = [0u8; TOKEN_BYTES];
    getrandom::fill(&mut bytes).expect("failed to generate random bytes");

    let mut token = String::with_capacity(TOKEN_BYTES * 2);
    for byte in bytes {
        write!(&mut token, "{byte:02x}").expect("write to string");
    }
    token
}

/// Create a new project invite.
pub async fn create_invite(
    pool: &SqlitePool,
    id: &str,
    project_id: &str,
    email: &str,
    role: ProjectRole,
    invited_by: &str,
) -> Result<ProjectInvite, sqlx::Error> {
    let now = Utc::now();
    let expires_at = now + Duration::days(INVITE_EXPIRY_DAYS);
    let token = generate_invite_token();

    sqlx::query_as::<_, ProjectInvite>(
        r#"INSERT INTO project_invites (id, project_id, email, role, token, invited_by, created_at, expires_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        RETURNING *"#,
    )
    .bind(id)
    .bind(project_id)
    .bind(email)
    .bind(role.as_str())
    .bind(&token)
    .bind(invited_by)
    .bind(now.to_rfc3339())
    .bind(expires_at.to_rfc3339())
    .fetch_one(pool)
    .await
}

/// Get an invite by its token.
pub async fn get_invite_by_token(
    pool: &SqlitePool,
    token: &str,
) -> Result<Option<ProjectInvite>, sqlx::Error> {
    sqlx::query_as::<_, ProjectInvite>("SELECT * FROM project_invites WHERE token = ?")
        .bind(token)
        .fetch_optional(pool)
        .await
}

/// Get an invite by ID.
pub async fn get_invite_by_id(
    pool: &SqlitePool,
    id: &str,
) -> Result<Option<ProjectInvite>, sqlx::Error> {
    sqlx::query_as::<_, ProjectInvite>("SELECT * FROM project_invites WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await
}

/// Get all pending invites for an email address.
pub async fn get_pending_invites_for_email(
    pool: &SqlitePool,
    email: &str,
) -> Result<Vec<ProjectInvite>, sqlx::Error> {
    let now = Utc::now().to_rfc3339();
    sqlx::query_as::<_, ProjectInvite>(
        "SELECT * FROM project_invites WHERE email = ? AND accepted_at IS NULL AND expires_at > ? ORDER BY created_at DESC",
    )
    .bind(email)
    .bind(&now)
    .fetch_all(pool)
    .await
}

/// Get all invites for a project.
pub async fn get_project_invites(
    pool: &SqlitePool,
    project_id: &str,
) -> Result<Vec<ProjectInvite>, sqlx::Error> {
    sqlx::query_as::<_, ProjectInvite>(
        "SELECT * FROM project_invites WHERE project_id = ? ORDER BY created_at DESC",
    )
    .bind(project_id)
    .fetch_all(pool)
    .await
}

/// Get pending (not accepted, not expired) invites for a project.
pub async fn get_pending_project_invites(
    pool: &SqlitePool,
    project_id: &str,
) -> Result<Vec<ProjectInvite>, sqlx::Error> {
    let now = Utc::now().to_rfc3339();
    sqlx::query_as::<_, ProjectInvite>(
        "SELECT * FROM project_invites WHERE project_id = ? AND accepted_at IS NULL AND expires_at > ? ORDER BY created_at DESC",
    )
    .bind(project_id)
    .bind(&now)
    .fetch_all(pool)
    .await
}

/// Accept an invite by token.
pub async fn accept_invite(
    pool: &SqlitePool,
    token: &str,
) -> Result<Option<ProjectInvite>, sqlx::Error> {
    let now = Utc::now().to_rfc3339();

    sqlx::query_as::<_, ProjectInvite>(
        r#"UPDATE project_invites 
        SET accepted_at = ? 
        WHERE token = ? AND accepted_at IS NULL AND expires_at > ?
        RETURNING *"#,
    )
    .bind(&now)
    .bind(token)
    .bind(&now)
    .fetch_optional(pool)
    .await
}

/// Delete an invite (revoke it).
pub async fn delete_invite(pool: &SqlitePool, id: &str) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM project_invites WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;

    Ok(result.rows_affected() > 0)
}

/// Delete all expired invites.
pub async fn expire_old_invites(pool: &SqlitePool) -> Result<u64, sqlx::Error> {
    let now = Utc::now().to_rfc3339();
    let result =
        sqlx::query("DELETE FROM project_invites WHERE expires_at < ? AND accepted_at IS NULL")
            .bind(&now)
            .execute(pool)
            .await?;

    Ok(result.rows_affected())
}

/// Check if an invite already exists for a project and email.
pub async fn invite_exists(
    pool: &SqlitePool,
    project_id: &str,
    email: &str,
) -> Result<bool, sqlx::Error> {
    let now = Utc::now().to_rfc3339();
    let count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM project_invites WHERE project_id = ? AND email = ? AND accepted_at IS NULL AND expires_at > ?",
    )
    .bind(project_id)
    .bind(email)
    .bind(&now)
    .fetch_one(pool)
    .await?;

    Ok(count > 0)
}

/// Mask an email address for privacy (e.g., "test@example.com" -> "t***@e***.com").
pub fn mask_email(email: &str) -> String {
    let parts: Vec<&str> = email.split('@').collect();
    if parts.len() != 2 {
        return "***".to_string();
    }

    let local = parts[0];
    let domain = parts[1];

    let masked_local = if local.is_empty() {
        "***".to_string()
    } else {
        format!("{}***", &local[..1])
    };

    let domain_parts: Vec<&str> = domain.split('.').collect();
    let masked_domain = if domain_parts.is_empty() {
        "***".to_string()
    } else {
        let first = domain_parts[0];
        let masked_first = if first.is_empty() {
            "***".to_string()
        } else {
            format!("{}***", &first[..1])
        };
        if domain_parts.len() > 1 {
            format!("{}.{}", masked_first, domain_parts[1..].join("."))
        } else {
            masked_first
        }
    };

    format!("{masked_local}@{masked_domain}")
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

        sqlx::query(
            r#"CREATE TABLE IF NOT EXISTS projects (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL
            )"#,
        )
        .execute(&pool)
        .await
        .expect("create projects table");

        create_users_table(&pool).await.expect("create users table");
        create_project_invites_table(&pool)
            .await
            .expect("create project_invites table");

        sqlx::query("INSERT INTO projects (id, name) VALUES ('proj-1', 'Test Project')")
            .execute(&pool)
            .await
            .expect("create test project");

        sqlx::query(
            "INSERT INTO users (id, email, password_hash, created_at, updated_at) VALUES ('user-owner', 'owner@example.com', 'hash', datetime('now'), datetime('now'))",
        )
        .execute(&pool)
        .await
        .expect("create owner user");

        (pool, dir)
    }

    #[tokio::test]
    async fn test_create_invite() {
        let (pool, _dir) = test_pool().await;

        let invite = create_invite(
            &pool,
            "invite-1",
            "proj-1",
            "newuser@example.com",
            ProjectRole::Member,
            "user-owner",
        )
        .await
        .expect("create invite");

        assert_eq!(invite.project_id, "proj-1");
        assert_eq!(invite.email, "newuser@example.com");
        assert_eq!(invite.role, "member");
        assert!(!invite.token.is_empty());
        assert!(invite.accepted_at.is_none());
    }

    #[tokio::test]
    async fn test_get_invite_by_token() {
        let (pool, _dir) = test_pool().await;

        let invite = create_invite(
            &pool,
            "invite-1",
            "proj-1",
            "test@example.com",
            ProjectRole::Member,
            "user-owner",
        )
        .await
        .expect("create");

        let fetched = get_invite_by_token(&pool, &invite.token)
            .await
            .expect("get")
            .expect("exists");
        assert_eq!(fetched.id, "invite-1");
    }

    #[tokio::test]
    async fn test_accept_invite() {
        let (pool, _dir) = test_pool().await;

        let invite = create_invite(
            &pool,
            "invite-1",
            "proj-1",
            "test@example.com",
            ProjectRole::Member,
            "user-owner",
        )
        .await
        .expect("create");

        let accepted = accept_invite(&pool, &invite.token)
            .await
            .expect("accept")
            .expect("found");
        assert!(accepted.accepted_at.is_some());

        let again = accept_invite(&pool, &invite.token)
            .await
            .expect("accept again");
        assert!(again.is_none());
    }

    #[tokio::test]
    async fn test_get_pending_invites_for_email() {
        let (pool, _dir) = test_pool().await;

        create_invite(
            &pool,
            "invite-1",
            "proj-1",
            "test@example.com",
            ProjectRole::Member,
            "user-owner",
        )
        .await
        .expect("create");

        let pending = get_pending_invites_for_email(&pool, "test@example.com")
            .await
            .expect("get");
        assert_eq!(pending.len(), 1);

        let none = get_pending_invites_for_email(&pool, "other@example.com")
            .await
            .expect("get");
        assert!(none.is_empty());
    }

    #[tokio::test]
    async fn test_delete_invite() {
        let (pool, _dir) = test_pool().await;

        create_invite(
            &pool,
            "invite-1",
            "proj-1",
            "test@example.com",
            ProjectRole::Member,
            "user-owner",
        )
        .await
        .expect("create");

        let deleted = delete_invite(&pool, "invite-1").await.expect("delete");
        assert!(deleted);

        let fetched = get_invite_by_id(&pool, "invite-1").await.expect("get");
        assert!(fetched.is_none());
    }

    #[tokio::test]
    async fn test_invite_exists() {
        let (pool, _dir) = test_pool().await;

        let exists = invite_exists(&pool, "proj-1", "test@example.com")
            .await
            .expect("check");
        assert!(!exists);

        create_invite(
            &pool,
            "invite-1",
            "proj-1",
            "test@example.com",
            ProjectRole::Member,
            "user-owner",
        )
        .await
        .expect("create");

        let exists = invite_exists(&pool, "proj-1", "test@example.com")
            .await
            .expect("check");
        assert!(exists);
    }

    #[tokio::test]
    async fn test_expire_old_invites() {
        let (pool, _dir) = test_pool().await;

        sqlx::query(
            r#"INSERT INTO project_invites (id, project_id, email, role, token, invited_by, created_at, expires_at)
            VALUES ('expired-1', 'proj-1', 'old@example.com', 'member', 'token-old', 'user-owner', datetime('now', '-10 days'), datetime('now', '-3 days'))"#,
        )
        .execute(&pool)
        .await
        .expect("create expired invite");

        create_invite(
            &pool,
            "valid-1",
            "proj-1",
            "new@example.com",
            ProjectRole::Member,
            "user-owner",
        )
        .await
        .expect("create valid invite");

        let expired = expire_old_invites(&pool).await.expect("expire");
        assert_eq!(expired, 1);

        let fetched = get_invite_by_id(&pool, "expired-1").await.expect("get");
        assert!(fetched.is_none());

        let valid = get_invite_by_id(&pool, "valid-1").await.expect("get");
        assert!(valid.is_some());
    }

    #[test]
    fn test_generate_invite_token() {
        let token1 = generate_invite_token();
        let token2 = generate_invite_token();

        assert_eq!(token1.len(), TOKEN_BYTES * 2);
        assert_ne!(token1, token2);
        assert!(token1.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_mask_email() {
        assert_eq!(mask_email("test@example.com"), "t***@e***.com");
        assert_eq!(mask_email("a@b.co"), "a***@b***.co");
        assert_eq!(mask_email("x@y.z"), "x***@y***.z");
        assert_eq!(mask_email("invalid"), "***");
    }
}
