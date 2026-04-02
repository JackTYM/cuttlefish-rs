//! Orchestrator agent: routes tasks and manages the workflow lifecycle.

use async_trait::async_trait;
use cuttlefish_core::{
    error::AgentError,
    traits::{
        agent::{Agent, AgentContext, AgentOutput, AgentRole},
        bus::BusMessage,
        provider::{CompletionRequest, Message, MessageRole, ModelProvider},
    },
};
use serde_json::json;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::{debug, info};
use uuid::Uuid;

use crate::bus::TokioMessageBus;
use crate::prompt_registry::PromptRegistry;
use cuttlefish_core::traits::bus::MessageBus;

/// An orchestrator task in the execution plan.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PlannedTask {
    /// Unique task identifier.
    pub id: String,
    /// Task description for the agent.
    pub description: String,
    /// Which agent should handle this task (coder or critic).
    pub agent: String,
    /// Current status of this task.
    #[serde(default)]
    pub status: TaskStatus,
}

/// Status of a planned task.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TaskStatus {
    /// Not yet started.
    #[default]
    Pending,
    /// Currently being executed.
    Running,
    /// Completed successfully.
    Completed,
    /// Failed to complete.
    Failed,
}

/// The orchestrator agent that plans and delegates work.
pub struct OrchestratorAgent {
    provider: Arc<dyn ModelProvider>,
    bus: TokioMessageBus,
    prompt_registry: Arc<PromptRegistry>,
}

impl OrchestratorAgent {
    /// Create a new orchestrator with the given model provider, message bus, and prompts directory.
    ///
    /// # Arguments
    ///
    /// * `provider` - The model provider for completions.
    /// * `bus` - The message bus for dispatching tasks.
    /// * `prompts_dir` - Path to the directory containing prompt `.md` files.
    pub fn new(
        provider: Arc<dyn ModelProvider>,
        bus: TokioMessageBus,
        prompts_dir: impl Into<PathBuf>,
    ) -> Self {
        Self {
            provider,
            bus,
            prompt_registry: Arc::new(PromptRegistry::new(prompts_dir)),
        }
    }

    /// Create a new orchestrator with a shared prompt registry.
    ///
    /// Use this when multiple agents share the same registry.
    pub fn with_registry(
        provider: Arc<dyn ModelProvider>,
        bus: TokioMessageBus,
        prompt_registry: Arc<PromptRegistry>,
    ) -> Self {
        Self {
            provider,
            bus,
            prompt_registry,
        }
    }

    /// Build a planning prompt for the given user input.
    fn build_planning_prompt(&self, input: &str) -> Result<CompletionRequest, AgentError> {
        let prompt = self
            .prompt_registry
            .load("orchestrator")
            .map_err(|e| AgentError(format!("Failed to load orchestrator prompt: {e}")))?;

        Ok(CompletionRequest {
            messages: vec![Message {
                role: MessageRole::User,
                content: input.to_string(),
            }],
            max_tokens: Some(2048),
            temperature: Some(0.2),
            system: Some(prompt.body),
        })
    }

    /// Publish a task to the appropriate agent topic.
    async fn dispatch_task(&self, task: &PlannedTask, project_id: &Uuid) -> Result<(), AgentError> {
        let topic = format!("agent.{}.input", task.agent);
        let msg = BusMessage::new(
            topic,
            json!({
                "task_id": task.id,
                "project_id": project_id.to_string(),
                "description": task.description,
            }),
        );
        debug!("Dispatching task {} to {}", task.id, task.agent);
        self.bus.publish(msg).await
    }
}

#[async_trait]
impl Agent for OrchestratorAgent {
    fn name(&self) -> &str {
        "orchestrator"
    }

    fn role(&self) -> AgentRole {
        AgentRole::Orchestrator
    }

    async fn execute(
        &self,
        ctx: &mut AgentContext,
        input: &str,
    ) -> Result<AgentOutput, AgentError> {
        info!(
            "Orchestrator executing for project {}: {}",
            ctx.project_id,
            &input[..input.len().min(80)]
        );

        // Build task plan using model
        let request = self.build_planning_prompt(input)?;
        let response = self
            .provider
            .complete(request)
            .await
            .map_err(|e| AgentError(format!("Planning failed: {e}")))?;

        // Try to parse task plan from response
        let tasks: Vec<PlannedTask> = serde_json::from_str::<serde_json::Value>(&response.content)
            .ok()
            .and_then(|v| v["tasks"].as_array().cloned())
            .map(|arr| {
                arr.iter()
                    .filter_map(|t| serde_json::from_value(t.clone()).ok())
                    .collect()
            })
            .unwrap_or_else(|| {
                // Fallback: create single coder task
                vec![PlannedTask {
                    id: "1".to_string(),
                    description: input.to_string(),
                    agent: "coder".to_string(),
                    status: TaskStatus::Pending,
                }]
            });

        info!("Orchestrator created {} tasks", tasks.len());

        // Dispatch tasks to agents
        for task in &tasks {
            self.dispatch_task(task, &ctx.project_id).await?;
        }

        // Add assistant response to context
        ctx.messages.push(Message {
            role: MessageRole::Assistant,
            content: response.content.clone(),
        });

        Ok(AgentOutput {
            content: response.content,
            files_changed: vec![],
            commands_run: vec![],
            success: true,
            metadata: json!({ "tasks_dispatched": tasks.len() }),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cuttlefish_providers::mock::MockModelProvider;
    use std::fs;
    use tempfile::TempDir;
    use uuid::Uuid;

    fn create_test_prompt(dir: &std::path::Path, name: &str, body: &str) {
        let content = format!(
            r#"---
name: {name}
description: Test agent
tools: []
category: deep
---

{body}"#
        );
        fs::write(dir.join(format!("{name}.md")), content).expect("write test prompt");
    }

    fn test_ctx() -> AgentContext {
        AgentContext {
            invocation_id: Uuid::new_v4(),
            project_id: Uuid::new_v4(),
            working_dir: std::path::PathBuf::from("/workspace"),
            available_tools: vec![],
            messages: vec![],
        }
    }

    #[tokio::test]
    async fn test_orchestrator_dispatches_on_execute() {
        let temp_dir = TempDir::new().expect("temp dir");
        create_test_prompt(
            temp_dir.path(),
            "orchestrator",
            "You are the orchestrator. Output JSON task plans.",
        );

        let mock = MockModelProvider::new("test");
        mock.add_response(
            r#"{"tasks": [{"id": "1", "description": "Create hello.js", "agent": "coder"}]}"#,
        );
        let bus = TokioMessageBus::new();
        let _rx = bus.subscribe("agent.coder.input").await.expect("subscribe");
        let agent = OrchestratorAgent::new(Arc::new(mock), bus, temp_dir.path());
        let mut ctx = test_ctx();
        let out = agent
            .execute(&mut ctx, "Create a hello world app")
            .await
            .expect("exec");
        assert!(out.success);
        assert_eq!(out.metadata["tasks_dispatched"], 1);
    }

    #[tokio::test]
    async fn test_orchestrator_fallback_on_bad_json() {
        let temp_dir = TempDir::new().expect("temp dir");
        create_test_prompt(
            temp_dir.path(),
            "orchestrator",
            "You are the orchestrator. Output JSON task plans.",
        );

        let mock = MockModelProvider::new("test");
        mock.add_response("I will create a Node.js app for you.");
        let bus = TokioMessageBus::new();
        let _rx = bus.subscribe("agent.coder.input").await.expect("sub");
        let agent = OrchestratorAgent::new(Arc::new(mock), bus, temp_dir.path());
        let mut ctx = test_ctx();
        let out = agent
            .execute(&mut ctx, "Build something")
            .await
            .expect("exec");
        assert!(out.success); // Falls back to single coder task
    }

    #[tokio::test]
    async fn test_orchestrator_fails_on_missing_prompt() {
        let temp_dir = TempDir::new().expect("temp dir");
        // No prompt file created

        let mock = MockModelProvider::new("test");
        let bus = TokioMessageBus::new();
        let agent = OrchestratorAgent::new(Arc::new(mock), bus, temp_dir.path());
        let mut ctx = test_ctx();
        let result = agent.execute(&mut ctx, "Do something").await;
        assert!(result.is_err());
        let err = result.expect_err("should fail");
        assert!(err.to_string().contains("Failed to load orchestrator prompt"));
    }

    #[test]
    fn test_task_status_default_is_pending() {
        let status = TaskStatus::default();
        assert_eq!(status, TaskStatus::Pending);
    }
}
