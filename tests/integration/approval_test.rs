//! Integration test for approval workflow.

use cuttlefish_agents::{ActionType, ConfidenceScore};
use cuttlefish_api::{ApprovalDecision, ApprovalRegistry, PendingApproval};
use std::sync::Arc;
use std::time::Duration;

#[tokio::test]
async fn test_approval_workflow_approve() {
    let registry = Arc::new(ApprovalRegistry::new());

    let approval = PendingApproval::new(
        "test-project",
        ActionType::FileWrite,
        "Write test file",
        ConfidenceScore::medium("Test action"),
    )
    .with_path("test.rs")
    .with_timeout(Duration::from_secs(5));

    let action_id = approval.action_id.clone();
    let registry_clone = Arc::clone(&registry);

    // Spawn task to wait for approval
    let wait_task = tokio::spawn(async move { registry_clone.request_approval(approval).await });

    // Give the task time to register
    tokio::time::sleep(Duration::from_millis(50)).await;

    // Approve the action
    assert!(registry.approve(&action_id));

    // Check the result
    let decision = wait_task.await.expect("task should complete");
    assert!(matches!(decision, ApprovalDecision::Approved));
}

#[tokio::test]
async fn test_approval_workflow_reject() {
    let registry = Arc::new(ApprovalRegistry::new());

    let approval = PendingApproval::new(
        "test-project",
        ActionType::BashCommand,
        "Run dangerous command",
        ConfidenceScore::low("Risky operation"),
    )
    .with_command("rm -rf /tmp/test")
    .with_timeout(Duration::from_secs(5));

    let action_id = approval.action_id.clone();
    let registry_clone = Arc::clone(&registry);

    let wait_task = tokio::spawn(async move { registry_clone.request_approval(approval).await });

    tokio::time::sleep(Duration::from_millis(50)).await;

    assert!(registry.reject(&action_id, Some("Too risky".to_string())));

    let decision = wait_task.await.expect("task should complete");
    if let ApprovalDecision::Rejected { reason } = decision {
        assert_eq!(reason, Some("Too risky".to_string()));
    } else {
        panic!("Expected Rejected decision");
    }
}

#[tokio::test]
async fn test_approval_workflow_timeout() {
    let registry = Arc::new(ApprovalRegistry::new());

    let approval = PendingApproval::new(
        "test-project",
        ActionType::FileWrite,
        "Write file",
        ConfidenceScore::medium("Test"),
    )
    .with_timeout(Duration::from_millis(100)); // Very short timeout

    let decision = registry.request_approval(approval).await;
    assert!(matches!(decision, ApprovalDecision::TimedOut));
}

#[tokio::test]
async fn test_approval_registry_get_for_project() {
    let registry = Arc::new(ApprovalRegistry::new());

    // Create two approvals for different projects
    let approval1 = PendingApproval::new(
        "project-1",
        ActionType::FileWrite,
        "Action 1",
        ConfidenceScore::medium("Test"),
    )
    .with_timeout(Duration::from_secs(60));

    let approval2 = PendingApproval::new(
        "project-2",
        ActionType::FileWrite,
        "Action 2",
        ConfidenceScore::medium("Test"),
    )
    .with_timeout(Duration::from_secs(60));

    let id1 = approval1.action_id.clone();
    let id2 = approval2.action_id.clone();

    // Spawn tasks that will wait
    let registry1 = Arc::clone(&registry);
    let registry2 = Arc::clone(&registry);

    let _task1 = tokio::spawn(async move { registry1.request_approval(approval1).await });
    let _task2 = tokio::spawn(async move { registry2.request_approval(approval2).await });

    tokio::time::sleep(Duration::from_millis(50)).await;

    // Check project filtering
    let project1_approvals = registry.get_for_project("project-1");
    assert_eq!(project1_approvals.len(), 1);
    assert_eq!(project1_approvals[0].project_id, "project-1");

    let project2_approvals = registry.get_for_project("project-2");
    assert_eq!(project2_approvals.len(), 1);
    assert_eq!(project2_approvals[0].project_id, "project-2");

    // Clean up
    registry.approve(&id1);
    registry.approve(&id2);
}

#[tokio::test]
async fn test_approval_nonexistent_action() {
    let registry = ApprovalRegistry::new();

    // These should return false, not panic
    assert!(!registry.approve("nonexistent-id"));
    assert!(!registry.reject("nonexistent-id", None));
}
