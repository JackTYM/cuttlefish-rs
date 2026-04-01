//! Workflow engine: Orchestrator→Coder→Critic loop.

use cuttlefish_core::{
    error::AgentError,
    traits::{
        agent::{Agent, AgentContext},
        provider::ModelProvider,
    },
};
use std::sync::Arc;
use tracing::{info, warn};
use uuid::Uuid;

use crate::{
    bus::TokioMessageBus,
    coder::CoderAgent,
    critic::CriticAgent,
    orchestrator::OrchestratorAgent,
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
}

/// Drives the complete Orchestrator→Coder→Critic workflow.
pub struct WorkflowEngine {
    orchestrator: OrchestratorAgent,
    coder: CoderAgent,
    critic: CriticAgent,
    max_iterations: usize,
}

impl WorkflowEngine {
    /// Create a workflow engine with all agents backed by the given provider.
    pub fn new(provider: Arc<dyn ModelProvider>, bus: TokioMessageBus) -> Self {
        Self {
            orchestrator: OrchestratorAgent::new(Arc::clone(&provider), bus),
            coder: CoderAgent::new(Arc::clone(&provider)),
            critic: CriticAgent::new(provider),
            max_iterations: MAX_CODER_CRITIC_ITERATIONS,
        }
    }

    /// Create a workflow engine with a custom max iterations limit.
    pub fn with_max_iterations(
        provider: Arc<dyn ModelProvider>,
        bus: TokioMessageBus,
        max_iterations: usize,
    ) -> Self {
        let mut engine = Self::new(provider, bus);
        engine.max_iterations = max_iterations;
        engine
    }

    /// Execute the full workflow for a given task.
    ///
    /// Workflow:
    /// 1. Orchestrator receives input and creates a task plan
    /// 2. Coder executes each task
    /// 3. Critic reviews the output
    /// 4. If rejected: Coder tries again with feedback (max `max_iterations` times)
    /// 5. If approved or max iterations reached: proceed to next task
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
            messages: vec![],
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
                input.to_string()
            } else {
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
            coder_ctx.messages.push(cuttlefish_core::traits::provider::Message {
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
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cuttlefish_providers::mock::MockModelProvider;

    fn make_engine(mock: MockModelProvider) -> WorkflowEngine {
        let bus = TokioMessageBus::new();
        WorkflowEngine::new(Arc::new(mock), bus)
    }

    #[tokio::test]
    async fn test_workflow_succeeds_with_approve() {
        let mock = MockModelProvider::new("test");
        // Orchestrator response
        mock.add_response(r#"{"tasks": [{"id":"1","description":"Create app","agent":"coder"}]}"#);
        // Coder response
        mock.add_response("Created index.js with hello world");
        // Critic approves
        mock.add_response(r#"{"verdict": "approve", "issues": [], "summary": "Good code"}"#);

        let engine = make_engine(mock);
        let result = engine
            .execute(Uuid::new_v4(), "Create a hello world app")
            .await
            .expect("exec");
        assert!(result.success);
        assert_eq!(result.iterations, 1);
    }

    #[tokio::test]
    async fn test_workflow_retries_on_reject() {
        let mock = MockModelProvider::new("test");
        // Orchestrator
        mock.add_response(r#"{"tasks": [{"id":"1","description":"Create app","agent":"coder"}]}"#);
        // Coder first attempt
        mock.add_response("Created index.js (buggy)");
        // Critic rejects
        mock.add_response(r#"{"verdict": "reject", "issues": [{"file":"index.js","message":"Bug"}], "summary": "Has bugs"}"#);
        // Coder second attempt
        mock.add_response("Created index.js (fixed)");
        // Critic approves
        mock.add_response(r#"{"verdict": "approve", "issues": [], "summary": "Fixed"}"#);

        let engine = make_engine(mock);
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
        // Orchestrator
        mock.add_response("{}");
        // Add reject cycle for each iteration (max 5)
        for _ in 0..5 {
            mock.add_response("Code attempt");
            mock.add_response(r#"{"verdict": "reject", "issues": [], "summary": "Still broken"}"#);
        }

        let engine = WorkflowEngine::with_max_iterations(
            Arc::new(mock),
            TokioMessageBus::new(),
            2, // test with 2 max iterations
        );
        let result = engine
            .execute(Uuid::new_v4(), "Task")
            .await
            .expect("exec");
        assert!(!result.success);
        assert_eq!(result.iterations, 2);
    }
}
