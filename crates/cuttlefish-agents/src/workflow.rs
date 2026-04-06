//! Workflow engine: Orchestrator→Coder→Critic loop with optional Planner, Explorer, Librarian, DevOps.

use cuttlefish_core::{
    error::AgentError,
    traits::{
        agent::{Agent, AgentContext, AgentRole},
        provider::ModelProvider,
    },
};
use std::path::PathBuf;
use std::sync::Arc;
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::{
    bus::TokioMessageBus, coder::CoderAgent, critic::CriticAgent, devops::DevOpsAgent,
    explorer::ExplorerAgent, librarian::LibrarianAgent, orchestrator::OrchestratorAgent,
    planner::PlannerAgent, prompt_registry::PromptRegistry,
};

/// Maximum Coder↔Critic iterations per task (Metis guardrail).
pub const MAX_CODER_CRITIC_ITERATIONS: usize = 5;

/// Result of a complete workflow run.
#[derive(Debug)]
pub struct WorkflowResult {
    /// Final output content.
    pub content: String,
    /// Whether the workflow completed successfully.
    pub success: bool,
    /// Number of Coder→Critic iterations performed.
    pub iterations: usize,
    /// The final review verdict.
    pub final_verdict: Option<String>,
    /// Whether a planning phase was executed.
    pub planning_executed: bool,
}

/// Configuration for optional agents in the workflow.
#[derive(Default)]
pub struct WorkflowConfig {
    /// Enable the Planner agent to run before coding.
    pub enable_planner: bool,
    /// Enable the Explorer agent for codebase searches.
    pub enable_explorer: bool,
    /// Enable the Librarian agent for documentation retrieval.
    pub enable_librarian: bool,
    /// Enable the DevOps agent for build/deploy operations.
    pub enable_devops: bool,
}

/// Drives the complete workflow with all 7 agent types.
///
/// Core loop: Orchestrator → (optional Planner) → Coder ↔ Critic
/// Support agents: Explorer, Librarian, DevOps (dispatched on demand)
pub struct WorkflowEngine {
    orchestrator: OrchestratorAgent,
    coder: CoderAgent,
    critic: CriticAgent,
    planner: Option<PlannerAgent>,
    explorer: Option<ExplorerAgent>,
    librarian: Option<LibrarianAgent>,
    devops: Option<DevOpsAgent>,
    max_iterations: usize,
    config: WorkflowConfig,
}

impl WorkflowEngine {
    /// Create a workflow engine with core agents (Orchestrator, Coder, Critic).
    pub fn new(
        provider: Arc<dyn ModelProvider>,
        bus: TokioMessageBus,
        prompts_dir: impl Into<PathBuf>,
    ) -> Self {
        let registry = Arc::new(PromptRegistry::new(prompts_dir));
        Self {
            orchestrator: OrchestratorAgent::with_registry(
                Arc::clone(&provider),
                bus,
                Arc::clone(&registry),
            ),
            coder: CoderAgent::with_registry(Arc::clone(&provider), Arc::clone(&registry)),
            critic: CriticAgent::with_registry(provider, registry),
            planner: None,
            explorer: None,
            librarian: None,
            devops: None,
            max_iterations: MAX_CODER_CRITIC_ITERATIONS,
            config: WorkflowConfig::default(),
        }
    }

    /// Create a workflow engine with a custom max iterations limit.
    pub fn with_max_iterations(
        provider: Arc<dyn ModelProvider>,
        bus: TokioMessageBus,
        prompts_dir: impl Into<PathBuf>,
        max_iterations: usize,
    ) -> Self {
        let mut engine = Self::new(provider, bus, prompts_dir);
        engine.max_iterations = max_iterations;
        engine
    }

    /// Create a workflow engine with all 7 agents enabled.
    pub fn with_all_agents(
        provider: Arc<dyn ModelProvider>,
        bus: TokioMessageBus,
        prompts_dir: impl Into<PathBuf>,
    ) -> Self {
        let registry = Arc::new(PromptRegistry::new(prompts_dir));
        Self {
            orchestrator: OrchestratorAgent::with_registry(
                Arc::clone(&provider),
                bus,
                Arc::clone(&registry),
            ),
            coder: CoderAgent::with_registry(Arc::clone(&provider), Arc::clone(&registry)),
            critic: CriticAgent::with_registry(Arc::clone(&provider), Arc::clone(&registry)),
            planner: Some(PlannerAgent::with_registry(
                Arc::clone(&provider),
                Arc::clone(&registry),
            )),
            explorer: Some(ExplorerAgent::with_registry(
                Arc::clone(&provider),
                Arc::clone(&registry),
            )),
            librarian: Some(LibrarianAgent::with_registry(
                Arc::clone(&provider),
                Arc::clone(&registry),
            )),
            devops: Some(DevOpsAgent::with_registry(provider, registry)),
            max_iterations: MAX_CODER_CRITIC_ITERATIONS,
            config: WorkflowConfig {
                enable_planner: true,
                enable_explorer: true,
                enable_librarian: true,
                enable_devops: true,
            },
        }
    }

    /// Create a workflow engine with custom configuration.
    pub fn with_config(
        provider: Arc<dyn ModelProvider>,
        bus: TokioMessageBus,
        prompts_dir: impl Into<PathBuf>,
        config: WorkflowConfig,
    ) -> Self {
        let registry = Arc::new(PromptRegistry::new(prompts_dir));
        Self {
            orchestrator: OrchestratorAgent::with_registry(
                Arc::clone(&provider),
                bus,
                Arc::clone(&registry),
            ),
            coder: CoderAgent::with_registry(Arc::clone(&provider), Arc::clone(&registry)),
            critic: CriticAgent::with_registry(Arc::clone(&provider), Arc::clone(&registry)),
            planner: if config.enable_planner {
                Some(PlannerAgent::with_registry(
                    Arc::clone(&provider),
                    Arc::clone(&registry),
                ))
            } else {
                None
            },
            explorer: if config.enable_explorer {
                Some(ExplorerAgent::with_registry(
                    Arc::clone(&provider),
                    Arc::clone(&registry),
                ))
            } else {
                None
            },
            librarian: if config.enable_librarian {
                Some(LibrarianAgent::with_registry(
                    Arc::clone(&provider),
                    Arc::clone(&registry),
                ))
            } else {
                None
            },
            devops: if config.enable_devops {
                Some(DevOpsAgent::with_registry(provider, registry))
            } else {
                None
            },
            max_iterations: MAX_CODER_CRITIC_ITERATIONS,
            config,
        }
    }

    /// Dispatch to a specific agent by role.
    pub async fn dispatch_to_agent(
        &self,
        role: AgentRole,
        ctx: &mut AgentContext,
        input: &str,
    ) -> Result<cuttlefish_core::traits::agent::AgentOutput, AgentError> {
        match role {
            AgentRole::Orchestrator => self.orchestrator.execute(ctx, input).await,
            AgentRole::Planner => {
                self.planner
                    .as_ref()
                    .ok_or_else(|| AgentError("Planner agent not enabled".to_string()))?
                    .execute(ctx, input)
                    .await
            }
            AgentRole::Coder => self.coder.execute(ctx, input).await,
            AgentRole::Critic => self.critic.execute(ctx, input).await,
            AgentRole::Explorer => {
                self.explorer
                    .as_ref()
                    .ok_or_else(|| AgentError("Explorer agent not enabled".to_string()))?
                    .execute(ctx, input)
                    .await
            }
            AgentRole::Librarian => {
                self.librarian
                    .as_ref()
                    .ok_or_else(|| AgentError("Librarian agent not enabled".to_string()))?
                    .execute(ctx, input)
                    .await
            }
            AgentRole::DevOps => {
                self.devops
                    .as_ref()
                    .ok_or_else(|| AgentError("DevOps agent not enabled".to_string()))?
                    .execute(ctx, input)
                    .await
            }
        }
    }

    /// Check if a specific agent role is enabled.
    pub fn is_agent_enabled(&self, role: AgentRole) -> bool {
        match role {
            AgentRole::Orchestrator | AgentRole::Coder | AgentRole::Critic => true,
            AgentRole::Planner => self.planner.is_some(),
            AgentRole::Explorer => self.explorer.is_some(),
            AgentRole::Librarian => self.librarian.is_some(),
            AgentRole::DevOps => self.devops.is_some(),
        }
    }

    /// Execute the full workflow for a given task.
    ///
    /// Workflow:
    /// 1. Orchestrator receives input and creates a task plan
    /// 2. (Optional) Planner creates detailed implementation plan
    /// 3. Coder executes each task
    /// 4. Critic reviews the output
    /// 5. If rejected: Coder tries again with feedback (max `max_iterations` times)
    /// 6. If approved or max iterations reached: proceed to next task
    pub async fn execute(
        &self,
        project_id: Uuid,
        input: &str,
    ) -> Result<WorkflowResult, AgentError> {
        info!(
            "Starting workflow for project {}: {}",
            project_id,
            &input[..input.len().min(60)]
        );

        // Phase 1: Orchestrate — create the plan
        let mut orch_ctx = AgentContext {
            invocation_id: Uuid::new_v4(),
            project_id,
            working_dir: std::path::PathBuf::from("/workspace"),
            available_tools: vec![],
            messages: vec![cuttlefish_core::traits::provider::Message {
                role: cuttlefish_core::traits::provider::MessageRole::User,
                content: input.to_string(),
            }],
        };

        let orch_output = self.orchestrator.execute(&mut orch_ctx, input).await?;
        info!(
            "Orchestrator output: {} tasks dispatched",
            orch_output
                .metadata
                .get("tasks_dispatched")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
        );

        // Phase 1.5: (Optional) Planning — create detailed implementation plan
        let mut planning_executed = false;
        let planning_output = if self.config.enable_planner {
            if let Some(ref planner) = self.planner {
                debug!("Running Planner agent for detailed planning");
                let mut planner_ctx = AgentContext {
                    invocation_id: Uuid::new_v4(),
                    project_id,
                    working_dir: std::path::PathBuf::from("/workspace"),
                    available_tools: vec![],
                    messages: vec![cuttlefish_core::traits::provider::Message {
                        role: cuttlefish_core::traits::provider::MessageRole::User,
                        content: format!(
                            "Create a detailed implementation plan for: {}\n\nOrchestrator context: {}",
                            input, orch_output.content
                        ),
                    }],
                };
                let plan_output = planner.execute(&mut planner_ctx, input).await?;
                planning_executed = true;
                info!("Planner created implementation plan");
                Some(plan_output.content)
            } else {
                None
            }
        } else {
            None
        };

        // Phase 2: Coder→Critic loop
        let mut coder_ctx = AgentContext {
            invocation_id: Uuid::new_v4(),
            project_id,
            working_dir: std::path::PathBuf::from("/workspace"),
            available_tools: vec![
                "read_file".to_string(),
                "write_file".to_string(),
                "execute_command".to_string(),
            ],
            messages: vec![],
        };

        let mut final_content = orch_output.content.clone();
        let mut iterations = 0;
        let mut final_verdict = None;

        for iteration in 0..self.max_iterations {
            iterations = iteration + 1;
            let coder_input = if iteration == 0 {
                let initial_input = if let Some(ref plan) = planning_output {
                    format!("{}\n\nImplementation Plan:\n{}", input, plan)
                } else {
                    input.to_string()
                };
                // Add initial user message to context (required for Bedrock)
                coder_ctx
                    .messages
                    .push(cuttlefish_core::traits::provider::Message {
                        role: cuttlefish_core::traits::provider::MessageRole::User,
                        content: initial_input.clone(),
                    });
                initial_input
            } else {
                // On subsequent iterations, critic feedback was already added as user message
                format!("{}\n\nPrevious critic feedback: {}", input, final_content)
            };

            // Coder executes
            let coder_output = self.coder.execute(&mut coder_ctx, &coder_input).await?;
            final_content = coder_output.content.clone();

            // Critic reviews
            let mut critic_ctx = AgentContext {
                invocation_id: Uuid::new_v4(),
                project_id,
                working_dir: std::path::PathBuf::from("/workspace"),
                available_tools: vec![],
                messages: coder_ctx.messages.clone(),
            };

            let critic_output = self.critic.execute(&mut critic_ctx, &final_content).await?;

            let verdict = critic_output
                .metadata
                .get("verdict")
                .and_then(|v| v.as_str())
                .unwrap_or("approve")
                .to_string();
            final_verdict = Some(verdict.clone());

            if critic_output.success {
                // Critic approved
                info!("Critic approved after {} iteration(s)", iterations);
                return Ok(WorkflowResult {
                    content: final_content,
                    success: true,
                    iterations,
                    final_verdict,
                    planning_executed,
                });
            }

            let summary = critic_output
                .metadata
                .get("summary")
                .and_then(|v| v.as_str())
                .unwrap_or("no summary");
            warn!(
                "Critic rejected (iteration {}/{}): {}",
                iterations, self.max_iterations, summary
            );

            // Prepare feedback for next iteration
            final_content = critic_output.content;

            // Add critic feedback to coder context
            coder_ctx
                .messages
                .push(cuttlefish_core::traits::provider::Message {
                    role: cuttlefish_core::traits::provider::MessageRole::User,
                    content: format!("Critic feedback: {}", final_content),
                });
        }

        // Max iterations reached — return last result
        warn!(
            "Max iterations ({}) reached without critic approval",
            self.max_iterations
        );
        Ok(WorkflowResult {
            content: final_content,
            success: false,
            iterations,
            final_verdict,
            planning_executed,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cuttlefish_providers::mock::MockModelProvider;
    use std::fs;
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

    fn make_engine(mock: MockModelProvider) -> (WorkflowEngine, TempDir) {
        let temp_dir = TempDir::new().expect("temp dir");
        create_test_prompts(temp_dir.path());
        let bus = TokioMessageBus::new();
        let engine = WorkflowEngine::new(Arc::new(mock), bus, temp_dir.path());
        (engine, temp_dir)
    }

    #[tokio::test]
    async fn test_workflow_succeeds_with_approve() {
        let mock = MockModelProvider::new("test");
        mock.add_response(r#"{"tasks": [{"id":"1","description":"Create app","agent":"coder"}]}"#);
        mock.add_response("Created index.js with hello world");
        mock.add_response(r#"{"verdict": "approve", "issues": [], "summary": "Good code"}"#);

        let (engine, _temp_dir) = make_engine(mock);
        let result = engine
            .execute(Uuid::new_v4(), "Create a hello world app")
            .await
            .expect("exec");
        assert!(result.success);
        assert_eq!(result.iterations, 1);
        assert!(!result.planning_executed);
    }

    #[tokio::test]
    async fn test_workflow_retries_on_reject() {
        let mock = MockModelProvider::new("test");
        mock.add_response(r#"{"tasks": [{"id":"1","description":"Create app","agent":"coder"}]}"#);
        mock.add_response("Created index.js (buggy)");
        mock.add_response(r#"{"verdict": "reject", "issues": [{"file":"index.js","message":"Bug"}], "summary": "Has bugs"}"#);
        mock.add_response("Created index.js (fixed)");
        mock.add_response(r#"{"verdict": "approve", "issues": [], "summary": "Fixed"}"#);

        let (engine, _temp_dir) = make_engine(mock);
        let result = engine
            .execute(Uuid::new_v4(), "Create app")
            .await
            .expect("exec");
        assert!(result.success);
        assert_eq!(result.iterations, 2);
    }

    #[tokio::test]
    async fn test_workflow_stops_at_max_iterations() {
        let mock = MockModelProvider::new("test");
        mock.add_response("{}");
        for _ in 0..5 {
            mock.add_response("Code attempt");
            mock.add_response(r#"{"verdict": "reject", "issues": [], "summary": "Still broken"}"#);
        }

        let temp_dir = TempDir::new().expect("temp dir");
        create_test_prompts(temp_dir.path());
        let engine = WorkflowEngine::with_max_iterations(
            Arc::new(mock),
            TokioMessageBus::new(),
            temp_dir.path(),
            2,
        );
        let result = engine.execute(Uuid::new_v4(), "Task").await.expect("exec");
        assert!(!result.success);
        assert_eq!(result.iterations, 2);
    }

    #[tokio::test]
    async fn test_workflow_with_planner_enabled() {
        let mock = MockModelProvider::new("test");
        mock.add_response(r#"{"tasks": [{"id":"1","description":"Create app","agent":"coder"}]}"#);
        mock.add_response("Step 1: Create module\nStep 2: Add tests");
        mock.add_response("Created module with tests");
        mock.add_response(r#"{"verdict": "approve", "issues": [], "summary": "Good"}"#);

        let temp_dir = TempDir::new().expect("temp dir");
        create_test_prompts(temp_dir.path());
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
            .execute(Uuid::new_v4(), "Create a module")
            .await
            .expect("exec");
        assert!(result.success);
        assert!(result.planning_executed);
    }

    #[tokio::test]
    async fn test_workflow_with_all_agents() {
        let mock = MockModelProvider::new("test");
        mock.add_response(r#"{"tasks": [{"id":"1","description":"Create app","agent":"coder"}]}"#);
        mock.add_response("Detailed plan here");
        mock.add_response("Code output");
        mock.add_response(r#"{"verdict": "approve", "issues": [], "summary": "OK"}"#);

        let temp_dir = TempDir::new().expect("temp dir");
        create_test_prompts(temp_dir.path());
        let engine = WorkflowEngine::with_all_agents(
            Arc::new(mock),
            TokioMessageBus::new(),
            temp_dir.path(),
        );
        let result = engine
            .execute(Uuid::new_v4(), "Build something")
            .await
            .expect("exec");
        assert!(result.success);
        assert!(result.planning_executed);
    }

    #[tokio::test]
    async fn test_dispatch_to_agent_coder() {
        let mock = MockModelProvider::new("test");
        mock.add_response("Coder output");

        let temp_dir = TempDir::new().expect("temp dir");
        create_test_prompts(temp_dir.path());
        let engine = WorkflowEngine::new(Arc::new(mock), TokioMessageBus::new(), temp_dir.path());

        let mut ctx = AgentContext {
            invocation_id: Uuid::new_v4(),
            project_id: Uuid::new_v4(),
            working_dir: std::path::PathBuf::from("/workspace"),
            available_tools: vec![],
            messages: vec![],
        };
        let output = engine
            .dispatch_to_agent(AgentRole::Coder, &mut ctx, "Write code")
            .await
            .expect("dispatch");
        assert!(output.success);
    }

    #[tokio::test]
    async fn test_dispatch_to_disabled_agent_fails() {
        let mock = MockModelProvider::new("test");
        let temp_dir = TempDir::new().expect("temp dir");
        create_test_prompts(temp_dir.path());
        let engine = WorkflowEngine::new(Arc::new(mock), TokioMessageBus::new(), temp_dir.path());

        let mut ctx = AgentContext {
            invocation_id: Uuid::new_v4(),
            project_id: Uuid::new_v4(),
            working_dir: std::path::PathBuf::from("/workspace"),
            available_tools: vec![],
            messages: vec![],
        };
        let result = engine
            .dispatch_to_agent(AgentRole::Planner, &mut ctx, "Plan")
            .await;
        assert!(result.is_err());
        assert!(
            result
                .expect_err("should fail")
                .to_string()
                .contains("not enabled")
        );
    }

    #[test]
    fn test_is_agent_enabled() {
        let mock = MockModelProvider::new("test");
        let temp_dir = TempDir::new().expect("temp dir");
        create_test_prompts(temp_dir.path());

        let engine = WorkflowEngine::new(
            Arc::new(mock.clone()),
            TokioMessageBus::new(),
            temp_dir.path(),
        );
        assert!(engine.is_agent_enabled(AgentRole::Orchestrator));
        assert!(engine.is_agent_enabled(AgentRole::Coder));
        assert!(engine.is_agent_enabled(AgentRole::Critic));
        assert!(!engine.is_agent_enabled(AgentRole::Planner));
        assert!(!engine.is_agent_enabled(AgentRole::Explorer));
        assert!(!engine.is_agent_enabled(AgentRole::Librarian));
        assert!(!engine.is_agent_enabled(AgentRole::DevOps));

        let temp_dir2 = TempDir::new().expect("temp dir");
        create_test_prompts(temp_dir2.path());
        let engine_all = WorkflowEngine::with_all_agents(
            Arc::new(mock),
            TokioMessageBus::new(),
            temp_dir2.path(),
        );
        assert!(engine_all.is_agent_enabled(AgentRole::Planner));
        assert!(engine_all.is_agent_enabled(AgentRole::Explorer));
        assert!(engine_all.is_agent_enabled(AgentRole::Librarian));
        assert!(engine_all.is_agent_enabled(AgentRole::DevOps));
    }
}
