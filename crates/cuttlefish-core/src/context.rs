//! Context management for agent conversations.
//!
//! The [`ContextManager`] assembles conversation history for agent invocations,
//! respecting token budgets and triggering summarization when needed.

use crate::error::{CuttlefishError, DatabaseError};
use crate::traits::provider::{CompletionRequest, Message, MessageRole, ModelProvider};
use std::sync::Arc;
use tracing::{debug, info};
use uuid::Uuid;

/// Default maximum token budget for context.
pub const DEFAULT_MAX_TOKENS: usize = 30_000;

/// Default threshold (number of messages) to trigger summarization.
pub const DEFAULT_SUMMARIZE_THRESHOLD: i64 = 50;

const SUMMARY_PROMPT: &str = "Summarize the following conversation, preserving: \
    key decisions made, code changes implemented, current project state, \
    and any outstanding tasks. Be concise but complete.\n\n";

/// Configuration for context management.
#[derive(Debug, Clone)]
pub struct ContextConfig {
    /// Maximum token budget for assembled context.
    pub max_tokens: usize,
    /// Number of messages that triggers automatic summarization.
    pub summarize_threshold: i64,
    /// Whether summarization is enabled.
    pub summarization_enabled: bool,
}

impl Default for ContextConfig {
    fn default() -> Self {
        Self {
            max_tokens: DEFAULT_MAX_TOKENS,
            summarize_threshold: DEFAULT_SUMMARIZE_THRESHOLD,
            summarization_enabled: true,
        }
    }
}

/// Manages conversation context for agent invocations.
///
/// Responsibilities:
/// - Assemble recent messages within token budget
/// - Include summaries of older messages
/// - Trigger automatic summarization when threshold exceeded
pub struct ContextManager {
    db: Arc<cuttlefish_db::Database>,
    provider: Arc<dyn ModelProvider>,
    config: ContextConfig,
}

impl ContextManager {
    /// Create a new `ContextManager`.
    pub fn new(
        db: Arc<cuttlefish_db::Database>,
        provider: Arc<dyn ModelProvider>,
        config: ContextConfig,
    ) -> Self {
        Self {
            db,
            provider,
            config,
        }
    }

    /// Create with default configuration.
    pub fn with_defaults(
        db: Arc<cuttlefish_db::Database>,
        provider: Arc<dyn ModelProvider>,
    ) -> Self {
        Self::new(db, provider, ContextConfig::default())
    }

    /// Build context messages for an agent, respecting token budget.
    ///
    /// Returns recent messages in chronological order, fitting within `max_tokens`.
    /// If summarization has been triggered, includes the summary as the first message.
    pub async fn build_context(
        &self,
        project_id: &str,
        max_tokens: Option<usize>,
    ) -> Result<Vec<Message>, CuttlefishError> {
        let budget = max_tokens.unwrap_or(self.config.max_tokens);

        if self.config.summarization_enabled {
            let count = self
                .db
                .get_message_count(project_id)
                .await
                .map_err(|e| DatabaseError(e.to_string()))?;

            if count >= self.config.summarize_threshold {
                info!(
                    "Message count {} exceeds threshold {}, triggering summarization for project {}",
                    count, self.config.summarize_threshold, project_id
                );
                self.trigger_summarization(project_id).await?;
            }
        }

        let raw_messages = self
            .db
            .get_recent_messages_chrono(project_id, 200)
            .await
            .map_err(|e| DatabaseError(e.to_string()))?;

        let mut messages = Vec::new();
        let mut total_tokens: usize = 0;

        for msg in &raw_messages {
            let token_estimate = msg.content.len() / 4 + 1;
            if total_tokens + token_estimate > budget && !messages.is_empty() {
                debug!(
                    "Token budget {} reached, stopping at {} messages",
                    budget,
                    messages.len()
                );
                break;
            }
            total_tokens += token_estimate;

            let role = match msg.role.as_str() {
                "user" => MessageRole::User,
                "assistant" => MessageRole::Assistant,
                _ => MessageRole::System,
            };
            messages.push(Message {
                role,
                content: msg.content.clone(),
            });
        }

        Ok(messages)
    }

    /// Trigger summarization of older messages for a project.
    ///
    /// Summarizes messages beyond the recent window, archiving them and
    /// inserting a summary message in their place.
    pub async fn trigger_summarization(&self, project_id: &str) -> Result<(), CuttlefishError> {
        // Keep the most recent half of threshold messages, summarize the rest
        let keep_count = self.config.summarize_threshold / 2;

        let cutoff_ts = self
            .db
            .get_nth_recent_message_timestamp(project_id, keep_count)
            .await
            .map_err(|e| DatabaseError(e.to_string()))?;

        let Some(cutoff) = cutoff_ts else {
            debug!(
                "Not enough messages to summarize for project {}",
                project_id
            );
            return Ok(());
        };

        let old_messages = self
            .db
            .get_recent_messages(project_id, self.config.summarize_threshold)
            .await
            .map_err(|e| DatabaseError(e.to_string()))?;

        if old_messages.is_empty() {
            return Ok(());
        }

        let conversation_text = old_messages
            .iter()
            .filter(|m| m.archived == 0)
            .map(|m| format!("{}: {}", m.role, m.content))
            .collect::<Vec<_>>()
            .join("\n");

        let summary_request = CompletionRequest {
            messages: vec![Message {
                role: MessageRole::User,
                content: format!("{SUMMARY_PROMPT}{conversation_text}"),
            }],
            max_tokens: Some(512),
            temperature: Some(0.1),
            system: Some(
                "You are a helpful assistant that creates concise conversation summaries."
                    .to_string(),
            ),
        };

        let summary_response = self.provider.complete(summary_request).await?;

        let summary_id = Uuid::new_v4().to_string();
        self.db
            .archive_and_summarize(project_id, &cutoff, &summary_id, &summary_response.content)
            .await
            .map_err(|e| DatabaseError(e.to_string()))?;

        info!(
            "Summarization complete for project {}, archived messages before {}",
            project_id, cutoff
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::ProviderError;
    use crate::traits::provider::{CompletionResponse, StreamChunk};
    use futures::StreamExt;
    use futures::stream::{self, BoxStream};
    use std::sync::Arc;

    /// Inline mock: cannot use cuttlefish-providers (would be circular dep).
    struct TestProvider;

    #[async_trait::async_trait]
    impl ModelProvider for TestProvider {
        fn name(&self) -> &str {
            "test"
        }

        async fn complete(
            &self,
            _req: CompletionRequest,
        ) -> Result<CompletionResponse, ProviderError> {
            Ok(CompletionResponse {
                content: "Summary of conversation.".to_string(),
                input_tokens: 10,
                output_tokens: 5,
                model: "test".to_string(),
                tool_calls: vec![],
            })
        }

        fn stream<'a>(
            &'a self,
            _req: CompletionRequest,
        ) -> BoxStream<'a, Result<StreamChunk, ProviderError>> {
            stream::iter(vec![Ok(StreamChunk::Text("test".to_string()))]).boxed()
        }

        async fn count_tokens(&self, text: &str) -> Result<usize, ProviderError> {
            Ok(text.len() / 4 + 1)
        }
    }

    async fn test_setup() -> (
        Arc<cuttlefish_db::Database>,
        Arc<TestProvider>,
        String,
        tempfile::NamedTempFile,
    ) {
        let tmp = tempfile::NamedTempFile::new().expect("temp file");
        let db = cuttlefish_db::Database::open(tmp.path()).await.expect("db");
        let project_id = Uuid::new_v4().to_string();
        db.create_project(
            &project_id,
            &format!("ctx-test-{}", &project_id[..8]),
            "Context test project",
            None,
        )
        .await
        .expect("create project");
        (Arc::new(db), Arc::new(TestProvider), project_id, tmp)
    }

    #[tokio::test]
    async fn test_build_context_empty() {
        let (db, provider, project_id, _tmp) = test_setup().await;
        let mgr = ContextManager::with_defaults(db, provider);
        let ctx = mgr
            .build_context(&project_id, None)
            .await
            .expect("build context");
        assert!(ctx.is_empty());
    }

    #[tokio::test]
    async fn test_build_context_within_budget() {
        let (db, provider, project_id, _tmp) = test_setup().await;

        for i in 0..5 {
            let id = Uuid::new_v4().to_string();
            db.insert_message(
                &id,
                &project_id,
                "user",
                &format!("message number {i}"),
                None,
                10,
            )
            .await
            .expect("insert");
        }

        let mgr = ContextManager::with_defaults(Arc::clone(&db), provider);
        let ctx = mgr
            .build_context(&project_id, Some(100_000))
            .await
            .expect("build context");
        assert_eq!(ctx.len(), 5);
    }

    #[tokio::test]
    async fn test_build_context_respects_token_budget() {
        let (db, provider, project_id, _tmp) = test_setup().await;

        for i in 0..10 {
            let id = Uuid::new_v4().to_string();
            let content = "a".repeat(80);
            db.insert_message(
                &id,
                &project_id,
                "user",
                &format!("{content} {i}"),
                None,
                20,
            )
            .await
            .expect("insert");
        }

        let mgr = ContextManager::with_defaults(Arc::clone(&db), provider);
        let ctx = mgr
            .build_context(&project_id, Some(100))
            .await
            .expect("build context");
        assert!(
            ctx.len() < 10,
            "Expected fewer than 10 messages with tight budget, got {}",
            ctx.len()
        );
        assert!(
            !ctx.is_empty(),
            "Should include at least the first message even if over budget"
        );
    }

    #[tokio::test]
    async fn test_summarization_triggered_at_threshold() {
        let (db, provider, project_id, _tmp) = test_setup().await;

        // Explicit timestamps needed: SQLite datetime('now') has 1s precision,
        // so rapid inserts share the same timestamp, breaking archive_and_summarize's
        // `created_at < cutoff` partitioning.
        for i in 0..55i64 {
            let id = Uuid::new_v4().to_string();
            let ts = format!("2026-01-01 00:{:02}:{:02}", i / 60, i % 60);
            sqlx::query(
                "INSERT INTO conversations (id, project_id, role, content, token_count, created_at) \
                 VALUES (?, ?, 'user', ?, 10, ?)",
            )
            .bind(&id)
            .bind(&project_id)
            .bind(format!("message {i}"))
            .bind(&ts)
            .execute(db.pool())
            .await
            .expect("insert with timestamp");
        }

        let config = ContextConfig {
            max_tokens: DEFAULT_MAX_TOKENS,
            summarize_threshold: 50,
            summarization_enabled: true,
        };
        let mgr = ContextManager::new(Arc::clone(&db), provider, config);

        let _ = mgr
            .build_context(&project_id, None)
            .await
            .expect("build context");

        let count_after = db.get_message_count(&project_id).await.expect("count");
        assert!(
            count_after < 55,
            "Expected fewer than 55 messages after summarization, got {count_after}"
        );
    }
}
