//! Draft prompt persistence for multi-client UX.
//!
//! Allows users to start typing a message on one client (e.g., TUI)
//! and continue on another (e.g., WebUI) without losing their input.

use sqlx::SqlitePool;

/// Create the draft_prompts table.
pub async fn create_draft_prompts_table(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS draft_prompts (
    project_id TEXT PRIMARY KEY,
    content TEXT NOT NULL,
    updated_at TEXT NOT NULL
)"#,
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// A draft prompt record from the database.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct DraftPromptRecord {
    /// Project ID (primary key).
    pub project_id: String,
    /// Draft content.
    pub content: String,
    /// When the draft was last updated (ISO 8601).
    pub updated_at: String,
}

/// Get the draft prompt for a project.
pub async fn get_draft(
    pool: &SqlitePool,
    project_id: &str,
) -> Result<Option<DraftPromptRecord>, sqlx::Error> {
    sqlx::query_as::<_, DraftPromptRecord>("SELECT * FROM draft_prompts WHERE project_id = ?")
        .bind(project_id)
        .fetch_optional(pool)
        .await
}

/// Save or update a draft prompt.
pub async fn save_draft(
    pool: &SqlitePool,
    project_id: &str,
    content: &str,
) -> Result<(), sqlx::Error> {
    let now = chrono::Utc::now().to_rfc3339();
    sqlx::query(
        r#"INSERT INTO draft_prompts (project_id, content, updated_at)
        VALUES (?, ?, ?)
        ON CONFLICT(project_id) DO UPDATE SET
            content = excluded.content,
            updated_at = excluded.updated_at"#,
    )
    .bind(project_id)
    .bind(content)
    .bind(&now)
    .execute(pool)
    .await?;
    Ok(())
}

/// Clear a draft prompt (e.g., after sending the message).
pub async fn clear_draft(pool: &SqlitePool, project_id: &str) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM draft_prompts WHERE project_id = ?")
        .bind(project_id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

/// Get all draft prompts (for debugging/admin).
pub async fn get_all_drafts(pool: &SqlitePool) -> Result<Vec<DraftPromptRecord>, sqlx::Error> {
    sqlx::query_as::<_, DraftPromptRecord>(
        "SELECT * FROM draft_prompts ORDER BY updated_at DESC",
    )
    .fetch_all(pool)
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    async fn test_pool() -> (SqlitePool, TempDir) {
        let dir = TempDir::new().expect("temp dir");
        let db_path = dir.path().join("test.db");
        let url = format!("sqlite://{}?mode=rwc", db_path.to_string_lossy());
        let pool = SqlitePool::connect(&url).await.expect("connect");
        create_draft_prompts_table(&pool).await.expect("migrate");
        (pool, dir)
    }

    #[tokio::test]
    async fn test_save_and_get_draft() {
        let (pool, _dir) = test_pool().await;

        save_draft(&pool, "project-1", "Hello, this is my partial message")
            .await
            .expect("save");

        let draft = get_draft(&pool, "project-1")
            .await
            .expect("get")
            .expect("exists");

        assert_eq!(draft.project_id, "project-1");
        assert_eq!(draft.content, "Hello, this is my partial message");
    }

    #[tokio::test]
    async fn test_update_draft() {
        let (pool, _dir) = test_pool().await;

        save_draft(&pool, "project-1", "First version")
            .await
            .expect("save");

        save_draft(&pool, "project-1", "Updated version")
            .await
            .expect("update");

        let draft = get_draft(&pool, "project-1")
            .await
            .expect("get")
            .expect("exists");

        assert_eq!(draft.content, "Updated version");
    }

    #[tokio::test]
    async fn test_clear_draft() {
        let (pool, _dir) = test_pool().await;

        save_draft(&pool, "project-1", "Content")
            .await
            .expect("save");

        let cleared = clear_draft(&pool, "project-1").await.expect("clear");
        assert!(cleared);

        let draft = get_draft(&pool, "project-1").await.expect("get");
        assert!(draft.is_none());
    }

    #[tokio::test]
    async fn test_no_draft() {
        let (pool, _dir) = test_pool().await;

        let draft = get_draft(&pool, "nonexistent").await.expect("get");
        assert!(draft.is_none());
    }
}
