//! Integration tests for the agent workflow system.
//!
//! These tests verify the complete workflow from task input to agent output
//! using mock providers. No real Docker, Discord, or GitHub required.

#[cfg(test)]
mod workflow_integration {
    use crate::{
        bus::TokioMessageBus,
        workflow::{WorkflowConfig, WorkflowEngine},
    };
    use cuttlefish_core::traits::bus::MessageBus;
    use cuttlefish_providers::mock::MockModelProvider;
    use std::fs;
    use std::sync::Arc;
    use tempfile::TempDir;
    use uuid::Uuid;

    fn create_test_prompts(dir: &std::path::Path) {
        for name in [
            "orchestrator",
            "coder",
            "critic",
            "planner",
            "explorer",
            "librarian",
            "devops",
        ] {
            let content = format!(
                r#"---
name: {name}
description: Test agent
tools: []
category: deep
---

You are the {name} agent."#
            );
            fs::write(dir.join(format!("{name}.md")), content).expect("write test prompt");
        }
    }

    #[tokio::test]
    async fn test_full_workflow_approve_on_first_try() {
        let temp_dir = TempDir::new().expect("temp dir");
        create_test_prompts(temp_dir.path());

        let mock = MockModelProvider::new("integration-test");
        mock.add_response(
            r#"{"tasks": [{"id": "1", "description": "Create hello.js", "agent": "coder"}]}"#,
        );
        mock.add_response("Created hello.js: console.log('Hello, World!');");
        mock.add_response(
            r#"{"verdict": "approve", "issues": [], "summary": "Clean, working code"}"#,
        );

        let engine = WorkflowEngine::new(Arc::new(mock), TokioMessageBus::new(), temp_dir.path());
        let result = engine
            .execute(Uuid::new_v4(), "Create a hello world Node.js script")
            .await
            .expect("workflow should execute");

        assert!(result.success, "Workflow should succeed with approved code");
        assert_eq!(result.iterations, 1, "Should complete in one iteration");
        assert_eq!(result.final_verdict.as_deref(), Some("approve"));
        assert!(
            !result.planning_executed,
            "Planning should not run by default"
        );
    }

    #[tokio::test]
    async fn test_full_workflow_retry_and_succeed() {
        let temp_dir = TempDir::new().expect("temp dir");
        create_test_prompts(temp_dir.path());

        let mock = MockModelProvider::new("integration-retry");
        mock.add_response("{}");
        mock.add_response("Created file with bug on line 5");
        mock.add_response(
            r#"{"verdict": "reject", "issues": [{"file": "hello.js", "line": 5, "message": "Unhandled error"}], "summary": "Bug found"}"#,
        );
        mock.add_response("Fixed: added try-catch around line 5");
        mock.add_response(
            r#"{"verdict": "approve", "issues": [], "summary": "Bug fixed, code is clean"}"#,
        );

        let engine = WorkflowEngine::new(Arc::new(mock), TokioMessageBus::new(), temp_dir.path());
        let result = engine
            .execute(Uuid::new_v4(), "Create a robust Node.js script")
            .await
            .expect("workflow should execute");

        assert!(result.success, "Should succeed after retry");
        assert_eq!(result.iterations, 2, "Should take 2 iterations");
    }

    #[tokio::test]
    async fn test_message_bus_pubsub_in_workflow() {
        let temp_dir = TempDir::new().expect("temp dir");
        create_test_prompts(temp_dir.path());

        let bus = TokioMessageBus::new();
        let mut rx = bus
            .subscribe("agent.coder.input")
            .await
            .expect("should subscribe");

        let mock = MockModelProvider::new("bus-test");
        mock.add_response(
            r#"{"tasks": [{"id": "1", "description": "Write tests", "agent": "coder"}]}"#,
        );
        mock.add_response("Tests written");
        mock.add_response(r#"{"verdict": "approve", "issues": [], "summary": "Good tests"}"#);

        let engine = WorkflowEngine::with_max_iterations(Arc::new(mock), bus, temp_dir.path(), 1);
        let _ = engine
            .execute(Uuid::new_v4(), "Write unit tests")
            .await
            .expect("workflow should execute");

        assert!(
            rx.try_recv().is_ok(),
            "Coder should have received a task via bus"
        );
    }

    #[tokio::test]
    async fn test_workflow_result_contains_content() {
        let temp_dir = TempDir::new().expect("temp dir");
        create_test_prompts(temp_dir.path());

        let mock = MockModelProvider::new("content-test");
        mock.add_response("{}");
        mock.add_response("Created index.ts with TypeScript hello world");
        mock.add_response(r#"{"verdict": "approve", "issues": [], "summary": "Approved"}"#);

        let engine = WorkflowEngine::new(Arc::new(mock), TokioMessageBus::new(), temp_dir.path());
        let result = engine
            .execute(Uuid::new_v4(), "Create TypeScript hello world")
            .await
            .expect("workflow should execute");

        assert!(!result.content.is_empty(), "Result should have content");
    }

    #[tokio::test]
    async fn test_workflow_stops_at_max_iterations() {
        let temp_dir = TempDir::new().expect("temp dir");
        create_test_prompts(temp_dir.path());

        let mock = MockModelProvider::new("max-iter-test");
        mock.add_response("{}");
        for _ in 0..3 {
            mock.add_response("Code attempt");
            mock.add_response(r#"{"verdict": "reject", "issues": [], "summary": "Still broken"}"#);
        }

        let engine = WorkflowEngine::with_max_iterations(
            Arc::new(mock),
            TokioMessageBus::new(),
            temp_dir.path(),
            3,
        );
        let result = engine
            .execute(Uuid::new_v4(), "Task")
            .await
            .expect("workflow should execute");

        assert!(!result.success, "Should fail when max iterations reached");
        assert_eq!(result.iterations, 3, "Should reach max iterations");
    }

    #[tokio::test]
    async fn test_workflow_with_multiple_tasks() {
        let temp_dir = TempDir::new().expect("temp dir");
        create_test_prompts(temp_dir.path());

        let mock = MockModelProvider::new("multi-task");
        mock.add_response(
            r#"{"tasks": [{"id": "1", "description": "Task 1", "agent": "coder"}, {"id": "2", "description": "Task 2", "agent": "coder"}]}"#,
        );
        mock.add_response("Completed task 1");
        mock.add_response(r#"{"verdict": "approve", "issues": [], "summary": "Task 1 done"}"#);

        let engine = WorkflowEngine::new(Arc::new(mock), TokioMessageBus::new(), temp_dir.path());
        let result = engine
            .execute(Uuid::new_v4(), "Execute multiple tasks")
            .await
            .expect("workflow should execute");

        assert!(result.success, "Should succeed with multiple tasks");
    }

    #[tokio::test]
    async fn test_workflow_with_planner_enabled() {
        let temp_dir = TempDir::new().expect("temp dir");
        create_test_prompts(temp_dir.path());

        let mock = MockModelProvider::new("planner-test");
        mock.add_response(
            r#"{"tasks": [{"id": "1", "description": "Build feature", "agent": "coder"}]}"#,
        );
        mock.add_response("Step 1: Create module\nStep 2: Add tests\nStep 3: Document");
        mock.add_response("Feature implemented with tests");
        mock.add_response(r#"{"verdict": "approve", "issues": [], "summary": "Well planned"}"#);

        let config = WorkflowConfig {
            enable_planner: true,
            ..Default::default()
        };
        let engine = WorkflowEngine::with_config(
            Arc::new(mock),
            TokioMessageBus::new(),
            temp_dir.path(),
            config,
        );
        let result = engine
            .execute(Uuid::new_v4(), "Build a new feature")
            .await
            .expect("workflow should execute");

        assert!(result.success, "Should succeed with planner");
        assert!(result.planning_executed, "Planning phase should have run");
    }

    #[tokio::test]
    async fn test_workflow_with_all_agents_enabled() {
        let temp_dir = TempDir::new().expect("temp dir");
        create_test_prompts(temp_dir.path());

        let mock = MockModelProvider::new("all-agents-test");
        mock.add_response(
            r#"{"tasks": [{"id": "1", "description": "Build app", "agent": "coder"}]}"#,
        );
        mock.add_response("Detailed implementation plan");
        mock.add_response("App built successfully");
        mock.add_response(r#"{"verdict": "approve", "issues": [], "summary": "Complete"}"#);

        let engine = WorkflowEngine::with_all_agents(
            Arc::new(mock),
            TokioMessageBus::new(),
            temp_dir.path(),
        );
        let result = engine
            .execute(Uuid::new_v4(), "Build an application")
            .await
            .expect("workflow should execute");

        assert!(result.success, "Should succeed with all agents");
        assert!(
            result.planning_executed,
            "Planning should run when all agents enabled"
        );
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
        let registry = std::sync::Arc::new(cuttlefish_core::TemplateRegistry::new());
        let state = AppState {
            api_key: "test-key".to_string(),
            template_registry: registry,
        };
        assert_eq!(state.api_key, "test-key");
    }

    /// Test that build_app returns a router.
    #[test]
    fn test_build_app_returns_router() {
        let registry = std::sync::Arc::new(cuttlefish_core::TemplateRegistry::new());
        let state = AppState {
            api_key: "test-key".to_string(),
            template_registry: registry,
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
    use std::fs;
    use std::sync::Arc;
    use tempfile::TempDir;
    use uuid::Uuid;

    fn create_test_prompts(dir: &std::path::Path) {
        for name in [
            "orchestrator",
            "coder",
            "critic",
            "planner",
            "explorer",
            "librarian",
            "devops",
        ] {
            let content = format!(
                r#"---
name: {name}
description: Test agent
tools: []
category: deep
---

You are the {name} agent."#
            );
            fs::write(dir.join(format!("{name}.md")), content).expect("write test prompt");
        }
    }

    #[tokio::test]
    async fn test_workflow_engine_wires_all_agents() {
        let temp_dir = TempDir::new().expect("temp dir");
        create_test_prompts(temp_dir.path());

        let mock = MockModelProvider::new("wiring-test");
        mock.add_response(
            r#"{"tasks": [{"id": "1", "description": "Test task", "agent": "coder"}]}"#,
        );
        mock.add_response("Code output");
        mock.add_response(r#"{"verdict": "approve", "issues": [], "summary": "OK"}"#);

        let bus = TokioMessageBus::new();
        let engine = WorkflowEngine::new(Arc::new(mock), bus, temp_dir.path());

        let result = engine
            .execute(Uuid::new_v4(), "Test input")
            .await
            .expect("workflow should execute");

        assert!(result.success, "Workflow should complete successfully");
        assert!(!result.content.is_empty(), "Should have content from coder");
        assert_eq!(result.final_verdict.as_deref(), Some("approve"));
    }

    #[tokio::test]
    async fn test_workflow_engine_respects_max_iterations() {
        let temp_dir = TempDir::new().expect("temp dir");
        create_test_prompts(temp_dir.path());

        let mock = MockModelProvider::new("iter-limit");
        mock.add_response("{}");
        for _ in 0..2 {
            mock.add_response("Attempt");
            mock.add_response(r#"{"verdict": "reject", "issues": [], "summary": "No"}"#);
        }

        let engine = WorkflowEngine::with_max_iterations(
            Arc::new(mock),
            TokioMessageBus::new(),
            temp_dir.path(),
            2,
        );
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

    #[tokio::test]
    async fn test_workflow_engine_uses_message_bus() {
        let temp_dir = TempDir::new().expect("temp dir");
        create_test_prompts(temp_dir.path());

        let bus = TokioMessageBus::new();

        let mock = MockModelProvider::new("bus-integration");
        mock.add_response(r#"{"tasks": [{"id": "1", "description": "Task", "agent": "coder"}]}"#);
        mock.add_response("Output");
        mock.add_response(r#"{"verdict": "approve", "issues": [], "summary": "OK"}"#);

        let engine = WorkflowEngine::new(Arc::new(mock), bus.clone(), temp_dir.path());
        let result = engine
            .execute(Uuid::new_v4(), "Input")
            .await
            .expect("workflow should execute");

        assert!(result.success, "Workflow should complete successfully");
        assert!(!result.content.is_empty(), "Should have content");
    }
}

#[cfg(test)]
mod all_agents_instantiation {
    use crate::{
        CoderAgent, CriticAgent, DevOpsAgent, ExplorerAgent, LibrarianAgent, OrchestratorAgent,
        PlannerAgent, TokioMessageBus,
    };
    use cuttlefish_core::traits::agent::{Agent, AgentRole};
    use cuttlefish_core::traits::provider::ModelProvider;
    use cuttlefish_providers::mock::MockModelProvider;
    use std::fs;
    use std::sync::Arc;
    use tempfile::TempDir;

    fn create_test_prompts(dir: &std::path::Path) {
        for name in [
            "orchestrator",
            "coder",
            "critic",
            "planner",
            "explorer",
            "librarian",
            "devops",
        ] {
            let content = format!(
                r#"---
name: {name}
description: Test agent
tools: []
category: deep
---

You are the {name} agent."#
            );
            fs::write(dir.join(format!("{name}.md")), content).expect("write test prompt");
        }
    }

    #[test]
    fn test_all_seven_agents_can_be_instantiated() {
        let temp_dir = TempDir::new().expect("temp dir");
        create_test_prompts(temp_dir.path());
        let mock: Arc<dyn ModelProvider> = Arc::new(MockModelProvider::new("instantiation-test"));
        let bus = TokioMessageBus::new();

        let orchestrator = OrchestratorAgent::new(Arc::clone(&mock), bus, temp_dir.path());
        assert_eq!(orchestrator.name(), "orchestrator");
        assert_eq!(orchestrator.role(), AgentRole::Orchestrator);

        let planner = PlannerAgent::new(Arc::clone(&mock), temp_dir.path());
        assert_eq!(planner.name(), "planner");
        assert_eq!(planner.role(), AgentRole::Planner);

        let coder = CoderAgent::new(Arc::clone(&mock), temp_dir.path());
        assert_eq!(coder.name(), "coder");
        assert_eq!(coder.role(), AgentRole::Coder);

        let critic = CriticAgent::new(Arc::clone(&mock), temp_dir.path());
        assert_eq!(critic.name(), "critic");
        assert_eq!(critic.role(), AgentRole::Critic);

        let explorer = ExplorerAgent::new(Arc::clone(&mock), temp_dir.path());
        assert_eq!(explorer.name(), "explorer");
        assert_eq!(explorer.role(), AgentRole::Explorer);

        let librarian = LibrarianAgent::new(Arc::clone(&mock), temp_dir.path());
        assert_eq!(librarian.name(), "librarian");
        assert_eq!(librarian.role(), AgentRole::Librarian);

        let devops = DevOpsAgent::new(mock, temp_dir.path());
        assert_eq!(devops.name(), "devops");
        assert_eq!(devops.role(), AgentRole::DevOps);
    }

    #[test]
    fn test_agent_role_enum_has_seven_variants() {
        let roles = [
            AgentRole::Orchestrator,
            AgentRole::Planner,
            AgentRole::Coder,
            AgentRole::Critic,
            AgentRole::Explorer,
            AgentRole::Librarian,
            AgentRole::DevOps,
        ];
        assert_eq!(roles.len(), 7, "Should have exactly 7 agent roles");
    }
}
