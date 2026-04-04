//! Project sharing database operations.
//!
//! This module provides functionality for sharing projects between users
//! with role-based access control.

use chrono::Utc;
use sqlx::SqlitePool;

use crate::models::{ProjectRole, ProjectShare};

/// Create the project_shares table and indexes.
pub async fn create_project_shares_table(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS project_shares (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL,
    user_id TEXT NOT NULL,
    role TEXT NOT NULL,
    shared_by TEXT NOT NULL,
    created_at TEXT NOT NULL,
    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (shared_by) REFERENCES users(id) ON DELETE SET NULL,
    UNIQUE(project_id, user_id)
)"#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_project_shares_project ON project_shares(project_id)",
    )
    .execute(pool)
    .await?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_project_shares_user ON project_shares(user_id)")
        .execute(pool)
        .await?;

    Ok(())
}

/// Share a project with a user.
///
/// Creates a new share record granting the specified role to the user.
/// Returns an error if the user already has access to the project.
pub async fn share_project(
    pool: &SqlitePool,
    id: &str,
    project_id: &str,
    user_id: &str,
    role: ProjectRole,
    shared_by: &str,
) -> Result<ProjectShare, sqlx::Error> {
    let now = Utc::now().to_rfc3339();

    sqlx::query_as::<_, ProjectShare>(
        r#"INSERT INTO project_shares (id, project_id, user_id, role, shared_by, created_at)
        VALUES (?, ?, ?, ?, ?, ?)
        RETURNING *"#,
    )
    .bind(id)
    .bind(project_id)
    .bind(user_id)
    .bind(role.as_str())
    .bind(shared_by)
    .bind(&now)
    .fetch_one(pool)
    .await
}

/// Get all shares for a project.
///
/// Returns a list of all users who have been granted access to the project.
/// Note: This does not expose email addresses for privacy.
pub async fn get_project_shares(
    pool: &SqlitePool,
    project_id: &str,
) -> Result<Vec<ProjectShare>, sqlx::Error> {
    sqlx::query_as::<_, ProjectShare>(
        "SELECT * FROM project_shares WHERE project_id = ? ORDER BY created_at ASC",
    )
    .bind(project_id)
    .fetch_all(pool)
    .await
}

/// Get a specific share by project and user.
pub async fn get_share(
    pool: &SqlitePool,
    project_id: &str,
    user_id: &str,
) -> Result<Option<ProjectShare>, sqlx::Error> {
    sqlx::query_as::<_, ProjectShare>(
        "SELECT * FROM project_shares WHERE project_id = ? AND user_id = ?",
    )
    .bind(project_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await
}

/// Get all projects shared with a user.
///
/// Returns all share records for projects the user has been granted access to.
pub async fn get_user_projects(
    pool: &SqlitePool,
    user_id: &str,
) -> Result<Vec<ProjectShare>, sqlx::Error> {
    sqlx::query_as::<_, ProjectShare>(
        "SELECT * FROM project_shares WHERE user_id = ? ORDER BY created_at DESC",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
}

/// Remove a share, revoking a user's access to a project.
///
/// Returns true if a share was removed, false if no share existed.
pub async fn remove_share(
    pool: &SqlitePool,
    project_id: &str,
    user_id: &str,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM project_shares WHERE project_id = ? AND user_id = ?")
        .bind(project_id)
        .bind(user_id)
        .execute(pool)
        .await?;

    Ok(result.rows_affected() > 0)
}

/// Update a user's role for a project.
///
/// Returns true if the role was updated, false if no share existed.
pub async fn update_share_role(
    pool: &SqlitePool,
    project_id: &str,
    user_id: &str,
    new_role: ProjectRole,
) -> Result<bool, sqlx::Error> {
    let result =
        sqlx::query("UPDATE project_shares SET role = ? WHERE project_id = ? AND user_id = ?")
            .bind(new_role.as_str())
            .bind(project_id)
            .bind(user_id)
            .execute(pool)
            .await?;

    Ok(result.rows_affected() > 0)
}

/// Check if a user can access a project with at least the required role.
///
/// Role hierarchy: owner > admin > member > viewer
/// Returns true if the user has the required role or higher.
pub async fn can_user_access(
    pool: &SqlitePool,
    user_id: &str,
    project_id: &str,
    required_role: ProjectRole,
) -> Result<bool, sqlx::Error> {
    let share = get_share(pool, project_id, user_id).await?;

    match share {
        Some(s) => {
            let user_role = ProjectRole::parse(&s.role);
            Ok(user_role.has_at_least(required_role))
        }
        None => Ok(false),
    }
}

/// Get the role a user has for a project.
///
/// Returns None if the user has no access to the project.
pub async fn get_user_role(
    pool: &SqlitePool,
    user_id: &str,
    project_id: &str,
) -> Result<Option<ProjectRole>, sqlx::Error> {
    let share = get_share(pool, project_id, user_id).await?;
    Ok(share.map(|s| ProjectRole::parse(&s.role)))
}

/// Count shares by role for a project.
pub async fn count_shares_by_role(
    pool: &SqlitePool,
    project_id: &str,
    role: ProjectRole,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM project_shares WHERE project_id = ? AND role = ?",
    )
    .bind(project_id)
    .bind(role.as_str())
    .fetch_one(pool)
    .await
}

/// Check if a user is the only owner of a project.
///
/// Used to prevent removing the last owner.
pub async fn is_only_owner(
    pool: &SqlitePool,
    project_id: &str,
    user_id: &str,
) -> Result<bool, sqlx::Error> {
    let owner_count = count_shares_by_role(pool, project_id, ProjectRole::Owner).await?;
    if owner_count != 1 {
        return Ok(false);
    }

    let share = get_share(pool, project_id, user_id).await?;
    match share {
        Some(s) => Ok(s.role == ProjectRole::Owner.as_str()),
        None => Ok(false),
    }
}

/// Check if an actor can modify another user's role.
///
/// Rules:
/// - Owners can modify anyone except other owners (unless they're the only owner)
/// - Admins can modify members and viewers
/// - Members and viewers cannot modify roles
/// - No one can escalate to a role higher than their own
pub fn can_modify_role(
    actor_role: ProjectRole,
    target_current_role: ProjectRole,
    target_new_role: ProjectRole,
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
    actor_role.level() >= ProjectRole::Admin.level()
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
        create_project_shares_table(&pool)
            .await
            .expect("create project_shares table");

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

        sqlx::query(
            "INSERT INTO users (id, email, password_hash, created_at, updated_at) VALUES ('user-member', 'member@example.com', 'hash', datetime('now'), datetime('now'))",
        )
        .execute(&pool)
        .await
        .expect("create member user");

        sqlx::query(
            "INSERT INTO users (id, email, password_hash, created_at, updated_at) VALUES ('user-viewer', 'viewer@example.com', 'hash', datetime('now'), datetime('now'))",
        )
        .execute(&pool)
        .await
        .expect("create viewer user");

        (pool, dir)
    }

    #[tokio::test]
    async fn test_share_project() {
        let (pool, _dir) = test_pool().await;

        let share = share_project(
            &pool,
            "share-1",
            "proj-1",
            "user-member",
            ProjectRole::Member,
            "user-owner",
        )
        .await
        .expect("share project");

        assert_eq!(share.project_id, "proj-1");
        assert_eq!(share.user_id, "user-member");
        assert_eq!(share.role, "member");
        assert_eq!(share.shared_by, "user-owner");
    }

    #[tokio::test]
    async fn test_get_project_shares() {
        let (pool, _dir) = test_pool().await;

        share_project(
            &pool,
            "share-1",
            "proj-1",
            "user-owner",
            ProjectRole::Owner,
            "user-owner",
        )
        .await
        .expect("share owner");

        share_project(
            &pool,
            "share-2",
            "proj-1",
            "user-member",
            ProjectRole::Member,
            "user-owner",
        )
        .await
        .expect("share member");

        let shares = get_project_shares(&pool, "proj-1")
            .await
            .expect("get shares");
        assert_eq!(shares.len(), 2);
    }

    #[tokio::test]
    async fn test_get_user_projects() {
        let (pool, _dir) = test_pool().await;

        share_project(
            &pool,
            "share-1",
            "proj-1",
            "user-member",
            ProjectRole::Member,
            "user-owner",
        )
        .await
        .expect("share");

        let projects = get_user_projects(&pool, "user-member")
            .await
            .expect("get projects");
        assert_eq!(projects.len(), 1);
        assert_eq!(projects[0].project_id, "proj-1");
    }

    #[tokio::test]
    async fn test_remove_share() {
        let (pool, _dir) = test_pool().await;

        share_project(
            &pool,
            "share-1",
            "proj-1",
            "user-member",
            ProjectRole::Member,
            "user-owner",
        )
        .await
        .expect("share");

        let removed = remove_share(&pool, "proj-1", "user-member")
            .await
            .expect("remove");
        assert!(removed);

        let share = get_share(&pool, "proj-1", "user-member")
            .await
            .expect("get");
        assert!(share.is_none());
    }

    #[tokio::test]
    async fn test_update_share_role() {
        let (pool, _dir) = test_pool().await;

        share_project(
            &pool,
            "share-1",
            "proj-1",
            "user-member",
            ProjectRole::Member,
            "user-owner",
        )
        .await
        .expect("share");

        let updated = update_share_role(&pool, "proj-1", "user-member", ProjectRole::Admin)
            .await
            .expect("update");
        assert!(updated);

        let share = get_share(&pool, "proj-1", "user-member")
            .await
            .expect("get")
            .expect("exists");
        assert_eq!(share.role, "admin");
    }

    #[tokio::test]
    async fn test_can_user_access() {
        let (pool, _dir) = test_pool().await;

        share_project(
            &pool,
            "share-1",
            "proj-1",
            "user-member",
            ProjectRole::Member,
            "user-owner",
        )
        .await
        .expect("share");

        // Member can access as viewer
        let can_view = can_user_access(&pool, "user-member", "proj-1", ProjectRole::Viewer)
            .await
            .expect("check");
        assert!(can_view);

        // Member can access as member
        let can_member = can_user_access(&pool, "user-member", "proj-1", ProjectRole::Member)
            .await
            .expect("check");
        assert!(can_member);

        // Member cannot access as admin
        let can_admin = can_user_access(&pool, "user-member", "proj-1", ProjectRole::Admin)
            .await
            .expect("check");
        assert!(!can_admin);

        // Non-shared user cannot access
        let can_other = can_user_access(&pool, "user-viewer", "proj-1", ProjectRole::Viewer)
            .await
            .expect("check");
        assert!(!can_other);
    }

    #[tokio::test]
    async fn test_is_only_owner() {
        let (pool, _dir) = test_pool().await;

        share_project(
            &pool,
            "share-1",
            "proj-1",
            "user-owner",
            ProjectRole::Owner,
            "user-owner",
        )
        .await
        .expect("share owner");

        let is_only = is_only_owner(&pool, "proj-1", "user-owner")
            .await
            .expect("check");
        assert!(is_only);

        // Add another owner
        share_project(
            &pool,
            "share-2",
            "proj-1",
            "user-member",
            ProjectRole::Owner,
            "user-owner",
        )
        .await
        .expect("share second owner");

        let is_only = is_only_owner(&pool, "proj-1", "user-owner")
            .await
            .expect("check");
        assert!(!is_only);
    }

    #[tokio::test]
    async fn test_unique_constraint() {
        let (pool, _dir) = test_pool().await;

        share_project(
            &pool,
            "share-1",
            "proj-1",
            "user-member",
            ProjectRole::Member,
            "user-owner",
        )
        .await
        .expect("first share");

        let result = share_project(
            &pool,
            "share-2",
            "proj-1",
            "user-member",
            ProjectRole::Admin,
            "user-owner",
        )
        .await;
        assert!(result.is_err());
    }

    #[test]
    fn test_can_modify_role() {
        // Owner can demote admin to member
        assert!(can_modify_role(
            ProjectRole::Owner,
            ProjectRole::Admin,
            ProjectRole::Member
        ));

        // Owner cannot modify another owner
        assert!(!can_modify_role(
            ProjectRole::Owner,
            ProjectRole::Owner,
            ProjectRole::Admin
        ));

        // Admin can demote member to viewer
        assert!(can_modify_role(
            ProjectRole::Admin,
            ProjectRole::Member,
            ProjectRole::Viewer
        ));

        // Admin cannot promote to owner
        assert!(!can_modify_role(
            ProjectRole::Admin,
            ProjectRole::Member,
            ProjectRole::Owner
        ));

        // Admin cannot modify another admin
        assert!(!can_modify_role(
            ProjectRole::Admin,
            ProjectRole::Admin,
            ProjectRole::Member
        ));

        // Member cannot modify anyone
        assert!(!can_modify_role(
            ProjectRole::Member,
            ProjectRole::Viewer,
            ProjectRole::Member
        ));
    }

    #[tokio::test]
    async fn test_get_user_role() {
        let (pool, _dir) = test_pool().await;

        share_project(
            &pool,
            "share-1",
            "proj-1",
            "user-member",
            ProjectRole::Member,
            "user-owner",
        )
        .await
        .expect("share");

        let role = get_user_role(&pool, "user-member", "proj-1")
            .await
            .expect("get role")
            .expect("has role");
        assert_eq!(role, ProjectRole::Member);

        let no_role = get_user_role(&pool, "user-viewer", "proj-1")
            .await
            .expect("get role");
        assert!(no_role.is_none());
    }
}
