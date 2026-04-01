//! Coder agent: writes code, runs builds, and executes tests in the sandbox.

use async_trait::async_trait;
use cuttlefish_core::{
    error::AgentError,
    traits::{
        agent::{Agent, AgentContext, AgentOutput, AgentRole},
        provider::{CompletionRequest, Message, MessageRole, ModelProvider},
    },
};
use serde_json::json;
use std::sync::Arc;
use tracing::info;

const CODER_SYSTEM_PROMPT: &str = "\
You are the Coder agent for Cuttlefish. You write code, create files, run builds, \
and execute tests inside a Docker sandbox. Use the available tools to: \
read files (read_file), write files (write_file), run commands (execute_command), \
and list directories (list_directory). \
Always create working, tested code. Run the build/test command after writing code \
to verify it works.";

/// The coder agent that writes and executes code.
pub struct CoderAgent {
    provider: Arc<dyn ModelProvider>,
}

impl CoderAgent {
    /// Create a new coder agent.
    pub fn new(provider: Arc<dyn ModelProvider>) -> Self {
        Self { provider }
    }

    /// Output produced by the coder (files changed + commands run).
    fn parse_output_metadata(content: &str) -> (Vec<std::path::PathBuf>, Vec<String>) {
        // Simple heuristic: look for file paths and commands mentioned in output
        let files_changed = Vec::new();
        let commands_run = Vec::new();
        // In a real implementation this would parse tool call history
        let _ = content;
        (files_changed, commands_run)
    }
}

#[async_trait]
impl Agent for CoderAgent {
    fn name(&self) -> &str {
        "coder"
    }

    fn role(&self) -> AgentRole {
        AgentRole::Coder
    }

    async fn execute(
        &self,
        ctx: &mut AgentContext,
        input: &str,
    ) -> Result<AgentOutput, AgentError> {
        info!("Coder executing task: {}", &input[..input.len().min(80)]);

        let request = CompletionRequest {
            messages: ctx.messages.clone(),
            max_tokens: Some(4096),
            temperature: Some(0.1),
            system: Some(CODER_SYSTEM_PROMPT.to_string()),
        };

        let response = self
            .provider
            .complete(request)
            .await
            .map_err(|e| AgentError(format!("Coder model call failed: {e}")))?;

        ctx.messages.push(Message {
            role: MessageRole::Assistant,
            content: response.content.clone(),
        });

        let (files_changed, commands_run) = Self::parse_output_metadata(&response.content);

        Ok(AgentOutput {
            content: response.content,
            files_changed,
            commands_run,
            success: true,
            metadata: json!({ "model": response.model, "tokens_used": response.output_tokens }),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cuttlefish_providers::mock::MockModelProvider;
    use uuid::Uuid;

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
    async fn test_coder_executes_and_returns_output() {
        let mock = MockModelProvider::new("test");
        mock.add_response("Created index.js with console.log('hello')");
        let agent = CoderAgent::new(Arc::new(mock));
        let mut ctx = test_ctx();
        ctx.messages.push(Message {
            role: MessageRole::User,
            content: "Create hello world".to_string(),
        });
        let out = agent
            .execute(&mut ctx, "Create hello world")
            .await
            .expect("exec");
        assert!(out.success);
        assert!(out.content.contains("index.js") || out.content.contains("hello"));
    }

    #[tokio::test]
    async fn test_coder_adds_response_to_context() {
        let mock = MockModelProvider::new("test");
        mock.add_response("Done.");
        let agent = CoderAgent::new(Arc::new(mock));
        let mut ctx = test_ctx();
        agent.execute(&mut ctx, "Do something").await.expect("exec");
        assert_eq!(ctx.messages.len(), 1); // assistant message added
        assert!(matches!(ctx.messages[0].role, MessageRole::Assistant));
    }

    #[test]
    fn test_coder_role() {
        let mock = Arc::new(MockModelProvider::default());
        let agent = CoderAgent::new(mock);
        assert_eq!(agent.name(), "coder");
        assert_eq!(agent.role(), AgentRole::Coder);
    }
}
