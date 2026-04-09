//! Message queue persistence for queueing messages while agent is busy.
//!
//! If a user sends multiple messages while the agent is processing,
//! they are queued and processed in order after the current operation completes.

use sqlx::SqlitePool;

/// Create the message_queue table.
pub async fn create_message_queue_table(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS message_queue (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL,
    content TEXT NOT NULL,
    queued_at TEXT NOT NULL,
    processed_at TEXT
)"#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_message_queue_project ON message_queue(project_id, queued_at)",
    )
    .execute(pool)
    .await?;

    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_message_queue_pending ON message_queue(project_id, processed_at)",
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// A queued message record from the database.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct QueuedMessageRecord {
    /// Message ID (UUID).
    pub id: String,
    /// Project ID.
    pub project_id: String,
    /// Message content.
    pub content: String,
    /// When the message was queued (ISO 8601).
    pub queued_at: String,
    /// When the message was processed (ISO 8601), or None if pending.
    pub processed_at: Option<String>,
}

/// Queue a message for later processing.
pub async fn queue_message(
    pool: &SqlitePool,
    id: &str,
    project_id: &str,
    content: &str,
) -> Result<(), sqlx::Error> {
    let now = chrono::Utc::now().to_rfc3339();
    sqlx::query(
        "INSERT INTO message_queue (id, project_id, content, queued_at) VALUES (?, ?, ?, ?)",
    )
    .bind(id)
    .bind(project_id)
    .bind(content)
    .bind(&now)
    .execute(pool)
    .await?;
    Ok(())
}

/// Get the next pending message for a project.
pub async fn get_next_pending(
    pool: &SqlitePool,
    project_id: &str,
) -> Result<Option<QueuedMessageRecord>, sqlx::Error> {
    sqlx::query_as::<_, QueuedMessageRecord>(
        "SELECT * FROM message_queue WHERE project_id = ? AND processed_at IS NULL ORDER BY queued_at ASC LIMIT 1",
    )
    .bind(project_id)
    .fetch_optional(pool)
    .await
}

/// Get all pending messages for a project (in order).
pub async fn get_all_pending(
    pool: &SqlitePool,
    project_id: &str,
) -> Result<Vec<QueuedMessageRecord>, sqlx::Error> {
    sqlx::query_as::<_, QueuedMessageRecord>(
        "SELECT * FROM message_queue WHERE project_id = ? AND processed_at IS NULL ORDER BY queued_at ASC",
    )
    .bind(project_id)
    .fetch_all(pool)
    .await
}

/// Mark a message as processed.
pub async fn mark_processed(pool: &SqlitePool, id: &str) -> Result<bool, sqlx::Error> {
    let now = chrono::Utc::now().to_rfc3339();
    let result = sqlx::query("UPDATE message_queue SET processed_at = ? WHERE id = ?")
        .bind(&now)
        .bind(id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

/// Count pending messages for a project.
pub async fn count_pending(pool: &SqlitePool, project_id: &str) -> Result<i64, sqlx::Error> {
    let row = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM message_queue WHERE project_id = ? AND processed_at IS NULL",
    )
    .bind(project_id)
    .fetch_one(pool)
    .await?;
    Ok(row)
}

/// Clean up old processed messages.
///
/// Deletes messages that were processed more than `days_old` days ago.
pub async fn cleanup_old_messages(pool: &SqlitePool, days_old: i64) -> Result<u64, sqlx::Error> {
    let cutoff = chrono::Utc::now() - chrono::Duration::days(days_old);
    let cutoff_str = cutoff.to_rfc3339();
    let result = sqlx::query(
        "DELETE FROM message_queue WHERE processed_at IS NOT NULL AND processed_at < ?",
    )
    .bind(&cutoff_str)
    .execute(pool)
    .await?;
    Ok(result.rows_affected())
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
        create_message_queue_table(&pool).await.expect("migrate");
        (pool, dir)
    }

    #[tokio::test]
    async fn test_queue_and_get_message() {
        let (pool, _dir) = test_pool().await;

        queue_message(&pool, "msg-1", "project-1", "Hello world")
            .await
            .expect("queue");

        let msg = get_next_pending(&pool, "project-1")
            .await
            .expect("get")
            .expect("exists");

        assert_eq!(msg.id, "msg-1");
        assert_eq!(msg.content, "Hello world");
        assert!(msg.processed_at.is_none());
    }

    #[tokio::test]
    async fn test_queue_order() {
        let (pool, _dir) = test_pool().await;

        queue_message(&pool, "msg-1", "project-1", "First")
            .await
            .expect("queue");
        queue_message(&pool, "msg-2", "project-1", "Second")
            .await
            .expect("queue");
        queue_message(&pool, "msg-3", "project-1", "Third")
            .await
            .expect("queue");

        let pending = get_all_pending(&pool, "project-1").await.expect("get");
        assert_eq!(pending.len(), 3);
        assert_eq!(pending[0].content, "First");
        assert_eq!(pending[1].content, "Second");
        assert_eq!(pending[2].content, "Third");
    }

    #[tokio::test]
    async fn test_mark_processed() {
        let (pool, _dir) = test_pool().await;

        queue_message(&pool, "msg-1", "project-1", "Content")
            .await
            .expect("queue");

        let marked = mark_processed(&pool, "msg-1").await.expect("mark");
        assert!(marked);

        // Should not appear in pending anymore
        let next = get_next_pending(&pool, "project-1").await.expect("get");
        assert!(next.is_none());
    }

    #[tokio::test]
    async fn test_count_pending() {
        let (pool, _dir) = test_pool().await;

        assert_eq!(count_pending(&pool, "project-1").await.expect("count"), 0);

        queue_message(&pool, "msg-1", "project-1", "One")
            .await
            .expect("queue");
        queue_message(&pool, "msg-2", "project-1", "Two")
            .await
            .expect("queue");

        assert_eq!(count_pending(&pool, "project-1").await.expect("count"), 2);

        mark_processed(&pool, "msg-1").await.expect("mark");

        assert_eq!(count_pending(&pool, "project-1").await.expect("count"), 1);
    }
}
