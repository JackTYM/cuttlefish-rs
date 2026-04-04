//! Tracked provider wrapper for async usage logging.

use async_trait::async_trait;
use cuttlefish_core::traits::provider::{
    CompletionRequest, CompletionResponse, ModelProvider, ProviderResult, StreamChunk,
};
use futures::StreamExt;
use futures::stream::BoxStream;
use sqlx::SqlitePool;
use std::sync::Arc;
use std::time::Instant;
use tracing::{debug, warn};

/// Context for usage tracking (project, session, user).
#[derive(Debug, Clone, Default)]
pub struct UsageContext {
    /// Project ID for this request.
    pub project_id: Option<String>,
    /// Session ID for this request.
    pub session_id: Option<String>,
    /// User ID for this request.
    pub user_id: Option<String>,
}

impl UsageContext {
    /// Create a new usage context.
    pub fn new(
        project_id: Option<String>,
        session_id: Option<String>,
        user_id: Option<String>,
    ) -> Self {
        Self {
            project_id,
            session_id,
            user_id,
        }
    }
}

/// A provider wrapper that logs usage asynchronously (fire-and-forget).
///
/// Wraps any `ModelProvider` and logs token usage to the database
/// without blocking the response.
pub struct TrackedProvider<P: ModelProvider> {
    inner: P,
    pool: Arc<SqlitePool>,
    context: UsageContext,
}

impl<P: ModelProvider> TrackedProvider<P> {
    /// Create a new tracked provider.
    pub fn new(inner: P, pool: Arc<SqlitePool>, context: UsageContext) -> Self {
        Self {
            inner,
            pool,
            context,
        }
    }

    /// Create with just a pool (empty context).
    pub fn with_pool(inner: P, pool: Arc<SqlitePool>) -> Self {
        Self::new(inner, pool, UsageContext::default())
    }

    /// Set the usage context.
    pub fn with_context(mut self, context: UsageContext) -> Self {
        self.context = context;
        self
    }

    fn log_usage_async(
        &self,
        input_tokens: u32,
        output_tokens: u32,
        latency_ms: i64,
        request_type: &str,
        success: bool,
        error_type: Option<String>,
    ) {
        let pool = Arc::clone(&self.pool);
        let provider = self.inner.name().to_string();
        let model = self.inner.name().to_string();
        let project_id = self.context.project_id.clone();
        let session_id = self.context.session_id.clone();
        let user_id = self.context.user_id.clone();
        let request_type = request_type.to_string();

        tokio::spawn(async move {
            let usage = cuttlefish_db::usage::ApiUsage {
                id: uuid::Uuid::new_v4().to_string(),
                project_id,
                session_id,
                user_id,
                provider,
                model,
                input_tokens: input_tokens as i64,
                output_tokens: output_tokens as i64,
                request_type,
                latency_ms: Some(latency_ms),
                success: if success { 1 } else { 0 },
                error_type,
                created_at: chrono::Utc::now().to_rfc3339(),
            };

            if let Err(e) = cuttlefish_db::usage::insert_usage(&pool, &usage).await {
                warn!("Failed to log usage: {}", e);
            } else {
                debug!(
                    "Logged usage: {} input, {} output tokens",
                    input_tokens, output_tokens
                );
            }
        });
    }
}

#[async_trait]
impl<P: ModelProvider + 'static> ModelProvider for TrackedProvider<P> {
    fn name(&self) -> &str {
        self.inner.name()
    }

    async fn complete(&self, request: CompletionRequest) -> ProviderResult<CompletionResponse> {
        let start = Instant::now();
        let result = self.inner.complete(request).await;
        let latency_ms = start.elapsed().as_millis() as i64;

        match &result {
            Ok(response) => {
                self.log_usage_async(
                    response.input_tokens,
                    response.output_tokens,
                    latency_ms,
                    "complete",
                    true,
                    None,
                );
            }
            Err(e) => {
                self.log_usage_async(0, 0, latency_ms, "complete", false, Some(e.to_string()));
            }
        }

        result
    }

    fn stream<'a>(
        &'a self,
        request: CompletionRequest,
    ) -> BoxStream<'a, ProviderResult<StreamChunk>> {
        let start = Instant::now();
        let inner_stream = self.inner.stream(request);

        let pool = Arc::clone(&self.pool);
        let provider = self.inner.name().to_string();
        let project_id = self.context.project_id.clone();
        let session_id = self.context.session_id.clone();
        let user_id = self.context.user_id.clone();

        let tracked_stream = inner_stream.then(move |chunk_result| {
            let pool = Arc::clone(&pool);
            let provider = provider.clone();
            let project_id = project_id.clone();
            let session_id = session_id.clone();
            let user_id = user_id.clone();
            let latency_ms = start.elapsed().as_millis() as i64;

            async move {
                if let Ok(StreamChunk::Usage {
                    input_tokens,
                    output_tokens,
                }) = &chunk_result
                {
                    let usage = cuttlefish_db::usage::ApiUsage {
                        id: uuid::Uuid::new_v4().to_string(),
                        project_id,
                        session_id,
                        user_id,
                        provider,
                        model: String::new(),
                        input_tokens: *input_tokens as i64,
                        output_tokens: *output_tokens as i64,
                        request_type: "stream".to_string(),
                        latency_ms: Some(latency_ms),
                        success: 1,
                        error_type: None,
                        created_at: chrono::Utc::now().to_rfc3339(),
                    };

                    tokio::spawn(async move {
                        if let Err(e) = cuttlefish_db::usage::insert_usage(&pool, &usage).await {
                            warn!("Failed to log stream usage: {}", e);
                        }
                    });
                }
                chunk_result
            }
        });

        Box::pin(tracked_stream)
    }

    async fn count_tokens(&self, text: &str) -> ProviderResult<usize> {
        self.inner.count_tokens(text).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock::MockModelProvider;
    use cuttlefish_core::traits::provider::{Message, MessageRole};
    use futures::StreamExt;
    use tempfile::TempDir;

    async fn test_pool() -> (Arc<SqlitePool>, TempDir) {
        let dir = TempDir::new().expect("temp dir");
        let db_path = dir.path().join("test.db");
        let url = format!("sqlite://{}?mode=rwc", db_path.to_string_lossy());
        let pool = SqlitePool::connect(&url).await.expect("connect");

        cuttlefish_db::usage::run_usage_migrations(&pool)
            .await
            .expect("migrations");

        (Arc::new(pool), dir)
    }

    fn test_request() -> CompletionRequest {
        CompletionRequest {
            messages: vec![Message {
                role: MessageRole::User,
                content: "Hello".to_string(),
            }],
            max_tokens: Some(100),
            temperature: None,
            system: None,
        }
    }

    #[tokio::test]
    async fn test_tracked_complete_logs_usage() {
        let (pool, _dir) = test_pool().await;
        let mock = MockModelProvider::new("test-provider");
        mock.add_response("Test response");

        let context = UsageContext::new(
            Some("proj-123".to_string()),
            Some("sess-456".to_string()),
            Some("user-789".to_string()),
        );

        let tracked = TrackedProvider::new(mock, Arc::clone(&pool), context);

        let response = tracked.complete(test_request()).await.expect("complete");
        assert_eq!(response.content, "Test response");

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let records = sqlx::query_as::<_, cuttlefish_db::usage::ApiUsage>(
            "SELECT * FROM api_usage WHERE project_id = ?",
        )
        .bind("proj-123")
        .fetch_all(pool.as_ref())
        .await
        .expect("query");

        assert_eq!(records.len(), 1);
        assert_eq!(records[0].provider, "test-provider");
        assert_eq!(records[0].request_type, "complete");
        assert_eq!(records[0].success, 1);
    }

    #[tokio::test]
    async fn test_tracked_stream_logs_usage_on_final_chunk() {
        let (pool, _dir) = test_pool().await;
        let mock = MockModelProvider::new("stream-provider");
        mock.add_response("Streamed content");

        let tracked = TrackedProvider::with_pool(mock, Arc::clone(&pool)).with_context(
            UsageContext::new(Some("proj-stream".to_string()), None, None),
        );

        let mut stream = tracked.stream(test_request());

        while let Some(chunk) = stream.next().await {
            let _ = chunk.expect("chunk ok");
        }

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let records = sqlx::query_as::<_, cuttlefish_db::usage::ApiUsage>(
            "SELECT * FROM api_usage WHERE project_id = ?",
        )
        .bind("proj-stream")
        .fetch_all(pool.as_ref())
        .await
        .expect("query");

        assert_eq!(records.len(), 1);
        assert_eq!(records[0].request_type, "stream");
    }

    #[tokio::test]
    async fn test_tracked_provider_name_passthrough() {
        let (pool, _dir) = test_pool().await;
        let mock = MockModelProvider::new("my-provider");
        let tracked = TrackedProvider::with_pool(mock, pool);

        assert_eq!(tracked.name(), "my-provider");
    }

    #[tokio::test]
    async fn test_tracked_count_tokens_passthrough() {
        let (pool, _dir) = test_pool().await;
        let mock = MockModelProvider::new("token-counter");
        let tracked = TrackedProvider::with_pool(mock, pool);

        let count = tracked.count_tokens("Hello world").await.expect("count");
        assert!(count > 0);
    }

    #[tokio::test]
    async fn test_logging_does_not_block_response() {
        let (pool, _dir) = test_pool().await;
        let mock = MockModelProvider::new("fast-provider");
        mock.add_response("Quick response");

        let tracked = TrackedProvider::with_pool(mock, pool);

        let start = Instant::now();
        let response = tracked.complete(test_request()).await.expect("complete");
        let elapsed = start.elapsed();

        assert_eq!(response.content, "Quick response");
        assert!(
            elapsed.as_millis() < 50,
            "Response should be fast, logging is async"
        );
    }
}
