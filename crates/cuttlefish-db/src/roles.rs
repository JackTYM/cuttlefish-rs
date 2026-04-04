//! Project membership and role database operations.

use chrono::Utc;
use sqlx::SqlitePool;

use crate::models::ProjectMember;

/// Create the project_members table and indexes.
pub async fn create_project_members_table(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS project_members (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL,
    user_id TEXT NOT NULL,
    role TEXT NOT NULL,
    invited_by TEXT,
    created_at TEXT NOT NULL,
    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (invited_by) REFERENCES users(id) ON DELETE SET NULL,
    UNIQUE(project_id, user_id)
)"#,
    )
    .execute(pool)
    .await?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_project_members_project ON project_members(project_id)")
        .execute(pool)
        .await?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_project_members_user ON project_members(user_id)")
        .execute(pool)
        .await?;

    Ok(())
}

/// Add a member to a project.
pub async fn add_project_member(
    pool: &SqlitePool,
    id: &str,
    project_id: &str,
    user_id: &str,
    role: &str,
    invited_by: Option<&str>,
) -> Result<ProjectMember, sqlx::Error> {
    let now = Utc::now().to_rfc3339();

    sqlx::query_as::<_, ProjectMember>(
        r#"INSERT INTO project_members (id, project_id, user_id, role, invited_by, created_at)
        VALUES (?, ?, ?, ?, ?, ?)
        RETURNING *"#,
    )
    .bind(id)
    .bind(project_id)
    .bind(user_id)
    .bind(role)
    .bind(invited_by)
    .bind(&now)
    .fetch_one(pool)
    .await
}

/// Get a project member by project and user ID.
pub async fn get_project_member(
    pool: &SqlitePool,
    project_id: &str,
    user_id: &str,
) -> Result<Option<ProjectMember>, sqlx::Error> {
    sqlx::query_as::<_, ProjectMember>(
        "SELECT * FROM project_members WHERE project_id = ? AND user_id = ?",
    )
    .bind(project_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await
}

/// List all members of a project.
pub async fn list_project_members(
    pool: &SqlitePool,
    project_id: &str,
) -> Result<Vec<ProjectMember>, sqlx::Error> {
    sqlx::query_as::<_, ProjectMember>(
        "SELECT * FROM project_members WHERE project_id = ? ORDER BY created_at ASC",
    )
    .bind(project_id)
    .fetch_all(pool)
    .await
}

/// List all projects a user is a member of.
pub async fn list_user_projects(
    pool: &SqlitePool,
    user_id: &str,
) -> Result<Vec<ProjectMember>, sqlx::Error> {
    sqlx::query_as::<_, ProjectMember>(
        "SELECT * FROM project_members WHERE user_id = ? ORDER BY created_at DESC",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
}

/// Update a member's role.
pub async fn update_member_role(
    pool: &SqlitePool,
    project_id: &str,
    user_id: &str,
    new_role: &str,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        "UPDATE project_members SET role = ? WHERE project_id = ? AND user_id = ?",
    )
    .bind(new_role)
    .bind(project_id)
    .bind(user_id)
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}

/// Remove a member from a project.
pub async fn remove_project_member(
    pool: &SqlitePool,
    project_id: &str,
    user_id: &str,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        "DELETE FROM project_members WHERE project_id = ? AND user_id = ?",
    )
    .bind(project_id)
    .bind(user_id)
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}

/// Count members with a specific role in a project.
pub async fn count_members_by_role(
    pool: &SqlitePool,
    project_id: &str,
    role: &str,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM project_members WHERE project_id = ? AND role = ?",
    )
    .bind(project_id)
    .bind(role)
    .fetch_one(pool)
    .await
}

/// Check if a user is a member of a project.
pub async fn is_project_member(
    pool: &SqlitePool,
    project_id: &str,
    user_id: &str,
) -> Result<bool, sqlx::Error> {
    let count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM project_members WHERE project_id = ? AND user_id = ?",
    )
    .bind(project_id)
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    Ok(count > 0)
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
        create_project_members_table(&pool).await.expect("create project_members table");

        sqlx::query("INSERT INTO projects (id, name) VALUES ('proj-1', 'Test Project')")
            .execute(&pool)
            .await
            .expect("create test project");

        sqlx::query(
            "INSERT INTO users (id, email, password_hash, created_at, updated_at) VALUES ('user-1', 'owner@example.com', 'hash', datetime('now'), datetime('now'))",
        )
        .execute(&pool)
        .await
        .expect("create owner user");

        sqlx::query(
            "INSERT INTO users (id, email, password_hash, created_at, updated_at) VALUES ('user-2', 'member@example.com', 'hash', datetime('now'), datetime('now'))",
        )
        .execute(&pool)
        .await
        .expect("create member user");

        (pool, dir)
    }

    #[tokio::test]
    async fn test_add_and_get_member() {
        let (pool, _dir) = test_pool().await;

        let member = add_project_member(
            &pool,
            "member-1",
            "proj-1",
            "user-1",
            "owner",
            None,
        )
        .await
        .expect("add member");

        assert_eq!(member.project_id, "proj-1");
        assert_eq!(member.user_id, "user-1");
        assert_eq!(member.role, "owner");

        let fetched = get_project_member(&pool, "proj-1", "user-1")
            .await
            .expect("get")
            .expect("exists");
        assert_eq!(fetched.id, "member-1");
    }

    #[tokio::test]
    async fn test_list_project_members() {
        let (pool, _dir) = test_pool().await;

        add_project_member(&pool, "m-1", "proj-1", "user-1", "owner", None)
            .await
            .expect("add owner");

        add_project_member(&pool, "m-2", "proj-1", "user-2", "member", Some("user-1"))
            .await
            .expect("add member");

        let members = list_project_members(&pool, "proj-1").await.expect("list");
        assert_eq!(members.len(), 2);
    }

    #[tokio::test]
    async fn test_list_user_projects() {
        let (pool, _dir) = test_pool().await;

        add_project_member(&pool, "m-1", "proj-1", "user-1", "owner", None)
            .await
            .expect("add");

        let projects = list_user_projects(&pool, "user-1").await.expect("list");
        assert_eq!(projects.len(), 1);
        assert_eq!(projects[0].project_id, "proj-1");
    }

    #[tokio::test]
    async fn test_update_member_role() {
        let (pool, _dir) = test_pool().await;

        add_project_member(&pool, "m-1", "proj-1", "user-2", "member", None)
            .await
            .expect("add");

        let updated = update_member_role(&pool, "proj-1", "user-2", "admin")
            .await
            .expect("update");
        assert!(updated);

        let member = get_project_member(&pool, "proj-1", "user-2")
            .await
            .expect("get")
            .expect("exists");
        assert_eq!(member.role, "admin");
    }

    #[tokio::test]
    async fn test_remove_project_member() {
        let (pool, _dir) = test_pool().await;

        add_project_member(&pool, "m-1", "proj-1", "user-2", "member", None)
            .await
            .expect("add");

        let removed = remove_project_member(&pool, "proj-1", "user-2")
            .await
            .expect("remove");
        assert!(removed);

        let member = get_project_member(&pool, "proj-1", "user-2")
            .await
            .expect("get");
        assert!(member.is_none());
    }

    #[tokio::test]
    async fn test_count_members_by_role() {
        let (pool, _dir) = test_pool().await;

        add_project_member(&pool, "m-1", "proj-1", "user-1", "owner", None)
            .await
            .expect("add owner");

        add_project_member(&pool, "m-2", "proj-1", "user-2", "member", None)
            .await
            .expect("add member");

        let owner_count = count_members_by_role(&pool, "proj-1", "owner")
            .await
            .expect("count");
        assert_eq!(owner_count, 1);

        let member_count = count_members_by_role(&pool, "proj-1", "member")
            .await
            .expect("count");
        assert_eq!(member_count, 1);
    }

    #[tokio::test]
    async fn test_is_project_member() {
        let (pool, _dir) = test_pool().await;

        add_project_member(&pool, "m-1", "proj-1", "user-1", "owner", None)
            .await
            .expect("add");

        let is_member = is_project_member(&pool, "proj-1", "user-1")
            .await
            .expect("check");
        assert!(is_member);

        let not_member = is_project_member(&pool, "proj-1", "user-2")
            .await
            .expect("check");
        assert!(!not_member);
    }

    #[tokio::test]
    async fn test_unique_constraint() {
        let (pool, _dir) = test_pool().await;

        add_project_member(&pool, "m-1", "proj-1", "user-1", "owner", None)
            .await
            .expect("add first");

        let result = add_project_member(&pool, "m-2", "proj-1", "user-1", "member", None).await;
        assert!(result.is_err());
    }
}
