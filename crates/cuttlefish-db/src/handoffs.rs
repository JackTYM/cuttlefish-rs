//! Async handoff system for collaboration.
//!
//! Enables users to pass work to others with full context preserved,
//! supporting async collaboration workflows.

use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

use crate::activity::log_activity;
use crate::models::{ActivityAction, ProjectRole};
use crate::sharing::can_user_access;

/// Status of a handoff.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HandoffStatus {
    /// Waiting for acceptance.
    Pending,
    /// Accepted by the assignee.
    Accepted,
    /// Rejected by the assignee.
    Rejected,
    /// Expired without action.
    Expired,
}

impl HandoffStatus {
    /// Convert status to string representation.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Accepted => "accepted",
            Self::Rejected => "rejected",
            Self::Expired => "expired",
        }
    }

    /// Parse status from string.
    pub fn parse(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "pending" => Self::Pending,
            "accepted" => Self::Accepted,
            "rejected" => Self::Rejected,
            "expired" => Self::Expired,
            _ => Self::Pending,
        }
    }
}

/// Priority level for handoffs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HandoffPriority {
    /// Low priority.
    Low,
    /// Normal priority.
    Normal,
    /// High priority.
    High,
    /// Urgent priority.
    Urgent,
}

impl HandoffPriority {
    /// Convert priority to string representation.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Normal => "normal",
            Self::High => "high",
            Self::Urgent => "urgent",
        }
    }

    /// Parse priority from string.
    pub fn parse(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "low" => Self::Low,
            "normal" => Self::Normal,
            "high" => Self::High,
            "urgent" => Self::Urgent,
            _ => Self::Normal,
        }
    }
}

/// Context snapshot for handoff.
///
/// Contains all the information needed to restore context when
/// accepting a handoff.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextSnapshot {
    /// Summary of the conversation so far.
    pub conversation_summary: String,
    /// Recent messages (last 20 max).
    pub recent_messages: Vec<MessageSnapshot>,
    /// Current memory state (serialized).
    pub memory_state: Option<String>,
    /// Current git branch.
    pub current_branch: Option<String>,
    /// List of open files.
    pub open_files: Vec<String>,
    /// Open questions to address.
    pub open_questions: Vec<String>,
    /// Suggested next steps.
    pub suggested_next_steps: Vec<String>,
}

impl ContextSnapshot {
    /// Create a new empty context snapshot.
    pub fn new(summary: impl Into<String>) -> Self {
        Self {
            conversation_summary: summary.into(),
            recent_messages: Vec::new(),
            memory_state: None,
            current_branch: None,
            open_files: Vec::new(),
            open_questions: Vec::new(),
            suggested_next_steps: Vec::new(),
        }
    }

    /// Serialize to JSON string.
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| "{}".to_string())
    }

    /// Deserialize from JSON string.
    pub fn from_json(s: &str) -> Option<Self> {
        serde_json::from_str(s).ok()
    }
}

/// A message snapshot for context.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageSnapshot {
    /// Role of the message sender.
    pub role: String,
    /// Message content (truncated if too long).
    pub content: String,
    /// Timestamp of the message.
    pub timestamp: String,
}

/// A handoff record.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Handoff {
    /// Unique handoff identifier.
    pub id: String,
    /// Project ID this handoff belongs to.
    pub project_id: String,
    /// User ID who created the handoff.
    pub from_user_id: String,
    /// User ID assigned to handle (None = anyone on project).
    pub to_user_id: Option<String>,
    /// Handoff title.
    pub title: String,
    /// Handoff message/description.
    pub message: Option<String>,
    /// Serialized context snapshot (JSON).
    pub context_snapshot: String,
    /// Priority level.
    pub priority: String,
    /// Current status.
    pub status: String,
    /// When the handoff was created.
    pub created_at: String,
    /// When the handoff was accepted.
    pub accepted_at: Option<String>,
    /// User ID who accepted the handoff.
    pub accepted_by: Option<String>,
    /// Rejection reason if rejected.
    pub rejection_reason: Option<String>,
}

impl Handoff {
    /// Get the parsed status.
    pub fn status(&self) -> HandoffStatus {
        HandoffStatus::parse(&self.status)
    }

    /// Get the parsed priority.
    pub fn priority(&self) -> HandoffPriority {
        HandoffPriority::parse(&self.priority)
    }

    /// Get the parsed context snapshot.
    pub fn context(&self) -> Option<ContextSnapshot> {
        ContextSnapshot::from_json(&self.context_snapshot)
    }

    /// Check if the handoff is pending.
    pub fn is_pending(&self) -> bool {
        self.status() == HandoffStatus::Pending
    }
}

/// Summary of a handoff for list endpoints.
///
/// Does not include full context to protect privacy and reduce payload size.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandoffSummary {
    /// Unique handoff identifier.
    pub id: String,
    /// Project ID.
    pub project_id: String,
    /// User ID who created the handoff.
    pub from_user_id: String,
    /// User ID assigned to handle.
    pub to_user_id: Option<String>,
    /// Handoff title.
    pub title: String,
    /// Handoff message (truncated).
    pub message_preview: Option<String>,
    /// Priority level.
    pub priority: String,
    /// Current status.
    pub status: String,
    /// When the handoff was created.
    pub created_at: String,
    /// Brief summary from context (first 200 chars).
    pub context_summary: String,
}

impl From<Handoff> for HandoffSummary {
    fn from(h: Handoff) -> Self {
        let context_summary = h
            .context()
            .map(|c| {
                let s = &c.conversation_summary;
                if s.len() > 200 {
                    format!("{}...", &s[..197])
                } else {
                    s.clone()
                }
            })
            .unwrap_or_default();

        let message_preview = h.message.map(|m| {
            if m.len() > 100 {
                format!("{}...", &m[..97])
            } else {
                m
            }
        });

        Self {
            id: h.id,
            project_id: h.project_id,
            from_user_id: h.from_user_id,
            to_user_id: h.to_user_id,
            title: h.title,
            message_preview,
            priority: h.priority,
            status: h.status,
            created_at: h.created_at,
            context_summary,
        }
    }
}

/// Notification payload for handoff events.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum HandoffNotification {
    /// A handoff was assigned to a user.
    Assigned {
        /// Handoff ID.
        handoff_id: String,
        /// Project ID.
        project_id: String,
        /// Project name (for display).
        project_name: Option<String>,
        /// Title of the handoff.
        title: String,
        /// User who created the handoff.
        from_user_id: String,
        /// Priority level.
        priority: String,
    },
    /// A handoff was accepted.
    Accepted {
        /// Handoff ID.
        handoff_id: String,
        /// Project ID.
        project_id: String,
        /// User who accepted.
        accepted_by: String,
    },
    /// A handoff was rejected.
    Rejected {
        /// Handoff ID.
        handoff_id: String,
        /// Project ID.
        project_id: String,
        /// User who rejected.
        rejected_by: String,
        /// Rejection reason.
        reason: Option<String>,
    },
    /// A handoff expired.
    Expired {
        /// Handoff ID.
        handoff_id: String,
        /// Project ID.
        project_id: String,
    },
}

/// Trait for notification delivery.
///
/// Implement this trait to deliver handoff notifications via
/// different channels (Discord, email, WebSocket, etc.).
#[allow(async_fn_in_trait)]
pub trait HandoffNotifier: Send + Sync {
    /// Send a notification to a user.
    async fn notify(&self, user_id: &str, notification: HandoffNotification)
        -> Result<(), String>;
}

/// Error types for handoff operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HandoffError {
    /// Cannot create handoff to yourself.
    SelfHandoff,
    /// User does not have access to the project.
    NoProjectAccess,
    /// Handoff not found.
    NotFound,
    /// Handoff is not in pending status.
    NotPending,
    /// User is not the assignee.
    NotAssignee,
    /// Database error.
    Database(String),
}

impl std::fmt::Display for HandoffError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SelfHandoff => write!(f, "cannot create handoff to yourself"),
            Self::NoProjectAccess => write!(f, "user does not have project access"),
            Self::NotFound => write!(f, "handoff not found"),
            Self::NotPending => write!(f, "handoff is not pending"),
            Self::NotAssignee => write!(f, "user is not the assignee"),
            Self::Database(e) => write!(f, "database error: {e}"),
        }
    }
}

impl std::error::Error for HandoffError {}

impl From<sqlx::Error> for HandoffError {
    fn from(e: sqlx::Error) -> Self {
        Self::Database(e.to_string())
    }
}

/// Create the handoffs table and indexes.
pub async fn create_handoffs_table(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS handoffs (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL,
    from_user_id TEXT NOT NULL,
    to_user_id TEXT,
    title TEXT NOT NULL,
    message TEXT,
    context_snapshot TEXT NOT NULL,
    priority TEXT NOT NULL DEFAULT 'normal',
    status TEXT NOT NULL DEFAULT 'pending',
    created_at TEXT NOT NULL,
    accepted_at TEXT,
    accepted_by TEXT,
    rejection_reason TEXT,
    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE,
    FOREIGN KEY (from_user_id) REFERENCES users(id) ON DELETE SET NULL,
    FOREIGN KEY (to_user_id) REFERENCES users(id) ON DELETE SET NULL,
    FOREIGN KEY (accepted_by) REFERENCES users(id) ON DELETE SET NULL
)"#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_handoffs_project ON handoffs(project_id, status)",
    )
    .execute(pool)
    .await?;

    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_handoffs_to_user ON handoffs(to_user_id, status)",
    )
    .execute(pool)
    .await?;

    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_handoffs_from_user ON handoffs(from_user_id, created_at DESC)",
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// Request to create a new handoff.
#[derive(Debug, Clone)]
pub struct CreateHandoffRequest<'a> {
    /// Unique handoff identifier.
    pub id: &'a str,
    /// Project ID.
    pub project_id: &'a str,
    /// User creating the handoff.
    pub from_user_id: &'a str,
    /// User assigned to handle (None = open to anyone).
    pub to_user_id: Option<&'a str>,
    /// Handoff title.
    pub title: &'a str,
    /// Optional message.
    pub message: Option<&'a str>,
    /// Context snapshot.
    pub context: &'a ContextSnapshot,
    /// Priority level.
    pub priority: HandoffPriority,
}

/// Create a new handoff.
///
/// # Errors
///
/// Returns `HandoffError::SelfHandoff` if `from_user_id` equals `to_user_id`.
/// Returns `HandoffError::NoProjectAccess` if either user lacks project access.
pub async fn create_handoff(
    pool: &SqlitePool,
    request: CreateHandoffRequest<'_>,
) -> Result<Handoff, HandoffError> {
    if let Some(to_id) = request.to_user_id
        && request.from_user_id == to_id
    {
        return Err(HandoffError::SelfHandoff);
    }

    let from_has_access =
        can_user_access(pool, request.from_user_id, request.project_id, ProjectRole::Member)
            .await
            .map_err(HandoffError::from)?;
    if !from_has_access {
        return Err(HandoffError::NoProjectAccess);
    }

    if let Some(to_id) = request.to_user_id {
        let to_has_access = can_user_access(pool, to_id, request.project_id, ProjectRole::Viewer)
            .await
            .map_err(HandoffError::from)?;
        if !to_has_access {
            return Err(HandoffError::NoProjectAccess);
        }
    }

    let now = Utc::now().to_rfc3339();
    let context_json = request.context.to_json();

    let handoff = sqlx::query_as::<_, Handoff>(
        r#"INSERT INTO handoffs (id, project_id, from_user_id, to_user_id, title, message, context_snapshot, priority, status, created_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, 'pending', ?)
        RETURNING *"#,
    )
    .bind(request.id)
    .bind(request.project_id)
    .bind(request.from_user_id)
    .bind(request.to_user_id)
    .bind(request.title)
    .bind(request.message)
    .bind(&context_json)
    .bind(request.priority.as_str())
    .bind(&now)
    .fetch_one(pool)
    .await?;

    let activity_id = uuid::Uuid::new_v4().to_string();
    let _ = log_activity(
        pool,
        &activity_id,
        request.project_id,
        request.from_user_id,
        &ActivityAction::HandoffCreated {
            to_user_id: request.to_user_id.map(String::from),
        },
        Some(&format!("Handoff: {}", request.title)),
    )
    .await;

    Ok(handoff)
}

/// Accept a handoff and return the full context.
///
/// # Errors
///
/// Returns `HandoffError::NotFound` if the handoff doesn't exist.
/// Returns `HandoffError::NotPending` if the handoff is not pending.
/// Returns `HandoffError::NotAssignee` if the user is not the assignee.
pub async fn accept_handoff(
    pool: &SqlitePool,
    handoff_id: &str,
    user_id: &str,
) -> Result<ContextSnapshot, HandoffError> {
    let handoff = get_handoff(pool, handoff_id).await?.ok_or(HandoffError::NotFound)?;

    if !handoff.is_pending() {
        return Err(HandoffError::NotPending);
    }

    // Check if user is the assignee (or handoff is open to anyone)
    if let Some(ref to_id) = handoff.to_user_id {
        if to_id != user_id {
            return Err(HandoffError::NotAssignee);
        }
    } else {
        // Open handoff - verify user has project access
        let has_access = can_user_access(pool, user_id, &handoff.project_id, ProjectRole::Member)
            .await
            .map_err(HandoffError::from)?;
        if !has_access {
            return Err(HandoffError::NoProjectAccess);
        }
    }

    let now = Utc::now().to_rfc3339();

    sqlx::query(
        "UPDATE handoffs SET status = 'accepted', accepted_at = ?, accepted_by = ? WHERE id = ?",
    )
    .bind(&now)
    .bind(user_id)
    .bind(handoff_id)
    .execute(pool)
    .await?;

    // Log activity
    let activity_id = uuid::Uuid::new_v4().to_string();
    let _ = log_activity(
        pool,
        &activity_id,
        &handoff.project_id,
        user_id,
        &ActivityAction::HandoffAccepted {
            by_user_id: user_id.to_string(),
        },
        Some(&format!("Accepted handoff: {}", handoff.title)),
    )
    .await;

    handoff.context().ok_or(HandoffError::NotFound)
}

/// Reject a handoff with an optional reason.
///
/// # Errors
///
/// Returns `HandoffError::NotFound` if the handoff doesn't exist.
/// Returns `HandoffError::NotPending` if the handoff is not pending.
/// Returns `HandoffError::NotAssignee` if the user is not the assignee.
pub async fn reject_handoff(
    pool: &SqlitePool,
    handoff_id: &str,
    user_id: &str,
    reason: Option<&str>,
) -> Result<(), HandoffError> {
    let handoff = get_handoff(pool, handoff_id).await?.ok_or(HandoffError::NotFound)?;

    if !handoff.is_pending() {
        return Err(HandoffError::NotPending);
    }

    // Check if user is the assignee
    if let Some(ref to_id) = handoff.to_user_id {
        if to_id != user_id {
            return Err(HandoffError::NotAssignee);
        }
    } else {
        // Open handoff - verify user has project access
        let has_access = can_user_access(pool, user_id, &handoff.project_id, ProjectRole::Member)
            .await
            .map_err(HandoffError::from)?;
        if !has_access {
            return Err(HandoffError::NoProjectAccess);
        }
    }

    sqlx::query("UPDATE handoffs SET status = 'rejected', rejection_reason = ? WHERE id = ?")
        .bind(reason)
        .bind(handoff_id)
        .execute(pool)
        .await?;

    Ok(())
}

/// Get a handoff by ID.
pub async fn get_handoff(
    pool: &SqlitePool,
    handoff_id: &str,
) -> Result<Option<Handoff>, sqlx::Error> {
    sqlx::query_as::<_, Handoff>("SELECT * FROM handoffs WHERE id = ?")
        .bind(handoff_id)
        .fetch_optional(pool)
        .await
}

/// Get pending handoffs for a user.
///
/// Returns summaries only (no full context) for privacy and performance.
/// Includes both directly assigned handoffs and open handoffs for projects
/// the user has access to.
pub async fn get_pending_handoffs(
    pool: &SqlitePool,
    user_id: &str,
) -> Result<Vec<HandoffSummary>, sqlx::Error> {
    // Get directly assigned handoffs
    let assigned: Vec<Handoff> = sqlx::query_as::<_, Handoff>(
        "SELECT * FROM handoffs WHERE to_user_id = ? AND status = 'pending' ORDER BY created_at DESC",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    // Get open handoffs for projects user has access to
    let open: Vec<Handoff> = sqlx::query_as::<_, Handoff>(
        r#"SELECT h.* FROM handoffs h
        INNER JOIN project_shares ps ON h.project_id = ps.project_id
        WHERE ps.user_id = ? AND h.to_user_id IS NULL AND h.status = 'pending'
        AND h.from_user_id != ?
        ORDER BY h.created_at DESC"#,
    )
    .bind(user_id)
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    let mut summaries: Vec<HandoffSummary> = assigned.into_iter().map(HandoffSummary::from).collect();
    summaries.extend(open.into_iter().map(HandoffSummary::from));

    // Sort by created_at descending
    summaries.sort_by(|a, b| b.created_at.cmp(&a.created_at));

    Ok(summaries)
}

/// Get handoffs created by a user.
pub async fn get_created_handoffs(
    pool: &SqlitePool,
    user_id: &str,
    limit: Option<i64>,
) -> Result<Vec<HandoffSummary>, sqlx::Error> {
    let limit = limit.unwrap_or(50);

    let handoffs: Vec<Handoff> = sqlx::query_as::<_, Handoff>(
        "SELECT * FROM handoffs WHERE from_user_id = ? ORDER BY created_at DESC LIMIT ?",
    )
    .bind(user_id)
    .bind(limit)
    .fetch_all(pool)
    .await?;

    Ok(handoffs.into_iter().map(HandoffSummary::from).collect())
}

/// Get handoffs for a project.
pub async fn get_project_handoffs(
    pool: &SqlitePool,
    project_id: &str,
    status: Option<HandoffStatus>,
    limit: Option<i64>,
) -> Result<Vec<HandoffSummary>, sqlx::Error> {
    let limit = limit.unwrap_or(50);

    let handoffs: Vec<Handoff> = if let Some(s) = status {
        sqlx::query_as::<_, Handoff>(
            "SELECT * FROM handoffs WHERE project_id = ? AND status = ? ORDER BY created_at DESC LIMIT ?",
        )
        .bind(project_id)
        .bind(s.as_str())
        .bind(limit)
        .fetch_all(pool)
        .await?
    } else {
        sqlx::query_as::<_, Handoff>(
            "SELECT * FROM handoffs WHERE project_id = ? ORDER BY created_at DESC LIMIT ?",
        )
        .bind(project_id)
        .bind(limit)
        .fetch_all(pool)
        .await?
    };

    Ok(handoffs.into_iter().map(HandoffSummary::from).collect())
}

/// Expire old pending handoffs.
///
/// Returns the number of handoffs expired.
pub async fn expire_old_handoffs(
    pool: &SqlitePool,
    max_age_days: i64,
) -> Result<u64, sqlx::Error> {
    let cutoff = Utc::now() - Duration::days(max_age_days);
    let cutoff_str = cutoff.to_rfc3339();

    let result = sqlx::query(
        "UPDATE handoffs SET status = 'expired' WHERE status = 'pending' AND created_at < ?",
    )
    .bind(&cutoff_str)
    .execute(pool)
    .await?;

    Ok(result.rows_affected())
}

/// Count pending handoffs for a user.
pub async fn count_pending_handoffs(
    pool: &SqlitePool,
    user_id: &str,
) -> Result<i64, sqlx::Error> {
    // Count directly assigned
    let assigned: i64 = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM handoffs WHERE to_user_id = ? AND status = 'pending'",
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    // Count open handoffs for accessible projects
    let open: i64 = sqlx::query_scalar::<_, i64>(
        r#"SELECT COUNT(*) FROM handoffs h
        INNER JOIN project_shares ps ON h.project_id = ps.project_id
        WHERE ps.user_id = ? AND h.to_user_id IS NULL AND h.status = 'pending'
        AND h.from_user_id != ?"#,
    )
    .bind(user_id)
    .bind(user_id)
    .fetch_all(pool)
    .await?
    .first()
    .copied()
    .unwrap_or(0);

    Ok(assigned + open)
}

/// Activity action types for handoffs (extends ActivityAction).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum HandoffActivityAction {
    /// A handoff was created.
    HandoffCreated {
        /// User ID assigned (None = open).
        to_user_id: Option<String>,
    },
    /// A handoff was accepted.
    HandoffAccepted {
        /// User ID who accepted.
        by_user_id: String,
    },
    /// A handoff was rejected.
    HandoffRejected {
        /// User ID who rejected.
        by_user_id: String,
        /// Rejection reason.
        reason: Option<String>,
    },
    /// A handoff expired.
    HandoffExpired,
}

// Extend ActivityAction with handoff-specific actions
impl From<HandoffActivityAction> for ActivityAction {
    fn from(action: HandoffActivityAction) -> Self {
        match action {
            HandoffActivityAction::HandoffCreated { to_user_id } => {
                ActivityAction::HandoffCreated { to_user_id }
            }
            HandoffActivityAction::HandoffAccepted { by_user_id } => {
                ActivityAction::HandoffAccepted { by_user_id }
            }
            // Map rejected and expired to generic actions for now
            HandoffActivityAction::HandoffRejected { .. } => ActivityAction::ProjectCreated,
            HandoffActivityAction::HandoffExpired => ActivityAction::ProjectCreated,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::create_users_table;
    use crate::sharing::{create_project_shares_table, share_project};
    use crate::activity::create_activity_log_table;
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
        create_project_shares_table(&pool).await.expect("create shares table");
        create_activity_log_table(&pool).await.expect("create activity table");
        create_handoffs_table(&pool).await.expect("create handoffs table");

        // Create test project
        sqlx::query("INSERT INTO projects (id, name) VALUES ('proj-1', 'Test Project')")
            .execute(&pool)
            .await
            .expect("create test project");

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

        // Give alice and bob access to the project
        share_project(&pool, "share-1", "proj-1", "user-alice", ProjectRole::Owner, "user-alice")
            .await
            .expect("share to alice");
        share_project(&pool, "share-2", "proj-1", "user-bob", ProjectRole::Member, "user-alice")
            .await
            .expect("share to bob");

        (pool, dir)
    }

    fn make_request<'a>(
        id: &'a str,
        project_id: &'a str,
        from_user_id: &'a str,
        to_user_id: Option<&'a str>,
        title: &'a str,
        message: Option<&'a str>,
        context: &'a ContextSnapshot,
        priority: HandoffPriority,
    ) -> CreateHandoffRequest<'a> {
        CreateHandoffRequest {
            id,
            project_id,
            from_user_id,
            to_user_id,
            title,
            message,
            context,
            priority,
        }
    }

    #[tokio::test]
    async fn test_create_handoff() {
        let (pool, _dir) = test_pool().await;

        let context = ContextSnapshot::new("Working on feature X");
        let handoff = create_handoff(
            &pool,
            make_request(
                "handoff-1",
                "proj-1",
                "user-alice",
                Some("user-bob"),
                "Continue feature X",
                Some("Please finish the implementation"),
                &context,
                HandoffPriority::Normal,
            ),
        )
        .await
        .expect("create handoff");

        assert_eq!(handoff.id, "handoff-1");
        assert_eq!(handoff.project_id, "proj-1");
        assert_eq!(handoff.from_user_id, "user-alice");
        assert_eq!(handoff.to_user_id, Some("user-bob".to_string()));
        assert_eq!(handoff.title, "Continue feature X");
        assert!(handoff.is_pending());
    }

    #[tokio::test]
    async fn test_create_handoff_self_error() {
        let (pool, _dir) = test_pool().await;

        let context = ContextSnapshot::new("Test");
        let result = create_handoff(
            &pool,
            make_request(
                "handoff-1",
                "proj-1",
                "user-alice",
                Some("user-alice"),
                "Self handoff",
                None,
                &context,
                HandoffPriority::Normal,
            ),
        )
        .await;

        assert!(matches!(result, Err(HandoffError::SelfHandoff)));
    }

    #[tokio::test]
    async fn test_create_handoff_no_access_error() {
        let (pool, _dir) = test_pool().await;

        let context = ContextSnapshot::new("Test");
        let result = create_handoff(
            &pool,
            make_request(
                "handoff-1",
                "proj-1",
                "user-charlie",
                Some("user-bob"),
                "No access handoff",
                None,
                &context,
                HandoffPriority::Normal,
            ),
        )
        .await;

        assert!(matches!(result, Err(HandoffError::NoProjectAccess)));
    }

    #[tokio::test]
    async fn test_create_handoff_to_user_no_access_error() {
        let (pool, _dir) = test_pool().await;

        let context = ContextSnapshot::new("Test");
        let result = create_handoff(
            &pool,
            make_request(
                "handoff-1",
                "proj-1",
                "user-alice",
                Some("user-charlie"),
                "To no access user",
                None,
                &context,
                HandoffPriority::Normal,
            ),
        )
        .await;

        assert!(matches!(result, Err(HandoffError::NoProjectAccess)));
    }

    #[tokio::test]
    async fn test_create_open_handoff() {
        let (pool, _dir) = test_pool().await;

        let context = ContextSnapshot::new("Open task");
        let handoff = create_handoff(
            &pool,
            make_request(
                "handoff-1",
                "proj-1",
                "user-alice",
                None,
                "Open task",
                None,
                &context,
                HandoffPriority::High,
            ),
        )
        .await
        .expect("create open handoff");

        assert!(handoff.to_user_id.is_none());
        assert_eq!(handoff.priority, "high");
    }

    #[tokio::test]
    async fn test_accept_handoff() {
        let (pool, _dir) = test_pool().await;

        let mut context = ContextSnapshot::new("Feature X progress");
        context.open_questions = vec!["How to handle edge case?".to_string()];
        context.suggested_next_steps = vec!["Add tests".to_string()];

        create_handoff(
            &pool,
            make_request(
                "handoff-1",
                "proj-1",
                "user-alice",
                Some("user-bob"),
                "Continue feature X",
                None,
                &context,
                HandoffPriority::Normal,
            ),
        )
        .await
        .expect("create");

        let restored = accept_handoff(&pool, "handoff-1", "user-bob")
            .await
            .expect("accept");

        assert_eq!(restored.conversation_summary, "Feature X progress");
        assert_eq!(restored.open_questions.len(), 1);
        assert_eq!(restored.suggested_next_steps.len(), 1);

        let handoff = get_handoff(&pool, "handoff-1").await.expect("get").expect("exists");
        assert_eq!(handoff.status(), HandoffStatus::Accepted);
        assert_eq!(handoff.accepted_by, Some("user-bob".to_string()));
    }

    #[tokio::test]
    async fn test_accept_handoff_wrong_user() {
        let (pool, _dir) = test_pool().await;

        let context = ContextSnapshot::new("Test");
        create_handoff(
            &pool,
            make_request(
                "handoff-1",
                "proj-1",
                "user-alice",
                Some("user-bob"),
                "For Bob",
                None,
                &context,
                HandoffPriority::Normal,
            ),
        )
        .await
        .expect("create");

        let result = accept_handoff(&pool, "handoff-1", "user-alice").await;
        assert!(matches!(result, Err(HandoffError::NotAssignee)));
    }

    #[tokio::test]
    async fn test_accept_open_handoff() {
        let (pool, _dir) = test_pool().await;

        let context = ContextSnapshot::new("Open task");
        create_handoff(
            &pool,
            make_request(
                "handoff-1",
                "proj-1",
                "user-alice",
                None,
                "Open task",
                None,
                &context,
                HandoffPriority::Normal,
            ),
        )
        .await
        .expect("create");

        let restored = accept_handoff(&pool, "handoff-1", "user-bob")
            .await
            .expect("accept");

        assert_eq!(restored.conversation_summary, "Open task");
    }

    #[tokio::test]
    async fn test_reject_handoff() {
        let (pool, _dir) = test_pool().await;

        let context = ContextSnapshot::new("Test");
        create_handoff(
            &pool,
            make_request(
                "handoff-1",
                "proj-1",
                "user-alice",
                Some("user-bob"),
                "Task",
                None,
                &context,
                HandoffPriority::Normal,
            ),
        )
        .await
        .expect("create");

        reject_handoff(&pool, "handoff-1", "user-bob", Some("Too busy"))
            .await
            .expect("reject");

        let handoff = get_handoff(&pool, "handoff-1").await.expect("get").expect("exists");
        assert_eq!(handoff.status(), HandoffStatus::Rejected);
        assert_eq!(handoff.rejection_reason, Some("Too busy".to_string()));
    }

    #[tokio::test]
    async fn test_get_pending_handoffs() {
        let (pool, _dir) = test_pool().await;

        let context = ContextSnapshot::new("Test");

        create_handoff(
            &pool,
            make_request(
                "handoff-1",
                "proj-1",
                "user-alice",
                Some("user-bob"),
                "For Bob",
                None,
                &context,
                HandoffPriority::Normal,
            ),
        )
        .await
        .expect("create");

        create_handoff(
            &pool,
            make_request(
                "handoff-2",
                "proj-1",
                "user-alice",
                None,
                "Open task",
                None,
                &context,
                HandoffPriority::High,
            ),
        )
        .await
        .expect("create");

        let pending = get_pending_handoffs(&pool, "user-bob").await.expect("get");
        assert_eq!(pending.len(), 2);

        for summary in &pending {
            assert!(summary.context_summary.len() <= 203);
        }
    }

    #[tokio::test]
    async fn test_expire_old_handoffs() {
        let (pool, _dir) = test_pool().await;

        let old_date = (Utc::now() - Duration::days(10)).to_rfc3339();
        sqlx::query(
            r#"INSERT INTO handoffs (id, project_id, from_user_id, title, context_snapshot, status, created_at)
            VALUES ('old-handoff', 'proj-1', 'user-alice', 'Old task', '{}', 'pending', ?)"#,
        )
        .bind(&old_date)
        .execute(&pool)
        .await
        .expect("insert old");

        let context = ContextSnapshot::new("Recent");
        create_handoff(
            &pool,
            make_request(
                "new-handoff",
                "proj-1",
                "user-alice",
                None,
                "New task",
                None,
                &context,
                HandoffPriority::Normal,
            ),
        )
        .await
        .expect("create");

        let expired = expire_old_handoffs(&pool, 7).await.expect("expire");
        assert_eq!(expired, 1);

        let old = get_handoff(&pool, "old-handoff").await.expect("get").expect("exists");
        assert_eq!(old.status(), HandoffStatus::Expired);

        let new = get_handoff(&pool, "new-handoff").await.expect("get").expect("exists");
        assert_eq!(new.status(), HandoffStatus::Pending);
    }

    #[tokio::test]
    async fn test_handoff_summary_truncation() {
        let (pool, _dir) = test_pool().await;

        let long_summary = "A".repeat(300);
        let long_message = "B".repeat(200);
        let context = ContextSnapshot::new(&long_summary);

        create_handoff(
            &pool,
            make_request(
                "handoff-1",
                "proj-1",
                "user-alice",
                Some("user-bob"),
                "Long content",
                Some(&long_message),
                &context,
                HandoffPriority::Normal,
            ),
        )
        .await
        .expect("create");

        let pending = get_pending_handoffs(&pool, "user-bob").await.expect("get");
        assert_eq!(pending.len(), 1);

        let summary = &pending[0];
        assert!(summary.context_summary.len() <= 203);
        assert!(summary.context_summary.ends_with("..."));
        assert!(summary.message_preview.as_ref().map(|m| m.len()).unwrap_or(0) <= 103);
    }

    #[tokio::test]
    async fn test_get_project_handoffs() {
        let (pool, _dir) = test_pool().await;

        let context = ContextSnapshot::new("Test");

        create_handoff(&pool, make_request("h1", "proj-1", "user-alice", None, "Task 1", None, &context, HandoffPriority::Normal))
            .await.expect("create");
        create_handoff(&pool, make_request("h2", "proj-1", "user-alice", None, "Task 2", None, &context, HandoffPriority::Normal))
            .await.expect("create");

        accept_handoff(&pool, "h1", "user-bob").await.expect("accept");

        let all = get_project_handoffs(&pool, "proj-1", None, None).await.expect("get all");
        assert_eq!(all.len(), 2);

        let pending = get_project_handoffs(&pool, "proj-1", Some(HandoffStatus::Pending), None)
            .await.expect("get pending");
        assert_eq!(pending.len(), 1);

        let accepted = get_project_handoffs(&pool, "proj-1", Some(HandoffStatus::Accepted), None)
            .await.expect("get accepted");
        assert_eq!(accepted.len(), 1);
    }

    #[tokio::test]
    async fn test_count_pending_handoffs() {
        let (pool, _dir) = test_pool().await;

        let context = ContextSnapshot::new("Test");

        create_handoff(&pool, make_request("h1", "proj-1", "user-alice", Some("user-bob"), "T1", None, &context, HandoffPriority::Normal))
            .await.expect("create");
        create_handoff(&pool, make_request("h2", "proj-1", "user-alice", Some("user-bob"), "T2", None, &context, HandoffPriority::Normal))
            .await.expect("create");

        create_handoff(&pool, make_request("h3", "proj-1", "user-alice", None, "T3", None, &context, HandoffPriority::Normal))
            .await.expect("create");

        let count = count_pending_handoffs(&pool, "user-bob").await.expect("count");
        assert_eq!(count, 3);

        accept_handoff(&pool, "h1", "user-bob").await.expect("accept");

        let count = count_pending_handoffs(&pool, "user-bob").await.expect("count");
        assert_eq!(count, 2);
    }

    #[tokio::test]
    async fn test_context_snapshot_serialization() {
        let mut context = ContextSnapshot::new("Summary");
        context.memory_state = Some("memory data".to_string());
        context.current_branch = Some("feature/x".to_string());
        context.open_files = vec!["src/main.rs".to_string()];
        context.open_questions = vec!["Q1".to_string(), "Q2".to_string()];
        context.suggested_next_steps = vec!["Step 1".to_string()];
        context.recent_messages = vec![
            MessageSnapshot {
                role: "user".to_string(),
                content: "Hello".to_string(),
                timestamp: "2024-01-01T00:00:00Z".to_string(),
            },
        ];

        let json = context.to_json();
        let restored = ContextSnapshot::from_json(&json).expect("parse");

        assert_eq!(restored.conversation_summary, "Summary");
        assert_eq!(restored.memory_state, Some("memory data".to_string()));
        assert_eq!(restored.current_branch, Some("feature/x".to_string()));
        assert_eq!(restored.open_files.len(), 1);
        assert_eq!(restored.open_questions.len(), 2);
        assert_eq!(restored.suggested_next_steps.len(), 1);
        assert_eq!(restored.recent_messages.len(), 1);
    }

    #[tokio::test]
    async fn test_handoff_status_parsing() {
        assert_eq!(HandoffStatus::parse("pending"), HandoffStatus::Pending);
        assert_eq!(HandoffStatus::parse("ACCEPTED"), HandoffStatus::Accepted);
        assert_eq!(HandoffStatus::parse("Rejected"), HandoffStatus::Rejected);
        assert_eq!(HandoffStatus::parse("expired"), HandoffStatus::Expired);
        assert_eq!(HandoffStatus::parse("unknown"), HandoffStatus::Pending);
    }

    #[tokio::test]
    async fn test_handoff_priority_parsing() {
        assert_eq!(HandoffPriority::parse("low"), HandoffPriority::Low);
        assert_eq!(HandoffPriority::parse("NORMAL"), HandoffPriority::Normal);
        assert_eq!(HandoffPriority::parse("High"), HandoffPriority::High);
        assert_eq!(HandoffPriority::parse("urgent"), HandoffPriority::Urgent);
        assert_eq!(HandoffPriority::parse("unknown"), HandoffPriority::Normal);
    }

    #[tokio::test]
    async fn test_cannot_accept_already_accepted() {
        let (pool, _dir) = test_pool().await;

        let context = ContextSnapshot::new("Test");
        create_handoff(&pool, make_request("h1", "proj-1", "user-alice", None, "Task", None, &context, HandoffPriority::Normal))
            .await.expect("create");

        accept_handoff(&pool, "h1", "user-bob").await.expect("accept");

        let result = accept_handoff(&pool, "h1", "user-bob").await;
        assert!(matches!(result, Err(HandoffError::NotPending)));
    }

    #[tokio::test]
    async fn test_get_created_handoffs() {
        let (pool, _dir) = test_pool().await;

        let context = ContextSnapshot::new("Test");

        create_handoff(&pool, make_request("h1", "proj-1", "user-alice", None, "T1", None, &context, HandoffPriority::Normal))
            .await.expect("create");
        create_handoff(&pool, make_request("h2", "proj-1", "user-alice", None, "T2", None, &context, HandoffPriority::Normal))
            .await.expect("create");

        let created = get_created_handoffs(&pool, "user-alice", None).await.expect("get");
        assert_eq!(created.len(), 2);

        let created = get_created_handoffs(&pool, "user-bob", None).await.expect("get");
        assert_eq!(created.len(), 0);
    }
}
