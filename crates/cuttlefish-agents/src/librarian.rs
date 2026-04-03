//! Librarian agent: finds and retrieves documentation for libraries, APIs, and frameworks.

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

/// The librarian agent that retrieves documentation.
pub struct LibrarianAgent {
    provider: Arc<dyn ModelProvider>,
    prompt_registry: Arc<PromptRegistry>,
}

impl LibrarianAgent {
    /// Create a new librarian agent with the given provider and prompts directory.
    pub fn new(provider: Arc<dyn ModelProvider>, prompts_dir: impl Into<PathBuf>) -> Self {
        Self {
            provider,
            prompt_registry: Arc::new(PromptRegistry::new(prompts_dir)),
        }
    }

    /// Create a new librarian agent with a shared prompt registry.
    pub fn with_registry(
        provider: Arc<dyn ModelProvider>,
        prompt_registry: Arc<PromptRegistry>,
    ) -> Self {
        Self {
            provider,
            prompt_registry,
        }
    }

    /// Default system prompt for the librarian agent.
    const DEFAULT_SYSTEM_PROMPT: &'static str = "You are the Librarian agent for Cuttlefish. You find and retrieve documentation for libraries, APIs, and frameworks. Search the web, fetch documentation pages, and provide accurate, up-to-date information about external dependencies.";
}

#[async_trait]
impl Agent for LibrarianAgent {
    fn name(&self) -> &str {
        "librarian"
    }

    fn role(&self) -> AgentRole {
        AgentRole::Librarian
    }

    async fn execute(
        &self,
        ctx: &mut AgentContext,
        input: &str,
    ) -> Result<AgentOutput, AgentError> {
        info!(
            "Librarian retrieving docs: {}",
            &input[..input.len().min(80)]
        );

        let system_prompt = self
            .prompt_registry
            .load("librarian")
            .map(|p| p.body)
            .unwrap_or_else(|_| Self::DEFAULT_SYSTEM_PROMPT.to_string());

        let request = CompletionRequest {
            messages: ctx.messages.clone(),
            max_tokens: Some(4096),
            temperature: Some(0.1),
            system: Some(system_prompt),
        };

        let response = self
            .provider
            .complete(request)
            .await
            .map_err(|e| AgentError(format!("Librarian model call failed: {e}")))?;

        ctx.messages.push(Message {
            role: MessageRole::Assistant,
            content: response.content.clone(),
        });

        Ok(AgentOutput {
            content: response.content,
            files_changed: vec![],
            commands_run: vec![],
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
category: quick
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
            messages: vec![Message {
                role: MessageRole::User,
                content: "Find documentation".to_string(),
            }],
        }
    }

    #[tokio::test]
    async fn test_librarian_executes_and_returns_output() {
        let temp_dir = TempDir::new().expect("temp dir");
        create_test_prompt(temp_dir.path(), "librarian", "You are the librarian agent.");

        let mock = MockModelProvider::new("test");
        mock.add_response("Documentation for tokio: https://docs.rs/tokio");
        let agent = LibrarianAgent::new(Arc::new(mock), temp_dir.path());
        let mut ctx = test_ctx();
        let out = agent
            .execute(&mut ctx, "Find documentation")
            .await
            .expect("exec");
        assert!(out.success);
        assert!(out.content.contains("tokio"));
    }

    #[tokio::test]
    async fn test_librarian_uses_default_prompt_when_missing() {
        let temp_dir = TempDir::new().expect("temp dir");

        let mock = MockModelProvider::new("test");
        mock.add_response("Here is the documentation.");
        let agent = LibrarianAgent::new(Arc::new(mock), temp_dir.path());
        let mut ctx = test_ctx();
        let out = agent.execute(&mut ctx, "Find docs").await.expect("exec");
        assert!(out.success);
    }

    #[tokio::test]
    async fn test_librarian_adds_response_to_context() {
        let temp_dir = TempDir::new().expect("temp dir");
        create_test_prompt(temp_dir.path(), "librarian", "You are the librarian agent.");

        let mock = MockModelProvider::new("test");
        mock.add_response("Done retrieving docs.");
        let agent = LibrarianAgent::new(Arc::new(mock), temp_dir.path());
        let mut ctx = test_ctx();
        agent.execute(&mut ctx, "Find docs").await.expect("exec");
        assert_eq!(ctx.messages.len(), 2);
        assert!(matches!(ctx.messages[1].role, MessageRole::Assistant));
    }

    #[test]
    fn test_librarian_role() {
        let temp_dir = TempDir::new().expect("temp dir");
        create_test_prompt(temp_dir.path(), "librarian", "You are the librarian agent.");

        let mock = Arc::new(MockModelProvider::default());
        let agent = LibrarianAgent::new(mock, temp_dir.path());
        assert_eq!(agent.name(), "librarian");
        assert_eq!(agent.role(), AgentRole::Librarian);
    }
}
