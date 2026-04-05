//! Pending approval registry for safety workflow integration.
//!
//! This module provides:
//! - `PendingApproval` - An action awaiting user approval
//! - `ApprovalRegistry` - Registry tracking pending approvals with async resolution

use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{broadcast, oneshot};
use uuid::Uuid;

use cuttlefish_agents::{ActionType, ConfidenceScore, RiskFactor};

/// Default timeout for pending approvals (5 minutes).
pub const DEFAULT_APPROVAL_TIMEOUT_SECS: u64 = 300;

/// A pending action awaiting user approval.
#[derive(Debug, Clone)]
pub struct PendingApproval {
    /// Unique action ID.
    pub action_id: String,
    /// Project ID this action belongs to.
    pub project_id: String,
    /// Type of action.
    pub action_type: ActionType,
    /// Human-readable description.
    pub description: String,
    /// File path if applicable.
    pub path: Option<String>,
    /// Command if applicable.
    pub command: Option<String>,
    /// Confidence score.
    pub confidence: ConfidenceScore,
    /// Risk factors identified.
    pub risk_factors: Vec<RiskFactor>,
    /// When the approval was created.
    pub created_at: Instant,
    /// Timeout duration.
    pub timeout: Duration,
    /// Optional diff content for preview.
    pub diff: Option<String>,
}

impl PendingApproval {
    /// Create a new pending approval.
    pub fn new(
        project_id: impl Into<String>,
        action_type: ActionType,
        description: impl Into<String>,
        confidence: ConfidenceScore,
    ) -> Self {
        Self {
            action_id: Uuid::new_v4().to_string(),
            project_id: project_id.into(),
            action_type,
            description: description.into(),
            path: None,
            command: None,
            confidence,
            risk_factors: Vec::new(),
            created_at: Instant::now(),
            timeout: Duration::from_secs(DEFAULT_APPROVAL_TIMEOUT_SECS),
            diff: None,
        }
    }

    /// Set the file path.
    pub fn with_path(mut self, path: impl Into<String>) -> Self {
        self.path = Some(path.into());
        self
    }

    /// Set the command.
    pub fn with_command(mut self, command: impl Into<String>) -> Self {
        self.command = Some(command.into());
        self
    }

    /// Set risk factors.
    pub fn with_risk_factors(mut self, factors: Vec<RiskFactor>) -> Self {
        self.risk_factors = factors;
        self
    }

    /// Set the diff content.
    pub fn with_diff(mut self, diff: impl Into<String>) -> Self {
        self.diff = Some(diff.into());
        self
    }

    /// Set a custom timeout.
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Check if this approval has expired.
    pub fn is_expired(&self) -> bool {
        self.created_at.elapsed() > self.timeout
    }

    /// Get remaining time before timeout.
    pub fn time_remaining(&self) -> Duration {
        self.timeout.saturating_sub(self.created_at.elapsed())
    }
}

/// Result of an approval decision.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ApprovalDecision {
    /// Action was approved by user.
    Approved,
    /// Action was rejected by user.
    Rejected {
        /// Optional reason for rejection.
        reason: Option<String>,
    },
    /// Approval timed out.
    TimedOut,
}

impl ApprovalDecision {
    /// Check if this decision allows the action to proceed.
    pub fn allows_proceed(&self) -> bool {
        matches!(self, Self::Approved)
    }
}

/// Internal state for a pending approval.
struct ApprovalState {
    approval: PendingApproval,
    /// Channel to send the decision back to the waiter.
    decision_tx: oneshot::Sender<ApprovalDecision>,
}

/// Registry for tracking pending approvals.
pub struct ApprovalRegistry {
    /// Map of action_id -> approval state.
    pending: DashMap<String, ApprovalState>,
    /// Broadcast channel for notifying about new approvals.
    new_approval_tx: broadcast::Sender<PendingApproval>,
    /// Broadcast channel for notifying about resolved approvals.
    resolved_tx: broadcast::Sender<(String, ApprovalDecision)>,
}

impl ApprovalRegistry {
    /// Create a new approval registry.
    pub fn new() -> Self {
        let (new_approval_tx, _) = broadcast::channel(100);
        let (resolved_tx, _) = broadcast::channel(100);
        Self {
            pending: DashMap::new(),
            new_approval_tx,
            resolved_tx,
        }
    }

    /// Register a pending approval and wait for a decision.
    ///
    /// Returns when the user approves, rejects, or the timeout is reached.
    pub async fn request_approval(&self, approval: PendingApproval) -> ApprovalDecision {
        let action_id = approval.action_id.clone();
        let timeout = approval.timeout;

        // Create oneshot channel for the decision
        let (decision_tx, decision_rx) = oneshot::channel();

        // Broadcast new approval notification
        let _ = self.new_approval_tx.send(approval.clone());

        // Store the approval state
        self.pending.insert(
            action_id.clone(),
            ApprovalState {
                approval,
                decision_tx,
            },
        );

        // Wait for decision or timeout
        let decision = tokio::select! {
            result = decision_rx => {
                match result {
                    Ok(decision) => decision,
                    Err(_) => ApprovalDecision::TimedOut, // Channel dropped
                }
            }
            _ = tokio::time::sleep(timeout) => {
                // Remove expired approval
                self.pending.remove(&action_id);
                ApprovalDecision::TimedOut
            }
        };

        // Broadcast resolution
        let _ = self.resolved_tx.send((action_id, decision.clone()));

        decision
    }

    /// Approve a pending action.
    ///
    /// Returns `true` if the action was found and approved, `false` if not found.
    pub fn approve(&self, action_id: &str) -> bool {
        if let Some((_, state)) = self.pending.remove(action_id) {
            let _ = state.decision_tx.send(ApprovalDecision::Approved);
            true
        } else {
            false
        }
    }

    /// Reject a pending action.
    ///
    /// Returns `true` if the action was found and rejected, `false` if not found.
    pub fn reject(&self, action_id: &str, reason: Option<String>) -> bool {
        if let Some((_, state)) = self.pending.remove(action_id) {
            let _ = state
                .decision_tx
                .send(ApprovalDecision::Rejected { reason });
            true
        } else {
            false
        }
    }

    /// Get a pending approval by ID (for preview/details).
    pub fn get(&self, action_id: &str) -> Option<PendingApproval> {
        self.pending.get(action_id).map(|s| s.approval.clone())
    }

    /// Get all pending approvals for a project.
    pub fn get_for_project(&self, project_id: &str) -> Vec<PendingApproval> {
        self.pending
            .iter()
            .filter(|entry| entry.approval.project_id == project_id)
            .map(|entry| entry.approval.clone())
            .collect()
    }

    /// Get all pending approvals.
    pub fn get_all(&self) -> Vec<PendingApproval> {
        self.pending
            .iter()
            .map(|entry| entry.approval.clone())
            .collect()
    }

    /// Subscribe to new approval notifications.
    pub fn subscribe_new(&self) -> broadcast::Receiver<PendingApproval> {
        self.new_approval_tx.subscribe()
    }

    /// Subscribe to approval resolution notifications.
    pub fn subscribe_resolved(&self) -> broadcast::Receiver<(String, ApprovalDecision)> {
        self.resolved_tx.subscribe()
    }

    /// Remove expired approvals.
    pub fn cleanup_expired(&self) -> usize {
        let expired: Vec<String> = self
            .pending
            .iter()
            .filter(|entry| entry.approval.is_expired())
            .map(|entry| entry.key().clone())
            .collect();

        let count = expired.len();
        for action_id in expired {
            if let Some((_, state)) = self.pending.remove(&action_id) {
                let _ = state.decision_tx.send(ApprovalDecision::TimedOut);
            }
        }
        count
    }

    /// Get the count of pending approvals.
    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }
}

impl Default for ApprovalRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Shared approval registry wrapped in Arc.
pub type SharedApprovalRegistry = Arc<ApprovalRegistry>;

/// Create a new shared approval registry.
pub fn create_approval_registry() -> SharedApprovalRegistry {
    Arc::new(ApprovalRegistry::new())
}

#[cfg(test)]
mod tests {
    use super::*;
    use cuttlefish_agents::ConfidenceScore;

    fn make_approval(project_id: &str) -> PendingApproval {
        PendingApproval::new(
            project_id,
            ActionType::FileWrite,
            "Write to main.rs",
            ConfidenceScore::new(0.7, vec![], "test"),
        )
        .with_path("src/main.rs")
    }

    #[test]
    fn test_pending_approval_creation() {
        let approval = make_approval("proj-1");
        assert!(!approval.is_expired());
        assert_eq!(approval.project_id, "proj-1");
        assert_eq!(approval.path, Some("src/main.rs".to_string()));
    }

    #[test]
    fn test_pending_approval_with_command() {
        let approval = PendingApproval::new(
            "proj-1",
            ActionType::BashCommand,
            "Run tests",
            ConfidenceScore::new(0.8, vec![], "test"),
        )
        .with_command("cargo test");

        assert_eq!(approval.command, Some("cargo test".to_string()));
        assert!(approval.path.is_none());
    }

    #[test]
    fn test_approval_decision_allows_proceed() {
        assert!(ApprovalDecision::Approved.allows_proceed());
        assert!(!ApprovalDecision::Rejected { reason: None }.allows_proceed());
        assert!(!ApprovalDecision::TimedOut.allows_proceed());
    }

    #[tokio::test]
    async fn test_registry_approve() {
        let registry = ApprovalRegistry::new();
        let approval = make_approval("proj-1");
        let action_id = approval.action_id.clone();

        // Spawn a task to wait for approval
        let registry_clone = Arc::new(registry);
        let registry_for_wait = Arc::clone(&registry_clone);
        let wait_task =
            tokio::spawn(async move { registry_for_wait.request_approval(approval).await });

        // Give the task time to register
        tokio::time::sleep(Duration::from_millis(10)).await;

        // Approve the action
        assert!(registry_clone.approve(&action_id));

        // Check the result
        let decision = wait_task.await.expect("task panicked");
        assert!(matches!(decision, ApprovalDecision::Approved));
    }

    #[tokio::test]
    async fn test_registry_reject() {
        let registry = Arc::new(ApprovalRegistry::new());
        let approval = make_approval("proj-1");
        let action_id = approval.action_id.clone();

        let registry_for_wait = Arc::clone(&registry);
        let wait_task =
            tokio::spawn(async move { registry_for_wait.request_approval(approval).await });

        tokio::time::sleep(Duration::from_millis(10)).await;

        assert!(registry.reject(&action_id, Some("Too risky".to_string())));

        let decision = wait_task.await.expect("task panicked");
        if let ApprovalDecision::Rejected { reason } = decision {
            assert_eq!(reason, Some("Too risky".to_string()));
        } else {
            panic!("Expected Rejected decision");
        }
    }

    #[tokio::test]
    async fn test_registry_timeout() {
        let registry = Arc::new(ApprovalRegistry::new());
        let approval = make_approval("proj-1").with_timeout(Duration::from_millis(50));

        let decision = registry.request_approval(approval).await;
        assert!(matches!(decision, ApprovalDecision::TimedOut));
    }

    #[test]
    fn test_registry_get_for_project() {
        let registry = ApprovalRegistry::new();

        // We can't easily test this without async context, but we can test empty case
        let approvals = registry.get_for_project("proj-1");
        assert!(approvals.is_empty());
    }

    #[test]
    fn test_registry_pending_count() {
        let registry = ApprovalRegistry::new();
        assert_eq!(registry.pending_count(), 0);
    }

    #[test]
    fn test_approve_nonexistent() {
        let registry = ApprovalRegistry::new();
        assert!(!registry.approve("nonexistent"));
    }

    #[test]
    fn test_reject_nonexistent() {
        let registry = ApprovalRegistry::new();
        assert!(!registry.reject("nonexistent", None));
    }
}
