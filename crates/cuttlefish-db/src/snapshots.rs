//! Session snapshots for point-in-time recovery.
//!
//! This module provides persistence for sandbox/code snapshots linked to
//! conversation messages, enabling users to restore to any previous state.

use sqlx::SqlitePool;

/// Type of snapshot (how the state was captured).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SnapshotType {
    /// Docker container snapshot (fast, copy-on-write).
    Docker,
    /// Git commit snapshot.
    Git,
    /// Filesystem tarball snapshot.
    Tarball,
}

impl SnapshotType {
    /// Convert to string for database storage.
    pub fn as_str(&self) -> &'static str {
        match self {
            SnapshotType::Docker => "docker",
            SnapshotType::Git => "git",
            SnapshotType::Tarball => "tarball",
        }
    }

    /// Parse from database string.
    pub fn parse(s: &str) -> Self {
        match s {
            "docker" => SnapshotType::Docker,
            "git" => SnapshotType::Git,
            "tarball" => SnapshotType::Tarball,
            _ => SnapshotType::Git, // Default fallback
        }
    }
}

/// A snapshot of the sandbox/code state at a specific point in conversation.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct SessionSnapshot {
    /// Unique snapshot ID.
    pub id: String,
    /// Project/session this snapshot belongs to.
    pub project_id: String,
    /// The message ID that triggered/preceded this snapshot.
    pub message_id: String,
    /// Message sequence number (for ordering).
    pub message_seq: i64,
    /// Type of snapshot (docker, git, tarball).
    pub snapshot_type: String,
    /// Reference to the snapshot (container ID, commit SHA, tarball path).
    pub snapshot_ref: String,
    /// Optional description/label.
    pub description: Option<String>,
    /// Size of the snapshot in bytes (if known).
    pub size_bytes: Option<i64>,
    /// When the snapshot was created.
    pub created_at: String,
}

/// Create the session_snapshots table.
pub async fn create_snapshots_table(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS session_snapshots (
            id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL,
            message_id TEXT NOT NULL,
            message_seq INTEGER NOT NULL,
            snapshot_type TEXT NOT NULL DEFAULT 'git',
            snapshot_ref TEXT NOT NULL,
            description TEXT,
            size_bytes INTEGER,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            UNIQUE(project_id, message_id)
        )"#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_snapshots_project_seq ON session_snapshots(project_id, message_seq DESC)",
    )
    .execute(pool)
    .await?;

    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_snapshots_message ON session_snapshots(message_id)",
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// Create a new snapshot.
pub async fn create_snapshot(
    pool: &SqlitePool,
    id: &str,
    project_id: &str,
    message_id: &str,
    message_seq: i64,
    snapshot_type: SnapshotType,
    snapshot_ref: &str,
    description: Option<&str>,
    size_bytes: Option<i64>,
) -> Result<SessionSnapshot, sqlx::Error> {
    sqlx::query_as::<_, SessionSnapshot>(
        r#"INSERT INTO session_snapshots
           (id, project_id, message_id, message_seq, snapshot_type, snapshot_ref, description, size_bytes)
           VALUES (?, ?, ?, ?, ?, ?, ?, ?)
           RETURNING *"#,
    )
    .bind(id)
    .bind(project_id)
    .bind(message_id)
    .bind(message_seq)
    .bind(snapshot_type.as_str())
    .bind(snapshot_ref)
    .bind(description)
    .bind(size_bytes)
    .fetch_one(pool)
    .await
}

/// Get a snapshot by message ID.
pub async fn get_snapshot_by_message(
    pool: &SqlitePool,
    message_id: &str,
) -> Result<Option<SessionSnapshot>, sqlx::Error> {
    sqlx::query_as::<_, SessionSnapshot>(
        "SELECT * FROM session_snapshots WHERE message_id = ?",
    )
    .bind(message_id)
    .fetch_optional(pool)
    .await
}

/// Get a snapshot by ID.
pub async fn get_snapshot(
    pool: &SqlitePool,
    id: &str,
) -> Result<Option<SessionSnapshot>, sqlx::Error> {
    sqlx::query_as::<_, SessionSnapshot>("SELECT * FROM session_snapshots WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await
}

/// List all snapshots for a project, ordered by message sequence (newest first).
pub async fn list_project_snapshots(
    pool: &SqlitePool,
    project_id: &str,
) -> Result<Vec<SessionSnapshot>, sqlx::Error> {
    sqlx::query_as::<_, SessionSnapshot>(
        "SELECT * FROM session_snapshots WHERE project_id = ? ORDER BY message_seq DESC",
    )
    .bind(project_id)
    .fetch_all(pool)
    .await
}

/// Get the most recent snapshot for a project.
pub async fn get_latest_snapshot(
    pool: &SqlitePool,
    project_id: &str,
) -> Result<Option<SessionSnapshot>, sqlx::Error> {
    sqlx::query_as::<_, SessionSnapshot>(
        "SELECT * FROM session_snapshots WHERE project_id = ? ORDER BY message_seq DESC LIMIT 1",
    )
    .bind(project_id)
    .fetch_optional(pool)
    .await
}

/// Get snapshot at or before a specific message sequence number.
pub async fn get_snapshot_at_or_before(
    pool: &SqlitePool,
    project_id: &str,
    message_seq: i64,
) -> Result<Option<SessionSnapshot>, sqlx::Error> {
    sqlx::query_as::<_, SessionSnapshot>(
        "SELECT * FROM session_snapshots WHERE project_id = ? AND message_seq <= ? ORDER BY message_seq DESC LIMIT 1",
    )
    .bind(project_id)
    .bind(message_seq)
    .fetch_optional(pool)
    .await
}

/// Delete snapshots older than a specific message sequence.
/// Used for cleanup after restore operations.
pub async fn delete_snapshots_after(
    pool: &SqlitePool,
    project_id: &str,
    message_seq: i64,
) -> Result<u64, sqlx::Error> {
    let result = sqlx::query(
        "DELETE FROM session_snapshots WHERE project_id = ? AND message_seq > ?",
    )
    .bind(project_id)
    .bind(message_seq)
    .execute(pool)
    .await?;
    Ok(result.rows_affected())
}

/// Delete all snapshots for a project.
pub async fn delete_project_snapshots(
    pool: &SqlitePool,
    project_id: &str,
) -> Result<u64, sqlx::Error> {
    let result = sqlx::query("DELETE FROM session_snapshots WHERE project_id = ?")
        .bind(project_id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected())
}

/// Count snapshots for a project.
pub async fn count_project_snapshots(
    pool: &SqlitePool,
    project_id: &str,
) -> Result<i64, sqlx::Error> {
    let row = sqlx::query("SELECT COUNT(*) as count FROM session_snapshots WHERE project_id = ?")
        .bind(project_id)
        .fetch_one(pool)
        .await?;
    Ok(sqlx::Row::get::<i64, _>(&row, "count"))
}

/// Get total size of snapshots for a project.
pub async fn get_project_snapshot_size(
    pool: &SqlitePool,
    project_id: &str,
) -> Result<i64, sqlx::Error> {
    let row = sqlx::query(
        "SELECT COALESCE(SUM(size_bytes), 0) as total FROM session_snapshots WHERE project_id = ?",
    )
    .bind(project_id)
    .fetch_one(pool)
    .await?;
    Ok(sqlx::Row::get::<i64, _>(&row, "total"))
}

/// Session timeline entry combining message and snapshot info.
#[derive(Debug, Clone)]
pub struct TimelineEntry {
    /// Message ID.
    pub message_id: String,
    /// Message sequence number.
    pub message_seq: i64,
    /// Message role (user, assistant, system).
    pub role: String,
    /// Message content (preview).
    pub content_preview: String,
    /// When the message was created.
    pub created_at: String,
    /// Whether this message has an associated snapshot.
    pub has_snapshot: bool,
    /// Snapshot ID if available.
    pub snapshot_id: Option<String>,
    /// Snapshot reference if available.
    pub snapshot_ref: Option<String>,
}

/// Get timeline entries for a project (messages with snapshot status).
pub async fn get_project_timeline(
    pool: &SqlitePool,
    project_id: &str,
    limit: i64,
) -> Result<Vec<TimelineEntry>, sqlx::Error> {
    // Join conversations with snapshots to get timeline
    let rows = sqlx::query(
        r#"SELECT
            c.id as message_id,
            ROW_NUMBER() OVER (ORDER BY c.created_at) as message_seq,
            c.role,
            SUBSTR(c.content, 1, 100) as content_preview,
            c.created_at,
            s.id as snapshot_id,
            s.snapshot_ref
        FROM conversations c
        LEFT JOIN session_snapshots s ON s.message_id = c.id
        WHERE c.project_id = ? AND c.archived = 0
        ORDER BY c.created_at DESC
        LIMIT ?"#,
    )
    .bind(project_id)
    .bind(limit)
    .fetch_all(pool)
    .await?;

    let entries = rows
        .into_iter()
        .map(|row| {
            let snapshot_id: Option<String> = sqlx::Row::get(&row, "snapshot_id");
            let snapshot_ref: Option<String> = sqlx::Row::get(&row, "snapshot_ref");
            TimelineEntry {
                message_id: sqlx::Row::get(&row, "message_id"),
                message_seq: sqlx::Row::get(&row, "message_seq"),
                role: sqlx::Row::get(&row, "role"),
                content_preview: sqlx::Row::get(&row, "content_preview"),
                created_at: sqlx::Row::get(&row, "created_at"),
                has_snapshot: snapshot_id.is_some(),
                snapshot_id,
                snapshot_ref,
            }
        })
        .collect();

    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn test_pool() -> SqlitePool {
        let pool = SqlitePool::connect(":memory:").await.expect("pool");
        create_snapshots_table(&pool).await.expect("migrations");
        pool
    }

    #[tokio::test]
    async fn test_create_and_get_snapshot() {
        let pool = test_pool().await;

        let snapshot = create_snapshot(
            &pool,
            "snap-1",
            "proj-1",
            "msg-1",
            1,
            SnapshotType::Git,
            "abc123",
            Some("Initial state"),
            Some(1024),
        )
        .await
        .expect("create");

        assert_eq!(snapshot.id, "snap-1");
        assert_eq!(snapshot.snapshot_ref, "abc123");

        let found = get_snapshot(&pool, "snap-1")
            .await
            .expect("get")
            .expect("exists");
        assert_eq!(found.project_id, "proj-1");
    }

    #[tokio::test]
    async fn test_get_snapshot_by_message() {
        let pool = test_pool().await;

        create_snapshot(
            &pool,
            "snap-1",
            "proj-1",
            "msg-42",
            1,
            SnapshotType::Docker,
            "container123",
            None,
            None,
        )
        .await
        .expect("create");

        let found = get_snapshot_by_message(&pool, "msg-42")
            .await
            .expect("get")
            .expect("exists");
        assert_eq!(found.snapshot_ref, "container123");
        assert_eq!(found.snapshot_type, "docker");
    }

    #[tokio::test]
    async fn test_list_and_count_snapshots() {
        let pool = test_pool().await;

        for i in 1..=5 {
            create_snapshot(
                &pool,
                &format!("snap-{}", i),
                "proj-1",
                &format!("msg-{}", i),
                i,
                SnapshotType::Git,
                &format!("ref{}", i),
                None,
                Some(100 * i),
            )
            .await
            .expect("create");
        }

        let snapshots = list_project_snapshots(&pool, "proj-1")
            .await
            .expect("list");
        assert_eq!(snapshots.len(), 5);
        // Should be ordered newest first
        assert_eq!(snapshots[0].message_seq, 5);

        let count = count_project_snapshots(&pool, "proj-1")
            .await
            .expect("count");
        assert_eq!(count, 5);

        let size = get_project_snapshot_size(&pool, "proj-1")
            .await
            .expect("size");
        assert_eq!(size, 100 + 200 + 300 + 400 + 500);
    }

    #[tokio::test]
    async fn test_get_latest_and_at_or_before() {
        let pool = test_pool().await;

        for i in 1..=3 {
            create_snapshot(
                &pool,
                &format!("snap-{}", i),
                "proj-1",
                &format!("msg-{}", i),
                i,
                SnapshotType::Git,
                &format!("ref{}", i),
                None,
                None,
            )
            .await
            .expect("create");
        }

        let latest = get_latest_snapshot(&pool, "proj-1")
            .await
            .expect("latest")
            .expect("exists");
        assert_eq!(latest.message_seq, 3);

        let at_2 = get_snapshot_at_or_before(&pool, "proj-1", 2)
            .await
            .expect("at_or_before")
            .expect("exists");
        assert_eq!(at_2.message_seq, 2);
    }

    #[tokio::test]
    async fn test_delete_snapshots_after() {
        let pool = test_pool().await;

        for i in 1..=5 {
            create_snapshot(
                &pool,
                &format!("snap-{}", i),
                "proj-1",
                &format!("msg-{}", i),
                i,
                SnapshotType::Git,
                &format!("ref{}", i),
                None,
                None,
            )
            .await
            .expect("create");
        }

        let deleted = delete_snapshots_after(&pool, "proj-1", 2)
            .await
            .expect("delete");
        assert_eq!(deleted, 3); // Deleted seq 3, 4, 5

        let remaining = list_project_snapshots(&pool, "proj-1")
            .await
            .expect("list");
        assert_eq!(remaining.len(), 2);
    }

    #[tokio::test]
    async fn test_snapshot_type_conversion() {
        assert_eq!(SnapshotType::Docker.as_str(), "docker");
        assert_eq!(SnapshotType::Git.as_str(), "git");
        assert_eq!(SnapshotType::Tarball.as_str(), "tarball");

        assert_eq!(SnapshotType::parse("docker"), SnapshotType::Docker);
        assert_eq!(SnapshotType::parse("git"), SnapshotType::Git);
        assert_eq!(SnapshotType::parse("tarball"), SnapshotType::Tarball);
        assert_eq!(SnapshotType::parse("unknown"), SnapshotType::Git);
    }
}
