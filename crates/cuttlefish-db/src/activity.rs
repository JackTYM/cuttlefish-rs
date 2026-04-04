//! Activity logging database operations.
//!
//! Tracks user actions on projects for audit and collaboration visibility.

use chrono::Utc;
use sqlx::SqlitePool;

pub use crate::models::ActivityAction;
use crate::models::ActivityEntry;

const DEFAULT_PAGE_SIZE: i64 = 50;

/// Create the activity_log table and indexes.
pub async fn create_activity_log_table(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS activity_log (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL,
    user_id TEXT NOT NULL,
    action TEXT NOT NULL,
    details TEXT,
    created_at TEXT NOT NULL,
    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE SET NULL
)"#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_activity_log_project ON activity_log(project_id, created_at DESC)",
    )
    .execute(pool)
    .await?;

    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_activity_log_user ON activity_log(user_id, created_at DESC)",
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// Log an activity entry.
pub async fn log_activity(
    pool: &SqlitePool,
    id: &str,
    project_id: &str,
    user_id: &str,
    action: &ActivityAction,
    details: Option<&str>,
) -> Result<ActivityEntry, sqlx::Error> {
    let now = Utc::now().to_rfc3339();
    let action_json = serde_json::to_string(action).unwrap_or_else(|_| "{}".to_string());

    sqlx::query_as::<_, ActivityEntry>(
        r#"INSERT INTO activity_log (id, project_id, user_id, action, details, created_at)
        VALUES (?, ?, ?, ?, ?, ?)
        RETURNING *"#,
    )
    .bind(id)
    .bind(project_id)
    .bind(user_id)
    .bind(&action_json)
    .bind(details)
    .bind(&now)
    .fetch_one(pool)
    .await
}

/// Get activity entries for a project with pagination.
pub async fn get_project_activity(
    pool: &SqlitePool,
    project_id: &str,
    limit: Option<i64>,
    before: Option<&str>,
) -> Result<Vec<ActivityEntry>, sqlx::Error> {
    let limit = limit.unwrap_or(DEFAULT_PAGE_SIZE);

    if let Some(before_ts) = before {
        sqlx::query_as::<_, ActivityEntry>(
            "SELECT * FROM activity_log WHERE project_id = ? AND created_at < ? ORDER BY created_at DESC LIMIT ?",
        )
        .bind(project_id)
        .bind(before_ts)
        .bind(limit)
        .fetch_all(pool)
        .await
    } else {
        sqlx::query_as::<_, ActivityEntry>(
            "SELECT * FROM activity_log WHERE project_id = ? ORDER BY created_at DESC LIMIT ?",
        )
        .bind(project_id)
        .bind(limit)
        .fetch_all(pool)
        .await
    }
}

/// Get activity entries for a user across all projects.
pub async fn get_user_activity(
    pool: &SqlitePool,
    user_id: &str,
    limit: Option<i64>,
    before: Option<&str>,
) -> Result<Vec<ActivityEntry>, sqlx::Error> {
    let limit = limit.unwrap_or(DEFAULT_PAGE_SIZE);

    if let Some(before_ts) = before {
        sqlx::query_as::<_, ActivityEntry>(
            "SELECT * FROM activity_log WHERE user_id = ? AND created_at < ? ORDER BY created_at DESC LIMIT ?",
        )
        .bind(user_id)
        .bind(before_ts)
        .bind(limit)
        .fetch_all(pool)
        .await
    } else {
        sqlx::query_as::<_, ActivityEntry>(
            "SELECT * FROM activity_log WHERE user_id = ? ORDER BY created_at DESC LIMIT ?",
        )
        .bind(user_id)
        .bind(limit)
        .fetch_all(pool)
        .await
    }
}

/// Get a single activity entry by ID.
pub async fn get_activity_by_id(
    pool: &SqlitePool,
    id: &str,
) -> Result<Option<ActivityEntry>, sqlx::Error> {
    sqlx::query_as::<_, ActivityEntry>("SELECT * FROM activity_log WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await
}

/// Count activity entries for a project.
pub async fn count_project_activity(
    pool: &SqlitePool,
    project_id: &str,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM activity_log WHERE project_id = ?")
        .bind(project_id)
        .fetch_one(pool)
        .await
}

/// Delete old activity entries (for cleanup/retention policy).
pub async fn delete_old_activity(pool: &SqlitePool, before: &str) -> Result<u64, sqlx::Error> {
    let result = sqlx::query("DELETE FROM activity_log WHERE created_at < ?")
        .bind(before)
        .execute(pool)
        .await?;

    Ok(result.rows_affected())
}

/// Parse an activity entry's action field into an ActivityAction.
pub fn parse_activity_action(entry: &ActivityEntry) -> Option<ActivityAction> {
    serde_json::from_str(&entry.action).ok()
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
        create_activity_log_table(&pool)
            .await
            .expect("create activity_log table");

        sqlx::query("INSERT INTO projects (id, name) VALUES ('proj-1', 'Test Project')")
            .execute(&pool)
            .await
            .expect("create test project");

        sqlx::query(
            "INSERT INTO users (id, email, password_hash, created_at, updated_at) VALUES ('user-1', 'test@example.com', 'hash', datetime('now'), datetime('now'))",
        )
        .execute(&pool)
        .await
        .expect("create test user");

        (pool, dir)
    }

    #[tokio::test]
    async fn test_log_activity() {
        let (pool, _dir) = test_pool().await;

        let entry = log_activity(
            &pool,
            "activity-1",
            "proj-1",
            "user-1",
            &ActivityAction::ProjectCreated,
            None,
        )
        .await
        .expect("log activity");

        assert_eq!(entry.project_id, "proj-1");
        assert_eq!(entry.user_id, "user-1");
        assert!(entry.action.contains("project_created"));
    }

    #[tokio::test]
    async fn test_log_activity_with_details() {
        let (pool, _dir) = test_pool().await;

        let action = ActivityAction::MemberAdded {
            member_id: "user-2".to_string(),
            role: "member".to_string(),
        };

        let entry = log_activity(
            &pool,
            "activity-1",
            "proj-1",
            "user-1",
            &action,
            Some("Added via invite"),
        )
        .await
        .expect("log activity");

        assert!(entry.action.contains("member_added"));
        assert_eq!(entry.details, Some("Added via invite".to_string()));
    }

    #[tokio::test]
    async fn test_get_project_activity() {
        let (pool, _dir) = test_pool().await;

        for i in 0..5 {
            log_activity(
                &pool,
                &format!("activity-{i}"),
                "proj-1",
                "user-1",
                &ActivityAction::ProjectCreated,
                None,
            )
            .await
            .expect("log");
        }

        let activities = get_project_activity(&pool, "proj-1", Some(3), None)
            .await
            .expect("get");
        assert_eq!(activities.len(), 3);

        let all = get_project_activity(&pool, "proj-1", None, None)
            .await
            .expect("get all");
        assert_eq!(all.len(), 5);
    }

    #[tokio::test]
    async fn test_get_project_activity_pagination() {
        let (pool, _dir) = test_pool().await;

        for i in 0..10 {
            sqlx::query(
                "INSERT INTO activity_log (id, project_id, user_id, action, created_at) VALUES (?, 'proj-1', 'user-1', '{}', ?)",
            )
            .bind(format!("activity-{i}"))
            .bind(format!("2024-01-{:02}T00:00:00Z", i + 1))
            .execute(&pool)
            .await
            .expect("insert");
        }

        let first_page = get_project_activity(&pool, "proj-1", Some(5), None)
            .await
            .expect("first page");
        assert_eq!(first_page.len(), 5);

        let last_ts = &first_page.last().expect("has last").created_at;
        let second_page = get_project_activity(&pool, "proj-1", Some(5), Some(last_ts))
            .await
            .expect("second page");
        assert_eq!(second_page.len(), 5);
    }

    #[tokio::test]
    async fn test_get_user_activity() {
        let (pool, _dir) = test_pool().await;

        log_activity(
            &pool,
            "activity-1",
            "proj-1",
            "user-1",
            &ActivityAction::ProjectCreated,
            None,
        )
        .await
        .expect("log");

        let activities = get_user_activity(&pool, "user-1", None, None)
            .await
            .expect("get");
        assert_eq!(activities.len(), 1);

        let none = get_user_activity(&pool, "user-other", None, None)
            .await
            .expect("get");
        assert!(none.is_empty());
    }

    #[tokio::test]
    async fn test_count_project_activity() {
        let (pool, _dir) = test_pool().await;

        for i in 0..3 {
            log_activity(
                &pool,
                &format!("activity-{i}"),
                "proj-1",
                "user-1",
                &ActivityAction::ProjectCreated,
                None,
            )
            .await
            .expect("log");
        }

        let count = count_project_activity(&pool, "proj-1")
            .await
            .expect("count");
        assert_eq!(count, 3);
    }

    #[tokio::test]
    async fn test_delete_old_activity() {
        let (pool, _dir) = test_pool().await;

        sqlx::query(
            "INSERT INTO activity_log (id, project_id, user_id, action, created_at) VALUES ('old-1', 'proj-1', 'user-1', '{}', '2020-01-01T00:00:00Z')",
        )
        .execute(&pool)
        .await
        .expect("insert old");

        log_activity(
            &pool,
            "new-1",
            "proj-1",
            "user-1",
            &ActivityAction::ProjectCreated,
            None,
        )
        .await
        .expect("log new");

        let deleted = delete_old_activity(&pool, "2023-01-01T00:00:00Z")
            .await
            .expect("delete");
        assert_eq!(deleted, 1);

        let remaining = get_project_activity(&pool, "proj-1", None, None)
            .await
            .expect("get");
        assert_eq!(remaining.len(), 1);
    }

    #[tokio::test]
    async fn test_parse_activity_action() {
        let (pool, _dir) = test_pool().await;

        let action = ActivityAction::FileChanged {
            path: "/src/main.rs".to_string(),
            change_type: "modified".to_string(),
        };

        let entry = log_activity(&pool, "activity-1", "proj-1", "user-1", &action, None)
            .await
            .expect("log");

        let parsed = parse_activity_action(&entry).expect("parse");
        match parsed {
            ActivityAction::FileChanged { path, change_type } => {
                assert_eq!(path, "/src/main.rs");
                assert_eq!(change_type, "modified");
            }
            _ => panic!("wrong action type"),
        }
    }

    #[tokio::test]
    async fn test_all_activity_action_types() {
        let (pool, _dir) = test_pool().await;

        let actions = vec![
            ActivityAction::ProjectCreated,
            ActivityAction::MemberAdded {
                member_id: "u1".to_string(),
                role: "member".to_string(),
            },
            ActivityAction::MemberRemoved {
                member_id: "u1".to_string(),
            },
            ActivityAction::RoleChanged {
                member_id: "u1".to_string(),
                old_role: "member".to_string(),
                new_role: "admin".to_string(),
            },
            ActivityAction::AgentTaskStarted {
                task_description: "Build project".to_string(),
            },
            ActivityAction::AgentTaskCompleted {
                task_description: "Build project".to_string(),
                success: true,
            },
            ActivityAction::FileChanged {
                path: "/file.rs".to_string(),
                change_type: "created".to_string(),
            },
            ActivityAction::UserJoined {
                user_id: "u2".to_string(),
            },
            ActivityAction::UserLeft {
                user_id: "u2".to_string(),
            },
            ActivityAction::SettingsChanged {
                setting: "model".to_string(),
            },
            ActivityAction::InviteSent {
                email_masked: "t***@e***.com".to_string(),
                role: "member".to_string(),
            },
        ];

        for (i, action) in actions.iter().enumerate() {
            let entry = log_activity(
                &pool,
                &format!("activity-{i}"),
                "proj-1",
                "user-1",
                action,
                None,
            )
            .await
            .expect("log");

            let parsed = parse_activity_action(&entry);
            assert!(parsed.is_some(), "Failed to parse action {i}");
        }
    }
}
