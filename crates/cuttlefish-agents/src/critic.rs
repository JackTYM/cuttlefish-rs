//! Critic agent: reviews code changes and provides structured feedback.

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

const CRITIC_SYSTEM_PROMPT: &str = "\
You are the Critic agent for Cuttlefish. You review code changes for quality, \
correctness, and adherence to project standards. \
Review the provided code and output a JSON verdict: \
{\"verdict\": \"approve\" or \"reject\", \"issues\": [{\"file\": \"...\", \"line\": N, \"message\": \"...\"}], \
\"summary\": \"brief review summary\"}. \
Only reject if there are genuine bugs, security issues, or major quality problems. \
Approve working code even if imperfect.";

/// The result of a critic review.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ReviewResult {
    /// Verdict: "approve" or "reject".
    pub verdict: ReviewVerdict,
    /// List of issues found.
    #[serde(default)]
    pub issues: Vec<ReviewIssue>,
    /// Summary of the review.
    pub summary: String,
}

/// The review verdict.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReviewVerdict {
    /// Code is acceptable.
    Approve,
    /// Code needs changes.
    Reject,
}

/// A specific issue found during review.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ReviewIssue {
    /// File containing the issue.
    pub file: String,
    /// Line number (if applicable).
    pub line: Option<u32>,
    /// Issue description.
    pub message: String,
}

/// The critic agent that reviews code.
pub struct CriticAgent {
    provider: Arc<dyn ModelProvider>,
    /// Maximum iterations for Coder→Critic before forcing approval.
    pub max_iterations: usize,
}

impl CriticAgent {
    /// Create a new critic agent.
    pub fn new(provider: Arc<dyn ModelProvider>) -> Self {
        Self {
            provider,
            max_iterations: 5,
        }
    }

    /// Create with a custom max iterations limit.
    pub fn with_max_iterations(provider: Arc<dyn ModelProvider>, max_iterations: usize) -> Self {
        Self {
            provider,
            max_iterations,
        }
    }

    /// Parse the review result from model output.
    fn parse_review(content: &str) -> ReviewResult {
        serde_json::from_str::<serde_json::Value>(content)
            .ok()
            .and_then(|v| serde_json::from_value(v).ok())
            .unwrap_or_else(|| {
                // If can't parse JSON, check for keywords
                let verdict = if content.to_lowercase().contains("reject") {
                    ReviewVerdict::Reject
                } else {
                    ReviewVerdict::Approve
                };
                ReviewResult {
                    verdict,
                    issues: vec![],
                    summary: content.chars().take(200).collect(),
                }
            })
    }
}

#[async_trait]
impl Agent for CriticAgent {
    fn name(&self) -> &str {
        "critic"
    }

    fn role(&self) -> AgentRole {
        AgentRole::Critic
    }

    async fn execute(
        &self,
        ctx: &mut AgentContext,
        input: &str,
    ) -> Result<AgentOutput, AgentError> {
        info!("Critic reviewing: {}", &input[..input.len().min(80)]);

        let request = CompletionRequest {
            messages: ctx.messages.clone(),
            max_tokens: Some(1024),
            temperature: Some(0.1),
            system: Some(CRITIC_SYSTEM_PROMPT.to_string()),
        };

        let response = self
            .provider
            .complete(request)
            .await
            .map_err(|e| AgentError(format!("Critic model call failed: {e}")))?;

        let review = Self::parse_review(&response.content);
        let approved = review.verdict == ReviewVerdict::Approve;

        ctx.messages.push(Message {
            role: MessageRole::Assistant,
            content: response.content.clone(),
        });

        Ok(AgentOutput {
            content: response.content,
            files_changed: vec![],
            commands_run: vec![],
            success: approved,
            metadata: json!({
                "verdict": if approved { "approve" } else { "reject" },
                "issues_count": review.issues.len(),
                "summary": review.summary,
            }),
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
            messages: vec![Message {
                role: MessageRole::User,
                content: "Review this code".to_string(),
            }],
        }
    }

    #[tokio::test]
    async fn test_critic_approves_good_code() {
        let mock = MockModelProvider::new("test");
        mock.add_response(r#"{"verdict": "approve", "issues": [], "summary": "Code looks good"}"#);
        let agent = CriticAgent::new(Arc::new(mock));
        let mut ctx = test_ctx();
        let out = agent.execute(&mut ctx, "Review code").await.expect("exec");
        assert!(out.success);
        assert_eq!(out.metadata["verdict"], "approve");
    }

    #[tokio::test]
    async fn test_critic_rejects_bad_code() {
        let mock = MockModelProvider::new("test");
        mock.add_response(
            r#"{"verdict": "reject", "issues": [{"file": "main.rs", "line": 5, "message": "SQL injection vulnerability"}], "summary": "Security issue found"}"#,
        );
        let agent = CriticAgent::new(Arc::new(mock));
        let mut ctx = test_ctx();
        let out = agent.execute(&mut ctx, "Review code").await.expect("exec");
        assert!(!out.success);
        assert_eq!(out.metadata["issues_count"], 1);
    }

    #[test]
    fn test_parse_review_from_text_fallback() {
        let result = CriticAgent::parse_review("This code has a reject issue.");
        assert_eq!(result.verdict, ReviewVerdict::Reject);
    }

    #[test]
    fn test_parse_review_approve_default() {
        let result = CriticAgent::parse_review("The code is fine.");
        assert_eq!(result.verdict, ReviewVerdict::Approve);
    }

    #[test]
    fn test_critic_role() {
        let mock = Arc::new(MockModelProvider::default());
        let agent = CriticAgent::new(mock);
        assert_eq!(agent.name(), "critic");
        assert_eq!(agent.role(), AgentRole::Critic);
    }
}
