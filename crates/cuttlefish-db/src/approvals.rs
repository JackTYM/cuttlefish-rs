//! Pending approval persistence for safety workflow.
//!
//! This module provides database operations for persisting pending approvals
//! so they survive server restarts.

use sqlx::SqlitePool;

/// Create the pending_approvals table.
pub async fn create_pending_approvals_table(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS pending_approvals (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL,
    action_type TEXT NOT NULL,
    description TEXT NOT NULL,
    path TEXT,
    command TEXT,
    confidence REAL NOT NULL,
    risk_factors TEXT,
    diff_preview TEXT,
    timeout_secs INTEGER NOT NULL,
    created_at TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending',
    resolved_at TEXT,
    resolved_by TEXT,
    rejection_reason TEXT
)"#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_pending_approvals_project ON pending_approvals(project_id, status)",
    )
    .execute(pool)
    .await?;

    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_pending_approvals_status ON pending_approvals(status, created_at)",
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// A pending approval record from the database.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct PendingApprovalRecord {
    /// Unique action ID.
    pub id: String,
    /// Project ID this action belongs to.
    pub project_id: String,
    /// Type of action (FileWrite, BashCommand, etc.).
    pub action_type: String,
    /// Human-readable description.
    pub description: String,
    /// File path if applicable.
    pub path: Option<String>,
    /// Command if applicable.
    pub command: Option<String>,
    /// Confidence score (0.0 - 1.0).
    pub confidence: f64,
    /// JSON-encoded risk factors.
    pub risk_factors: Option<String>,
    /// Unified diff preview.
    pub diff_preview: Option<String>,
    /// Timeout in seconds.
    pub timeout_secs: i64,
    /// When the approval was created (ISO 8601).
    pub created_at: String,
    /// Status: pending, approved, rejected, expired.
    pub status: String,
    /// When resolved (ISO 8601).
    pub resolved_at: Option<String>,
    /// User who resolved.
    pub resolved_by: Option<String>,
    /// Reason for rejection.
    pub rejection_reason: Option<String>,
}

/// Insert a new pending approval.
#[allow(clippy::too_many_arguments)]
pub async fn insert_pending_approval(
    pool: &SqlitePool,
    id: &str,
    project_id: &str,
    action_type: &str,
    description: &str,
    path: Option<&str>,
    command: Option<&str>,
    confidence: f64,
    risk_factors: Option<&str>,
    diff_preview: Option<&str>,
    timeout_secs: i64,
    created_at: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"INSERT INTO pending_approvals
        (id, project_id, action_type, description, path, command, confidence,
         risk_factors, diff_preview, timeout_secs, created_at, status)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 'pending')"#,
    )
    .bind(id)
    .bind(project_id)
    .bind(action_type)
    .bind(description)
    .bind(path)
    .bind(command)
    .bind(confidence)
    .bind(risk_factors)
    .bind(diff_preview)
    .bind(timeout_secs)
    .bind(created_at)
    .execute(pool)
    .await?;
    Ok(())
}

/// Get a pending approval by ID.
pub async fn get_pending_approval(
    pool: &SqlitePool,
    id: &str,
) -> Result<Option<PendingApprovalRecord>, sqlx::Error> {
    sqlx::query_as::<_, PendingApprovalRecord>("SELECT * FROM pending_approvals WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await
}

/// Get all pending approvals (status = 'pending').
pub async fn get_all_pending(pool: &SqlitePool) -> Result<Vec<PendingApprovalRecord>, sqlx::Error> {
    sqlx::query_as::<_, PendingApprovalRecord>(
        "SELECT * FROM pending_approvals WHERE status = 'pending' ORDER BY created_at ASC",
    )
    .fetch_all(pool)
    .await
}

/// Get pending approvals for a specific project.
pub async fn get_pending_for_project(
    pool: &SqlitePool,
    project_id: &str,
) -> Result<Vec<PendingApprovalRecord>, sqlx::Error> {
    sqlx::query_as::<_, PendingApprovalRecord>(
        "SELECT * FROM pending_approvals WHERE project_id = ? AND status = 'pending' ORDER BY created_at ASC",
    )
    .bind(project_id)
    .fetch_all(pool)
    .await
}

/// Mark an approval as approved.
pub async fn approve_approval(
    pool: &SqlitePool,
    id: &str,
    resolved_by: Option<&str>,
) -> Result<bool, sqlx::Error> {
    let now = chrono::Utc::now().to_rfc3339();
    let result = sqlx::query(
        "UPDATE pending_approvals SET status = 'approved', resolved_at = ?, resolved_by = ? WHERE id = ? AND status = 'pending'",
    )
    .bind(&now)
    .bind(resolved_by)
    .bind(id)
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}

/// Mark an approval as rejected.
pub async fn reject_approval(
    pool: &SqlitePool,
    id: &str,
    resolved_by: Option<&str>,
    reason: Option<&str>,
) -> Result<bool, sqlx::Error> {
    let now = chrono::Utc::now().to_rfc3339();
    let result = sqlx::query(
        "UPDATE pending_approvals SET status = 'rejected', resolved_at = ?, resolved_by = ?, rejection_reason = ? WHERE id = ? AND status = 'pending'",
    )
    .bind(&now)
    .bind(resolved_by)
    .bind(reason)
    .bind(id)
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}

/// Mark expired approvals based on their timeout.
///
/// Returns the number of approvals marked as expired.
pub async fn expire_timed_out_approvals(pool: &SqlitePool) -> Result<u64, sqlx::Error> {
    let now = chrono::Utc::now().to_rfc3339();
    // Mark as expired if created_at + timeout_secs < now
    let result = sqlx::query(
        r#"UPDATE pending_approvals
        SET status = 'expired', resolved_at = ?
        WHERE status = 'pending'
        AND datetime(created_at, '+' || timeout_secs || ' seconds') < datetime(?)"#,
    )
    .bind(&now)
    .bind(&now)
    .execute(pool)
    .await?;
    Ok(result.rows_affected())
}

/// Delete old resolved approvals (cleanup).
///
/// Deletes approvals that were resolved more than `days_old` days ago.
pub async fn cleanup_old_approvals(pool: &SqlitePool, days_old: i64) -> Result<u64, sqlx::Error> {
    let cutoff = chrono::Utc::now() - chrono::Duration::days(days_old);
    let cutoff_str = cutoff.to_rfc3339();
    let result =
        sqlx::query("DELETE FROM pending_approvals WHERE status != 'pending' AND resolved_at < ?")
            .bind(&cutoff_str)
            .execute(pool)
            .await?;
    Ok(result.rows_affected())
}

/// Count pending approvals.
pub async fn count_pending(pool: &SqlitePool) -> Result<i64, sqlx::Error> {
    let row = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM pending_approvals WHERE status = 'pending'",
    )
    .fetch_one(pool)
    .await?;
    Ok(row)
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
        create_pending_approvals_table(&pool)
            .await
            .expect("migrate");
        (pool, dir)
    }

    #[tokio::test]
    async fn test_insert_and_get_approval() {
        let (pool, _dir) = test_pool().await;
        let now = chrono::Utc::now().to_rfc3339();

        insert_pending_approval(
            &pool,
            "action-1",
            "project-1",
            "FileWrite",
            "Write to main.rs",
            Some("src/main.rs"),
            None,
            0.75,
            Some(r#"["modifies_code"]"#),
            Some("+ new line"),
            300,
            &now,
        )
        .await
        .expect("insert");

        let record = get_pending_approval(&pool, "action-1")
            .await
            .expect("get")
            .expect("exists");

        assert_eq!(record.project_id, "project-1");
        assert_eq!(record.action_type, "FileWrite");
        assert_eq!(record.status, "pending");
        assert_eq!(record.path, Some("src/main.rs".to_string()));
    }

    #[tokio::test]
    async fn test_approve_approval() {
        let (pool, _dir) = test_pool().await;
        let now = chrono::Utc::now().to_rfc3339();

        insert_pending_approval(
            &pool,
            "action-2",
            "project-1",
            "BashCommand",
            "Run tests",
            None,
            Some("cargo test"),
            0.8,
            None,
            None,
            300,
            &now,
        )
        .await
        .expect("insert");

        let approved = approve_approval(&pool, "action-2", Some("user-1"))
            .await
            .expect("approve");
        assert!(approved);

        let record = get_pending_approval(&pool, "action-2")
            .await
            .expect("get")
            .expect("exists");
        assert_eq!(record.status, "approved");
        assert_eq!(record.resolved_by, Some("user-1".to_string()));
    }

    #[tokio::test]
    async fn test_reject_approval() {
        let (pool, _dir) = test_pool().await;
        let now = chrono::Utc::now().to_rfc3339();

        insert_pending_approval(
            &pool,
            "action-3",
            "project-1",
            "FileWrite",
            "Delete important file",
            Some("/etc/passwd"),
            None,
            0.2,
            None,
            None,
            300,
            &now,
        )
        .await
        .expect("insert");

        let rejected = reject_approval(&pool, "action-3", Some("user-1"), Some("Too risky"))
            .await
            .expect("reject");
        assert!(rejected);

        let record = get_pending_approval(&pool, "action-3")
            .await
            .expect("get")
            .expect("exists");
        assert_eq!(record.status, "rejected");
        assert_eq!(record.rejection_reason, Some("Too risky".to_string()));
    }

    #[tokio::test]
    async fn test_get_all_pending() {
        let (pool, _dir) = test_pool().await;
        let now = chrono::Utc::now().to_rfc3339();

        // Insert 3 pending
        for i in 1..=3 {
            insert_pending_approval(
                &pool,
                &format!("action-{i}"),
                "project-1",
                "FileWrite",
                &format!("Action {i}"),
                None,
                None,
                0.5,
                None,
                None,
                300,
                &now,
            )
            .await
            .expect("insert");
        }

        // Approve one
        approve_approval(&pool, "action-2", None)
            .await
            .expect("approve");

        let pending = get_all_pending(&pool).await.expect("get all");
        assert_eq!(pending.len(), 2);
    }

    #[tokio::test]
    async fn test_count_pending() {
        let (pool, _dir) = test_pool().await;
        let now = chrono::Utc::now().to_rfc3339();

        assert_eq!(count_pending(&pool).await.expect("count"), 0);

        insert_pending_approval(
            &pool,
            "action-1",
            "project-1",
            "FileWrite",
            "Test",
            None,
            None,
            0.5,
            None,
            None,
            300,
            &now,
        )
        .await
        .expect("insert");

        assert_eq!(count_pending(&pool).await.expect("count"), 1);
    }
}
