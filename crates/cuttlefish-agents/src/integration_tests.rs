//! Integration tests for the agent workflow system.
//!
//! These tests verify the complete workflow from task input to agent output
//! using mock providers. No real Docker, Discord, or GitHub required.

#[cfg(test)]
mod workflow_integration {
    use crate::{bus::TokioMessageBus, workflow::WorkflowEngine};
    use cuttlefish_core::traits::bus::MessageBus;
    use cuttlefish_providers::mock::MockModelProvider;
    use std::sync::Arc;
    use uuid::Uuid;

    /// Test that a complete workflow succeeds when critic approves on first try.
    #[tokio::test]
    async fn test_full_workflow_approve_on_first_try() {
        let mock = MockModelProvider::new("integration-test");
        // Orchestrator plans one task
        mock.add_response(
            r#"{"tasks": [{"id": "1", "description": "Create hello.js", "agent": "coder"}]}"#,
        );
        // Coder creates the file
        mock.add_response("Created hello.js: console.log('Hello, World!');");
        // Critic approves
        mock.add_response(
            r#"{"verdict": "approve", "issues": [], "summary": "Clean, working code"}"#,
        );

        let engine = WorkflowEngine::new(Arc::new(mock), TokioMessageBus::new());
        let result = engine
            .execute(Uuid::new_v4(), "Create a hello world Node.js script")
            .await
            .expect("workflow should execute");

        assert!(result.success, "Workflow should succeed with approved code");
        assert_eq!(result.iterations, 1, "Should complete in one iteration");
        assert_eq!(result.final_verdict.as_deref(), Some("approve"));
    }

    /// Test that a workflow retries when critic rejects, then succeeds.
    #[tokio::test]
    async fn test_full_workflow_retry_and_succeed() {
        let mock = MockModelProvider::new("integration-retry");
        // Orchestrator
        mock.add_response("{}"); // Fallback single task
        // First coder attempt — buggy
        mock.add_response("Created file with bug on line 5");
        // First critic — reject
        mock.add_response(
            r#"{"verdict": "reject", "issues": [{"file": "hello.js", "line": 5, "message": "Unhandled error"}], "summary": "Bug found"}"#,
        );
        // Second coder attempt — fixed
        mock.add_response("Fixed: added try-catch around line 5");
        // Second critic — approve
        mock.add_response(
            r#"{"verdict": "approve", "issues": [], "summary": "Bug fixed, code is clean"}"#,
        );

        let engine = WorkflowEngine::new(Arc::new(mock), TokioMessageBus::new());
        let result = engine
            .execute(Uuid::new_v4(), "Create a robust Node.js script")
            .await
            .expect("workflow should execute");

        assert!(result.success, "Should succeed after retry");
        assert_eq!(result.iterations, 2, "Should take 2 iterations");
    }

    /// Test that message bus pub/sub works within workflow execution.
    #[tokio::test]
    async fn test_message_bus_pubsub_in_workflow() {
        let bus = TokioMessageBus::new();

        // Subscribe before publish
        let mut rx = bus
            .subscribe("agent.coder.input")
            .await
            .expect("should subscribe");

        let mock = MockModelProvider::new("bus-test");
        // Orchestrator dispatches to coder
        mock.add_response(
            r#"{"tasks": [{"id": "1", "description": "Write tests", "agent": "coder"}]}"#,
        );
        // Coder and critic responses
        mock.add_response("Tests written");
        mock.add_response(r#"{"verdict": "approve", "issues": [], "summary": "Good tests"}"#);

        let engine = WorkflowEngine::with_max_iterations(Arc::new(mock), bus, 1);
        let _ = engine
            .execute(Uuid::new_v4(), "Write unit tests")
            .await
            .expect("workflow should execute");

        // The orchestrator should have published to the coder topic
        assert!(
            rx.try_recv().is_ok(),
            "Coder should have received a task via bus"
        );
    }

    /// Test that workflow result contains content from execution.
    #[tokio::test]
    async fn test_workflow_result_contains_content() {
        let mock = MockModelProvider::new("content-test");
        mock.add_response("{}");
        mock.add_response("Created index.ts with TypeScript hello world");
        mock.add_response(r#"{"verdict": "approve", "issues": [], "summary": "Approved"}"#);

        let engine = WorkflowEngine::new(Arc::new(mock), TokioMessageBus::new());
        let result = engine
            .execute(Uuid::new_v4(), "Create TypeScript hello world")
            .await
            .expect("workflow should execute");

        assert!(!result.content.is_empty(), "Result should have content");
    }

    /// Test that workflow stops at max iterations without approval.
    #[tokio::test]
    async fn test_workflow_stops_at_max_iterations() {
        let mock = MockModelProvider::new("max-iter-test");
        // Orchestrator
        mock.add_response("{}");
        // Add reject cycle for each iteration (max 3)
        for _ in 0..3 {
            mock.add_response("Code attempt");
            mock.add_response(r#"{"verdict": "reject", "issues": [], "summary": "Still broken"}"#);
        }

        let engine = WorkflowEngine::with_max_iterations(Arc::new(mock), TokioMessageBus::new(), 3);
        let result = engine
            .execute(Uuid::new_v4(), "Task")
            .await
            .expect("workflow should execute");

        assert!(!result.success, "Should fail when max iterations reached");
        assert_eq!(result.iterations, 3, "Should reach max iterations");
    }

    /// Test that workflow handles multiple tasks from orchestrator.
    #[tokio::test]
    async fn test_workflow_with_multiple_tasks() {
        let mock = MockModelProvider::new("multi-task");
        // Orchestrator plans multiple tasks
        mock.add_response(
            r#"{"tasks": [{"id": "1", "description": "Task 1", "agent": "coder"}, {"id": "2", "description": "Task 2", "agent": "coder"}]}"#,
        );
        // First task: coder + critic
        mock.add_response("Completed task 1");
        mock.add_response(r#"{"verdict": "approve", "issues": [], "summary": "Task 1 done"}"#);

        let engine = WorkflowEngine::new(Arc::new(mock), TokioMessageBus::new());
        let result = engine
            .execute(Uuid::new_v4(), "Execute multiple tasks")
            .await
            .expect("workflow should execute");

        assert!(result.success, "Should succeed with multiple tasks");
    }
}

#[cfg(test)]
mod api_integration {
    use cuttlefish_api::{build_app, routes::AppState};

    /// Test that server message serializes to JSON correctly.
    #[test]
    fn test_server_message_serialization() {
        use cuttlefish_api::ServerMessage;

        let msg = ServerMessage::Response {
            project_id: "test-project".to_string(),
            agent: "orchestrator".to_string(),
            content: "Hello from agent".to_string(),
        };
        let json = msg.to_json();
        assert!(json.contains("response"));
        assert!(json.contains("test-project"));
    }

    /// Test that server message error variant serializes correctly.
    #[test]
    fn test_server_message_error_serialization() {
        use cuttlefish_api::ServerMessage;

        let msg = ServerMessage::Error {
            message: "Test error".to_string(),
        };
        let json = msg.to_json();
        assert!(json.contains("error"));
        assert!(json.contains("Test error"));
    }

    /// Test that server message pong variant serializes correctly.
    #[test]
    fn test_server_message_pong_serialization() {
        use cuttlefish_api::ServerMessage;

        let msg = ServerMessage::Pong;
        let json = msg.to_json();
        assert!(json.contains("pong"));
    }

    /// Test that app state can be created.
    #[test]
    fn test_app_state_creation() {
        let state = AppState {
            api_key: "test-key".to_string(),
        };
        assert_eq!(state.api_key, "test-key");
    }

    /// Test that build_app returns a router.
    #[test]
    fn test_build_app_returns_router() {
        let state = AppState {
            api_key: "test-key".to_string(),
        };
        let _app = build_app(state);
        // If this compiles and runs, the router was built successfully
    }
}

#[cfg(test)]
mod message_bus_integration {
    use crate::bus::TokioMessageBus;
    use cuttlefish_core::traits::bus::{BusMessage, MessageBus};

    /// Test that multiple subscribers receive the same message.
    #[tokio::test]
    async fn test_multiple_subscribers_receive_message() {
        let bus = TokioMessageBus::new();
        let mut rx1 = bus.subscribe("test.topic").await.expect("should subscribe");
        let mut rx2 = bus.subscribe("test.topic").await.expect("should subscribe");

        let msg = BusMessage::new("test.topic", serde_json::json!({"data": "test"}));
        bus.publish(msg).await.expect("should publish");

        assert!(rx1.recv().await.is_ok(), "First subscriber should receive");
        assert!(rx2.recv().await.is_ok(), "Second subscriber should receive");
    }

    /// Test that different topics are isolated.
    #[tokio::test]
    async fn test_topic_isolation() {
        let bus = TokioMessageBus::new();
        let mut rx_a = bus.subscribe("topic.a").await.expect("should subscribe");
        let mut rx_b = bus.subscribe("topic.b").await.expect("should subscribe");

        let msg = BusMessage::new("topic.a", serde_json::json!({}));
        bus.publish(msg).await.expect("should publish");

        assert!(rx_a.recv().await.is_ok(), "Topic A should receive");
        assert!(
            rx_b.try_recv().is_err(),
            "Topic B should not receive message from topic A"
        );
    }

    /// Test that late subscribers don't receive old messages.
    #[tokio::test]
    async fn test_late_subscriber_misses_old_message() {
        let bus = TokioMessageBus::new();
        let mut rx1 = bus.subscribe("late.topic").await.expect("should subscribe");

        let msg = BusMessage::new("late.topic", serde_json::json!({}));
        bus.publish(msg).await.expect("should publish");

        // Receive the message with first subscriber
        assert!(rx1.recv().await.is_ok());

        // Subscribe after message was published
        let mut rx2 = bus.subscribe("late.topic").await.expect("should subscribe");

        // Late subscriber should not have the old message
        assert!(
            rx2.try_recv().is_err(),
            "Late subscriber should not receive old message"
        );
    }
}

#[cfg(test)]
mod cross_crate_wiring {
    use crate::{bus::TokioMessageBus, workflow::WorkflowEngine};
    use cuttlefish_providers::mock::MockModelProvider;
    use std::sync::Arc;
    use uuid::Uuid;

    /// Test that WorkflowEngine properly wires all agents together.
    #[tokio::test]
    async fn test_workflow_engine_wires_all_agents() {
        let mock = MockModelProvider::new("wiring-test");
        // Orchestrator response
        mock.add_response(
            r#"{"tasks": [{"id": "1", "description": "Test task", "agent": "coder"}]}"#,
        );
        // Coder response
        mock.add_response("Code output");
        // Critic response
        mock.add_response(r#"{"verdict": "approve", "issues": [], "summary": "OK"}"#);

        let bus = TokioMessageBus::new();
        let engine = WorkflowEngine::new(Arc::new(mock), bus);

        let result = engine
            .execute(Uuid::new_v4(), "Test input")
            .await
            .expect("workflow should execute");

        assert!(result.success, "Workflow should complete successfully");
        assert!(!result.content.is_empty(), "Should have content from coder");
        assert_eq!(result.final_verdict.as_deref(), Some("approve"));
    }

    /// Test that workflow engine respects max iterations setting.
    #[tokio::test]
    async fn test_workflow_engine_respects_max_iterations() {
        let mock = MockModelProvider::new("iter-limit");
        mock.add_response("{}");
        // Add 2 reject cycles
        for _ in 0..2 {
            mock.add_response("Attempt");
            mock.add_response(r#"{"verdict": "reject", "issues": [], "summary": "No"}"#);
        }

        let engine = WorkflowEngine::with_max_iterations(Arc::new(mock), TokioMessageBus::new(), 2);
        let result = engine
            .execute(Uuid::new_v4(), "Task")
            .await
            .expect("workflow should execute");

        assert_eq!(result.iterations, 2, "Should stop at max iterations");
        assert!(
            !result.success,
            "Should not succeed when max iterations reached"
        );
    }

    /// Test that workflow engine integrates with message bus.
    #[tokio::test]
    async fn test_workflow_engine_uses_message_bus() {
        let bus = TokioMessageBus::new();

        let mock = MockModelProvider::new("bus-integration");
        mock.add_response(r#"{"tasks": [{"id": "1", "description": "Task", "agent": "coder"}]}"#);
        mock.add_response("Output");
        mock.add_response(r#"{"verdict": "approve", "issues": [], "summary": "OK"}"#);

        let engine = WorkflowEngine::new(Arc::new(mock), bus.clone());
        let result = engine
            .execute(Uuid::new_v4(), "Input")
            .await
            .expect("workflow should execute");

        assert!(result.success, "Workflow should complete successfully");
        assert!(!result.content.is_empty(), "Should have content");
    }
}
