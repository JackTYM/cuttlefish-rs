//! Workflow state persistence for tracking active/interrupted workflows.
//!
//! This allows tracking which projects have active workflows and their status,
//! so users can know if a workflow was interrupted by a server restart.

use sqlx::SqlitePool;

/// Create the workflow_state table.
pub async fn create_workflow_state_table(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS workflow_state (
    project_id TEXT PRIMARY KEY,
    status TEXT NOT NULL DEFAULT 'idle',
    current_agent TEXT,
    current_iteration INTEGER DEFAULT 0,
    max_iterations INTEGER DEFAULT 5,
    started_at TEXT,
    paused_at TEXT,
    interrupted_at TEXT,
    last_user_message TEXT,
    context_json TEXT
)"#,
    )
    .execute(pool)
    .await?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_workflow_state_status ON workflow_state(status)")
        .execute(pool)
        .await?;

    Ok(())
}

/// Workflow status values.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkflowStatus {
    /// No workflow running.
    Idle,
    /// Workflow is actively running.
    Running,
    /// Workflow is paused by user.
    Paused,
    /// Workflow is waiting for user approval.
    WaitingApproval,
    /// Workflow was interrupted (server restart).
    Interrupted,
}

impl WorkflowStatus {
    /// Convert to string for storage.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Running => "running",
            Self::Paused => "paused",
            Self::WaitingApproval => "waiting_approval",
            Self::Interrupted => "interrupted",
        }
    }

    /// Parse from database string value.
    pub fn parse(s: &str) -> Self {
        match s {
            "running" => Self::Running,
            "paused" => Self::Paused,
            "waiting_approval" => Self::WaitingApproval,
            "interrupted" => Self::Interrupted,
            _ => Self::Idle,
        }
    }
}

/// A workflow state record from the database.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct WorkflowStateRecord {
    /// Project ID (primary key).
    pub project_id: String,
    /// Current status.
    pub status: String,
    /// Current agent executing.
    pub current_agent: Option<String>,
    /// Current Coder→Critic iteration.
    pub current_iteration: i64,
    /// Maximum iterations allowed.
    pub max_iterations: i64,
    /// When the workflow started.
    pub started_at: Option<String>,
    /// When the workflow was paused.
    pub paused_at: Option<String>,
    /// When the workflow was interrupted (server restart).
    pub interrupted_at: Option<String>,
    /// Last user message that triggered the workflow.
    pub last_user_message: Option<String>,
    /// JSON context for potential resume.
    pub context_json: Option<String>,
}

/// Get workflow state for a project.
pub async fn get_workflow_state(
    pool: &SqlitePool,
    project_id: &str,
) -> Result<Option<WorkflowStateRecord>, sqlx::Error> {
    sqlx::query_as::<_, WorkflowStateRecord>("SELECT * FROM workflow_state WHERE project_id = ?")
        .bind(project_id)
        .fetch_optional(pool)
        .await
}

/// Start a workflow for a project.
pub async fn start_workflow(
    pool: &SqlitePool,
    project_id: &str,
    user_message: &str,
    max_iterations: i64,
) -> Result<(), sqlx::Error> {
    let now = chrono::Utc::now().to_rfc3339();
    sqlx::query(
        r#"INSERT INTO workflow_state (project_id, status, current_iteration, max_iterations, started_at, last_user_message)
        VALUES (?, 'running', 0, ?, ?, ?)
        ON CONFLICT(project_id) DO UPDATE SET
            status = 'running',
            current_iteration = 0,
            max_iterations = excluded.max_iterations,
            started_at = excluded.started_at,
            paused_at = NULL,
            interrupted_at = NULL,
            last_user_message = excluded.last_user_message"#,
    )
    .bind(project_id)
    .bind(max_iterations)
    .bind(&now)
    .bind(user_message)
    .execute(pool)
    .await?;
    Ok(())
}

/// Update workflow progress.
pub async fn update_workflow_progress(
    pool: &SqlitePool,
    project_id: &str,
    current_agent: &str,
    iteration: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE workflow_state SET current_agent = ?, current_iteration = ? WHERE project_id = ?",
    )
    .bind(current_agent)
    .bind(iteration)
    .bind(project_id)
    .execute(pool)
    .await?;
    Ok(())
}

/// Set workflow status.
pub async fn set_workflow_status(
    pool: &SqlitePool,
    project_id: &str,
    status: WorkflowStatus,
) -> Result<(), sqlx::Error> {
    let now = chrono::Utc::now().to_rfc3339();

    match status {
        WorkflowStatus::Paused => {
            sqlx::query(
                "UPDATE workflow_state SET status = 'paused', paused_at = ? WHERE project_id = ?",
            )
            .bind(&now)
            .bind(project_id)
            .execute(pool)
            .await?;
        }
        WorkflowStatus::Interrupted => {
            sqlx::query(
                "UPDATE workflow_state SET status = 'interrupted', interrupted_at = ? WHERE project_id = ?",
            )
            .bind(&now)
            .bind(project_id)
            .execute(pool)
            .await?;
        }
        _ => {
            sqlx::query("UPDATE workflow_state SET status = ? WHERE project_id = ?")
                .bind(status.as_str())
                .bind(project_id)
                .execute(pool)
                .await?;
        }
    }
    Ok(())
}

/// Complete a workflow (set to idle).
pub async fn complete_workflow(pool: &SqlitePool, project_id: &str) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"UPDATE workflow_state SET
            status = 'idle',
            current_agent = NULL,
            started_at = NULL,
            paused_at = NULL,
            interrupted_at = NULL
        WHERE project_id = ?"#,
    )
    .bind(project_id)
    .execute(pool)
    .await?;
    Ok(())
}

/// Get all running workflows (to mark as interrupted on startup).
pub async fn get_running_workflows(
    pool: &SqlitePool,
) -> Result<Vec<WorkflowStateRecord>, sqlx::Error> {
    sqlx::query_as::<_, WorkflowStateRecord>(
        "SELECT * FROM workflow_state WHERE status = 'running'",
    )
    .fetch_all(pool)
    .await
}

/// Get all interrupted workflows (for user notification).
pub async fn get_interrupted_workflows(
    pool: &SqlitePool,
) -> Result<Vec<WorkflowStateRecord>, sqlx::Error> {
    sqlx::query_as::<_, WorkflowStateRecord>(
        "SELECT * FROM workflow_state WHERE status = 'interrupted' ORDER BY interrupted_at DESC",
    )
    .fetch_all(pool)
    .await
}

/// Mark all running workflows as interrupted (call on server startup).
pub async fn mark_running_as_interrupted(pool: &SqlitePool) -> Result<u64, sqlx::Error> {
    let now = chrono::Utc::now().to_rfc3339();
    let result = sqlx::query(
        "UPDATE workflow_state SET status = 'interrupted', interrupted_at = ? WHERE status = 'running'",
    )
    .bind(&now)
    .execute(pool)
    .await?;
    Ok(result.rows_affected())
}

/// Clear interrupted status (user acknowledged or resumed).
pub async fn clear_interrupted(pool: &SqlitePool, project_id: &str) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE workflow_state SET status = 'idle', interrupted_at = NULL WHERE project_id = ? AND status = 'interrupted'")
        .bind(project_id)
        .execute(pool)
        .await?;
    Ok(())
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
        create_workflow_state_table(&pool).await.expect("migrate");
        (pool, dir)
    }

    #[tokio::test]
    async fn test_start_and_get_workflow() {
        let (pool, _dir) = test_pool().await;

        start_workflow(&pool, "project-1", "Create a hello world app", 5)
            .await
            .expect("start");

        let state = get_workflow_state(&pool, "project-1")
            .await
            .expect("get")
            .expect("exists");

        assert_eq!(state.status, "running");
        assert_eq!(state.max_iterations, 5);
        assert_eq!(
            state.last_user_message,
            Some("Create a hello world app".to_string())
        );
    }

    #[tokio::test]
    async fn test_update_progress() {
        let (pool, _dir) = test_pool().await;

        start_workflow(&pool, "project-1", "Task", 5)
            .await
            .expect("start");

        update_workflow_progress(&pool, "project-1", "coder", 2)
            .await
            .expect("update");

        let state = get_workflow_state(&pool, "project-1")
            .await
            .expect("get")
            .expect("exists");

        assert_eq!(state.current_agent, Some("coder".to_string()));
        assert_eq!(state.current_iteration, 2);
    }

    #[tokio::test]
    async fn test_mark_interrupted() {
        let (pool, _dir) = test_pool().await;

        start_workflow(&pool, "project-1", "Task 1", 5)
            .await
            .expect("start");
        start_workflow(&pool, "project-2", "Task 2", 5)
            .await
            .expect("start");

        let count = mark_running_as_interrupted(&pool).await.expect("mark");
        assert_eq!(count, 2);

        let interrupted = get_interrupted_workflows(&pool).await.expect("get");
        assert_eq!(interrupted.len(), 2);

        let state = get_workflow_state(&pool, "project-1")
            .await
            .expect("get")
            .expect("exists");
        assert_eq!(state.status, "interrupted");
    }

    #[tokio::test]
    async fn test_complete_workflow() {
        let (pool, _dir) = test_pool().await;

        start_workflow(&pool, "project-1", "Task", 5)
            .await
            .expect("start");

        complete_workflow(&pool, "project-1")
            .await
            .expect("complete");

        let state = get_workflow_state(&pool, "project-1")
            .await
            .expect("get")
            .expect("exists");

        assert_eq!(state.status, "idle");
        assert!(state.started_at.is_none());
    }
}
