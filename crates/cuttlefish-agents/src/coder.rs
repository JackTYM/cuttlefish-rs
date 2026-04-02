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
use std::path::PathBuf;
use std::sync::Arc;
use tracing::info;

use crate::prompt_registry::PromptRegistry;

/// The coder agent that writes and executes code.
pub struct CoderAgent {
    provider: Arc<dyn ModelProvider>,
    prompt_registry: Arc<PromptRegistry>,
}

impl CoderAgent {
    /// Create a new coder agent with the given provider and prompts directory.
    pub fn new(provider: Arc<dyn ModelProvider>, prompts_dir: impl Into<PathBuf>) -> Self {
        Self {
            provider,
            prompt_registry: Arc::new(PromptRegistry::new(prompts_dir)),
        }
    }

    /// Create a new coder agent with a shared prompt registry.
    pub fn with_registry(
        provider: Arc<dyn ModelProvider>,
        prompt_registry: Arc<PromptRegistry>,
    ) -> Self {
        Self {
            provider,
            prompt_registry,
        }
    }

    fn parse_output_metadata(content: &str) -> (Vec<std::path::PathBuf>, Vec<String>) {
        let files_changed = Vec::new();
        let commands_run = Vec::new();
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

        let prompt = self
            .prompt_registry
            .load("coder")
            .map_err(|e| AgentError(format!("Failed to load coder prompt: {e}")))?;

        let request = CompletionRequest {
            messages: ctx.messages.clone(),
            max_tokens: Some(4096),
            temperature: Some(0.1),
            system: Some(prompt.body),
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
    async fn test_coder_executes_and_returns_output() {
        let temp_dir = TempDir::new().expect("temp dir");
        create_test_prompt(temp_dir.path(), "coder", "You are the coder agent.");

        let mock = MockModelProvider::new("test");
        mock.add_response("Created index.js with console.log('hello')");
        let agent = CoderAgent::new(Arc::new(mock), temp_dir.path());
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
        let temp_dir = TempDir::new().expect("temp dir");
        create_test_prompt(temp_dir.path(), "coder", "You are the coder agent.");

        let mock = MockModelProvider::new("test");
        mock.add_response("Done.");
        let agent = CoderAgent::new(Arc::new(mock), temp_dir.path());
        let mut ctx = test_ctx();
        agent.execute(&mut ctx, "Do something").await.expect("exec");
        assert_eq!(ctx.messages.len(), 1);
        assert!(matches!(ctx.messages[0].role, MessageRole::Assistant));
    }

    #[tokio::test]
    async fn test_coder_fails_on_missing_prompt() {
        let temp_dir = TempDir::new().expect("temp dir");

        let mock = MockModelProvider::new("test");
        let agent = CoderAgent::new(Arc::new(mock), temp_dir.path());
        let mut ctx = test_ctx();
        let result = agent.execute(&mut ctx, "Do something").await;
        assert!(result.is_err());
        let err = result.expect_err("should fail");
        assert!(err.to_string().contains("Failed to load coder prompt"));
    }

    #[test]
    fn test_coder_role() {
        let temp_dir = TempDir::new().expect("temp dir");
        create_test_prompt(temp_dir.path(), "coder", "You are the coder agent.");

        let mock = Arc::new(MockModelProvider::default());
        let agent = CoderAgent::new(mock, temp_dir.path());
        assert_eq!(agent.name(), "coder");
        assert_eq!(agent.role(), AgentRole::Coder);
    }
}
