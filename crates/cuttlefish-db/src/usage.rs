//! Usage tracking database operations for API cost monitoring.
//!
//! Provides CRUD operations for:
//! - `api_usage` - Individual API request usage records
//! - `model_pricing` - Configurable pricing per model
//! - `usage_alerts` - Spending threshold alerts

use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

/// An API usage record tracking tokens consumed by a single request.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ApiUsage {
    /// Unique identifier (UUID string).
    pub id: String,
    /// Project ID this usage belongs to (optional).
    pub project_id: Option<String>,
    /// Session ID for grouping related requests.
    pub session_id: Option<String>,
    /// User ID who made the request.
    pub user_id: Option<String>,
    /// Provider name (e.g., "anthropic", "openai").
    pub provider: String,
    /// Model name (e.g., "claude-sonnet-4-6").
    pub model: String,
    /// Number of input tokens consumed.
    pub input_tokens: i64,
    /// Number of output tokens generated.
    pub output_tokens: i64,
    /// Request type: "complete" or "stream".
    pub request_type: String,
    /// Request latency in milliseconds.
    pub latency_ms: Option<i64>,
    /// Whether the request succeeded (1) or failed (0).
    pub success: i64,
    /// Error type if the request failed.
    pub error_type: Option<String>,
    /// Timestamp when the request was made (ISO 8601 format).
    pub created_at: String,
}

/// Pricing configuration for a specific model.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ModelPricing {
    /// Unique identifier (UUID string).
    pub id: String,
    /// Provider name (e.g., "anthropic", "openai").
    pub provider: String,
    /// Model name (e.g., "claude-sonnet-4-6").
    pub model: String,
    /// Price per million input tokens in USD.
    pub input_price_per_million: f64,
    /// Price per million output tokens in USD.
    pub output_price_per_million: f64,
    /// Date from which this pricing is effective (ISO 8601 format).
    pub effective_from: String,
    /// Timestamp when this pricing was created (ISO 8601 format).
    pub created_at: String,
}

/// A spending alert configuration.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct UsageAlert {
    /// Unique identifier (UUID string).
    pub id: String,
    /// User ID who owns this alert.
    pub user_id: String,
    /// Project ID to monitor (None = all projects).
    pub project_id: Option<String>,
    /// Spending threshold in USD.
    pub threshold_usd: f64,
    /// Alert period: "daily", "weekly", or "monthly".
    pub period: String,
    /// Timestamp when the alert was last triggered.
    pub last_triggered_at: Option<String>,
    /// Whether the alert is enabled (1) or disabled (0).
    pub enabled: i64,
    /// Timestamp when the alert was created (ISO 8601 format).
    pub created_at: String,
}

/// Run usage tracking migrations.
pub async fn run_usage_migrations(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS api_usage (
    id TEXT PRIMARY KEY,
    project_id TEXT,
    session_id TEXT,
    user_id TEXT,
    provider TEXT NOT NULL,
    model TEXT NOT NULL,
    input_tokens INTEGER NOT NULL,
    output_tokens INTEGER NOT NULL,
    request_type TEXT NOT NULL,
    latency_ms INTEGER,
    success INTEGER NOT NULL DEFAULT 1,
    error_type TEXT,
    created_at TEXT NOT NULL
)"#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_usage_project ON api_usage(project_id, created_at DESC)",
    )
    .execute(pool)
    .await?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_usage_user ON api_usage(user_id, created_at DESC)")
        .execute(pool)
        .await?;

    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_usage_provider ON api_usage(provider, created_at DESC)",
    )
    .execute(pool)
    .await?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_usage_date ON api_usage(created_at)")
        .execute(pool)
        .await?;

    sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS model_pricing (
    id TEXT PRIMARY KEY,
    provider TEXT NOT NULL,
    model TEXT NOT NULL,
    input_price_per_million REAL NOT NULL,
    output_price_per_million REAL NOT NULL,
    effective_from TEXT NOT NULL,
    created_at TEXT NOT NULL,
    UNIQUE(provider, model, effective_from)
)"#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_pricing_lookup ON model_pricing(provider, model)",
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS usage_alerts (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    project_id TEXT,
    threshold_usd REAL NOT NULL,
    period TEXT NOT NULL,
    last_triggered_at TEXT,
    enabled INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL
)"#,
    )
    .execute(pool)
    .await?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_alerts_user ON usage_alerts(user_id)")
        .execute(pool)
        .await?;

    Ok(())
}

/// Insert a new API usage record.
pub async fn insert_usage(pool: &SqlitePool, usage: &ApiUsage) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"INSERT INTO api_usage (
            id, project_id, session_id, user_id, provider, model,
            input_tokens, output_tokens, request_type, latency_ms,
            success, error_type, created_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
    )
    .bind(&usage.id)
    .bind(&usage.project_id)
    .bind(&usage.session_id)
    .bind(&usage.user_id)
    .bind(&usage.provider)
    .bind(&usage.model)
    .bind(usage.input_tokens)
    .bind(usage.output_tokens)
    .bind(&usage.request_type)
    .bind(usage.latency_ms)
    .bind(usage.success)
    .bind(&usage.error_type)
    .bind(&usage.created_at)
    .execute(pool)
    .await?;
    Ok(())
}

/// Get a usage record by ID.
pub async fn get_usage(pool: &SqlitePool, id: &str) -> Result<Option<ApiUsage>, sqlx::Error> {
    sqlx::query_as::<_, ApiUsage>("SELECT * FROM api_usage WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await
}

/// Get usage records for a project within a time range.
pub async fn get_usage_by_project(
    pool: &SqlitePool,
    project_id: &str,
    from: &str,
    to: &str,
) -> Result<Vec<ApiUsage>, sqlx::Error> {
    sqlx::query_as::<_, ApiUsage>(
        "SELECT * FROM api_usage WHERE project_id = ? AND created_at >= ? AND created_at <= ? ORDER BY created_at DESC",
    )
    .bind(project_id)
    .bind(from)
    .bind(to)
    .fetch_all(pool)
    .await
}

/// Get usage records for a user within a time range.
pub async fn get_usage_by_user(
    pool: &SqlitePool,
    user_id: &str,
    from: &str,
    to: &str,
) -> Result<Vec<ApiUsage>, sqlx::Error> {
    sqlx::query_as::<_, ApiUsage>(
        "SELECT * FROM api_usage WHERE user_id = ? AND created_at >= ? AND created_at <= ? ORDER BY created_at DESC",
    )
    .bind(user_id)
    .bind(from)
    .bind(to)
    .fetch_all(pool)
    .await
}

/// Get all usage records within a time range.
pub async fn get_all_usage(
    pool: &SqlitePool,
    from: &str,
    to: &str,
) -> Result<Vec<ApiUsage>, sqlx::Error> {
    sqlx::query_as::<_, ApiUsage>(
        "SELECT * FROM api_usage WHERE created_at >= ? AND created_at <= ? ORDER BY created_at DESC",
    )
    .bind(from)
    .bind(to)
    .fetch_all(pool)
    .await
}

/// Insert or update model pricing.
pub async fn upsert_pricing(pool: &SqlitePool, pricing: &ModelPricing) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"INSERT INTO model_pricing (
            id, provider, model, input_price_per_million,
            output_price_per_million, effective_from, created_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?)
        ON CONFLICT(provider, model, effective_from) DO UPDATE SET
            input_price_per_million = excluded.input_price_per_million,
            output_price_per_million = excluded.output_price_per_million"#,
    )
    .bind(&pricing.id)
    .bind(&pricing.provider)
    .bind(&pricing.model)
    .bind(pricing.input_price_per_million)
    .bind(pricing.output_price_per_million)
    .bind(&pricing.effective_from)
    .bind(&pricing.created_at)
    .execute(pool)
    .await?;
    Ok(())
}

/// Get the current pricing for a model (most recent effective_from <= now).
pub async fn get_current_pricing(
    pool: &SqlitePool,
    provider: &str,
    model: &str,
) -> Result<Option<ModelPricing>, sqlx::Error> {
    let now = chrono::Utc::now().to_rfc3339();
    sqlx::query_as::<_, ModelPricing>(
        "SELECT * FROM model_pricing WHERE provider = ? AND model = ? AND effective_from <= ? ORDER BY effective_from DESC LIMIT 1",
    )
    .bind(provider)
    .bind(model)
    .bind(&now)
    .fetch_optional(pool)
    .await
}

/// Get all pricing records for a provider.
pub async fn get_pricing_by_provider(
    pool: &SqlitePool,
    provider: &str,
) -> Result<Vec<ModelPricing>, sqlx::Error> {
    sqlx::query_as::<_, ModelPricing>(
        "SELECT * FROM model_pricing WHERE provider = ? ORDER BY model, effective_from DESC",
    )
    .bind(provider)
    .fetch_all(pool)
    .await
}

/// Get all current pricing records.
pub async fn get_all_current_pricing(pool: &SqlitePool) -> Result<Vec<ModelPricing>, sqlx::Error> {
    // Get the most recent pricing for each provider/model combination
    sqlx::query_as::<_, ModelPricing>(
        r#"SELECT p1.* FROM model_pricing p1
        INNER JOIN (
            SELECT provider, model, MAX(effective_from) as max_effective
            FROM model_pricing
            WHERE effective_from <= datetime('now')
            GROUP BY provider, model
        ) p2 ON p1.provider = p2.provider AND p1.model = p2.model AND p1.effective_from = p2.max_effective
        ORDER BY p1.provider, p1.model"#,
    )
    .fetch_all(pool)
    .await
}

/// Create a new usage alert.
pub async fn create_alert(pool: &SqlitePool, alert: &UsageAlert) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"INSERT INTO usage_alerts (
            id, user_id, project_id, threshold_usd, period,
            last_triggered_at, enabled, created_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)"#,
    )
    .bind(&alert.id)
    .bind(&alert.user_id)
    .bind(&alert.project_id)
    .bind(alert.threshold_usd)
    .bind(&alert.period)
    .bind(&alert.last_triggered_at)
    .bind(alert.enabled)
    .bind(&alert.created_at)
    .execute(pool)
    .await?;
    Ok(())
}

/// Get an alert by ID.
pub async fn get_alert(pool: &SqlitePool, id: &str) -> Result<Option<UsageAlert>, sqlx::Error> {
    sqlx::query_as::<_, UsageAlert>("SELECT * FROM usage_alerts WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await
}

/// Get all alerts for a user.
pub async fn get_alerts_by_user(
    pool: &SqlitePool,
    user_id: &str,
) -> Result<Vec<UsageAlert>, sqlx::Error> {
    sqlx::query_as::<_, UsageAlert>(
        "SELECT * FROM usage_alerts WHERE user_id = ? ORDER BY created_at DESC",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
}

/// Get all enabled alerts.
pub async fn get_enabled_alerts(pool: &SqlitePool) -> Result<Vec<UsageAlert>, sqlx::Error> {
    sqlx::query_as::<_, UsageAlert>(
        "SELECT * FROM usage_alerts WHERE enabled = 1 ORDER BY user_id, created_at DESC",
    )
    .fetch_all(pool)
    .await
}

/// Update alert's last_triggered_at timestamp.
pub async fn update_alert_triggered(pool: &SqlitePool, id: &str) -> Result<(), sqlx::Error> {
    let now = chrono::Utc::now().to_rfc3339();
    sqlx::query("UPDATE usage_alerts SET last_triggered_at = ? WHERE id = ?")
        .bind(&now)
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

/// Enable or disable an alert.
pub async fn set_alert_enabled(
    pool: &SqlitePool,
    id: &str,
    enabled: bool,
) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE usage_alerts SET enabled = ? WHERE id = ?")
        .bind(if enabled { 1i64 } else { 0i64 })
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

/// Delete an alert.
pub async fn delete_alert(pool: &SqlitePool, id: &str) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM usage_alerts WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

/// Daily usage aggregation result.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct DailyUsage {
    /// Date in YYYY-MM-DD format.
    pub date: String,
    /// Total input tokens for the day.
    pub input_tokens: i64,
    /// Total output tokens for the day.
    pub output_tokens: i64,
    /// Number of requests for the day.
    pub request_count: i64,
}

/// Provider usage aggregation result.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ProviderUsage {
    /// Provider name.
    pub provider: String,
    /// Total input tokens for this provider.
    pub input_tokens: i64,
    /// Total output tokens for this provider.
    pub output_tokens: i64,
    /// Number of requests for this provider.
    pub request_count: i64,
}

/// Project cost ranking result.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ProjectTokens {
    /// Project ID.
    pub project_id: String,
    /// Total input tokens.
    pub input_tokens: i64,
    /// Total output tokens.
    pub output_tokens: i64,
    /// Number of requests.
    pub request_count: i64,
}

/// Get usage aggregated by day for a time range.
pub async fn aggregated_by_day(
    pool: &SqlitePool,
    project_id: Option<&str>,
    from: &str,
    to: &str,
) -> Result<Vec<DailyUsage>, sqlx::Error> {
    if let Some(pid) = project_id {
        sqlx::query_as::<_, DailyUsage>(
            r#"SELECT 
                date(created_at) as date,
                SUM(input_tokens) as input_tokens,
                SUM(output_tokens) as output_tokens,
                COUNT(*) as request_count
            FROM api_usage 
            WHERE project_id = ? AND created_at >= ? AND created_at <= ?
            GROUP BY date(created_at)
            ORDER BY date ASC"#,
        )
        .bind(pid)
        .bind(from)
        .bind(to)
        .fetch_all(pool)
        .await
    } else {
        sqlx::query_as::<_, DailyUsage>(
            r#"SELECT 
                date(created_at) as date,
                SUM(input_tokens) as input_tokens,
                SUM(output_tokens) as output_tokens,
                COUNT(*) as request_count
            FROM api_usage 
            WHERE created_at >= ? AND created_at <= ?
            GROUP BY date(created_at)
            ORDER BY date ASC"#,
        )
        .bind(from)
        .bind(to)
        .fetch_all(pool)
        .await
    }
}

/// Get usage aggregated by provider for a time range.
pub async fn aggregated_by_provider(
    pool: &SqlitePool,
    project_id: Option<&str>,
    from: &str,
    to: &str,
) -> Result<Vec<ProviderUsage>, sqlx::Error> {
    if let Some(pid) = project_id {
        sqlx::query_as::<_, ProviderUsage>(
            r#"SELECT 
                provider,
                SUM(input_tokens) as input_tokens,
                SUM(output_tokens) as output_tokens,
                COUNT(*) as request_count
            FROM api_usage 
            WHERE project_id = ? AND created_at >= ? AND created_at <= ?
            GROUP BY provider
            ORDER BY input_tokens + output_tokens DESC"#,
        )
        .bind(pid)
        .bind(from)
        .bind(to)
        .fetch_all(pool)
        .await
    } else {
        sqlx::query_as::<_, ProviderUsage>(
            r#"SELECT 
                provider,
                SUM(input_tokens) as input_tokens,
                SUM(output_tokens) as output_tokens,
                COUNT(*) as request_count
            FROM api_usage 
            WHERE created_at >= ? AND created_at <= ?
            GROUP BY provider
            ORDER BY input_tokens + output_tokens DESC"#,
        )
        .bind(from)
        .bind(to)
        .fetch_all(pool)
        .await
    }
}

/// Get top projects by token usage for a user.
pub async fn top_projects_by_tokens(
    pool: &SqlitePool,
    user_id: &str,
    from: &str,
    to: &str,
    limit: i64,
) -> Result<Vec<ProjectTokens>, sqlx::Error> {
    sqlx::query_as::<_, ProjectTokens>(
        r#"SELECT 
            project_id,
            SUM(input_tokens) as input_tokens,
            SUM(output_tokens) as output_tokens,
            COUNT(*) as request_count
        FROM api_usage 
        WHERE user_id = ? AND project_id IS NOT NULL 
            AND created_at >= ? AND created_at <= ?
        GROUP BY project_id
        ORDER BY input_tokens + output_tokens DESC
        LIMIT ?"#,
    )
    .bind(user_id)
    .bind(from)
    .bind(to)
    .bind(limit)
    .fetch_all(pool)
    .await
}

/// Get total token usage for a user in a time range.
pub async fn get_user_totals(
    pool: &SqlitePool,
    user_id: &str,
    from: &str,
    to: &str,
) -> Result<(i64, i64, i64), sqlx::Error> {
    let row = sqlx::query(
        r#"SELECT 
            COALESCE(SUM(input_tokens), 0) as input_tokens,
            COALESCE(SUM(output_tokens), 0) as output_tokens,
            COUNT(*) as request_count
        FROM api_usage 
        WHERE user_id = ? AND created_at >= ? AND created_at <= ?"#,
    )
    .bind(user_id)
    .bind(from)
    .bind(to)
    .fetch_one(pool)
    .await?;

    use sqlx::Row;
    Ok((
        row.get::<i64, _>("input_tokens"),
        row.get::<i64, _>("output_tokens"),
        row.get::<i64, _>("request_count"),
    ))
}

/// Get total token usage for a project in a time range.
pub async fn get_project_totals(
    pool: &SqlitePool,
    project_id: &str,
    from: &str,
    to: &str,
) -> Result<(i64, i64, i64), sqlx::Error> {
    let row = sqlx::query(
        r#"SELECT 
            COALESCE(SUM(input_tokens), 0) as input_tokens,
            COALESCE(SUM(output_tokens), 0) as output_tokens,
            COUNT(*) as request_count
        FROM api_usage 
        WHERE project_id = ? AND created_at >= ? AND created_at <= ?"#,
    )
    .bind(project_id)
    .bind(from)
    .bind(to)
    .fetch_one(pool)
    .await?;

    use sqlx::Row;
    Ok((
        row.get::<i64, _>("input_tokens"),
        row.get::<i64, _>("output_tokens"),
        row.get::<i64, _>("request_count"),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    async fn test_pool() -> (SqlitePool, TempDir) {
        let dir = TempDir::new().expect("temp dir");
        let db_path = dir.path().join("test.db");
        let url = format!("sqlite://{}?mode=rwc", db_path.to_string_lossy());
        let pool = SqlitePool::connect(&url).await.expect("connect");

        // Run migrations
        run_usage_migrations(&pool).await.expect("migrations");

        (pool, dir)
    }

    #[tokio::test]
    async fn test_insert_and_get_usage() {
        let (pool, _dir) = test_pool().await;

        let usage = ApiUsage {
            id: uuid::Uuid::new_v4().to_string(),
            project_id: Some("proj-123".to_string()),
            session_id: Some("sess-456".to_string()),
            user_id: Some("user-789".to_string()),
            provider: "anthropic".to_string(),
            model: "claude-sonnet-4-6".to_string(),
            input_tokens: 1000,
            output_tokens: 500,
            request_type: "complete".to_string(),
            latency_ms: Some(1234),
            success: 1,
            error_type: None,
            created_at: chrono::Utc::now().to_rfc3339(),
        };

        insert_usage(&pool, &usage).await.expect("insert");

        let retrieved = get_usage(&pool, &usage.id)
            .await
            .expect("get")
            .expect("exists");

        assert_eq!(retrieved.id, usage.id);
        assert_eq!(retrieved.provider, "anthropic");
        assert_eq!(retrieved.model, "claude-sonnet-4-6");
        assert_eq!(retrieved.input_tokens, 1000);
        assert_eq!(retrieved.output_tokens, 500);
    }

    #[tokio::test]
    async fn test_get_usage_by_project() {
        let (pool, _dir) = test_pool().await;

        let project_id = "proj-test";
        let now = chrono::Utc::now();

        // Insert 3 usage records
        for i in 0..3 {
            let usage = ApiUsage {
                id: uuid::Uuid::new_v4().to_string(),
                project_id: Some(project_id.to_string()),
                session_id: None,
                user_id: None,
                provider: "openai".to_string(),
                model: "gpt-4o".to_string(),
                input_tokens: 100 * (i + 1),
                output_tokens: 50 * (i + 1),
                request_type: "complete".to_string(),
                latency_ms: None,
                success: 1,
                error_type: None,
                created_at: now.to_rfc3339(),
            };
            insert_usage(&pool, &usage).await.expect("insert");
        }

        let from = (now - chrono::Duration::hours(1)).to_rfc3339();
        let to = (now + chrono::Duration::hours(1)).to_rfc3339();

        let records = get_usage_by_project(&pool, project_id, &from, &to)
            .await
            .expect("get");

        assert_eq!(records.len(), 3);
    }

    #[tokio::test]
    async fn test_upsert_and_get_pricing() {
        let (pool, _dir) = test_pool().await;

        let pricing = ModelPricing {
            id: uuid::Uuid::new_v4().to_string(),
            provider: "anthropic".to_string(),
            model: "claude-opus-4-6".to_string(),
            input_price_per_million: 15.0,
            output_price_per_million: 75.0,
            effective_from: "2024-01-01T00:00:00Z".to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
        };

        upsert_pricing(&pool, &pricing).await.expect("upsert");

        let retrieved = get_current_pricing(&pool, "anthropic", "claude-opus-4-6")
            .await
            .expect("get")
            .expect("exists");

        assert_eq!(retrieved.input_price_per_million, 15.0);
        assert_eq!(retrieved.output_price_per_million, 75.0);
    }

    #[tokio::test]
    async fn test_pricing_update_on_conflict() {
        let (pool, _dir) = test_pool().await;

        let pricing1 = ModelPricing {
            id: uuid::Uuid::new_v4().to_string(),
            provider: "openai".to_string(),
            model: "gpt-4o".to_string(),
            input_price_per_million: 2.5,
            output_price_per_million: 10.0,
            effective_from: "2024-01-01T00:00:00Z".to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
        };

        upsert_pricing(&pool, &pricing1).await.expect("upsert 1");

        // Update with same provider/model/effective_from
        let pricing2 = ModelPricing {
            id: uuid::Uuid::new_v4().to_string(),
            provider: "openai".to_string(),
            model: "gpt-4o".to_string(),
            input_price_per_million: 3.0,
            output_price_per_million: 12.0,
            effective_from: "2024-01-01T00:00:00Z".to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
        };

        upsert_pricing(&pool, &pricing2).await.expect("upsert 2");

        let retrieved = get_current_pricing(&pool, "openai", "gpt-4o")
            .await
            .expect("get")
            .expect("exists");

        // Should have updated values
        assert_eq!(retrieved.input_price_per_million, 3.0);
        assert_eq!(retrieved.output_price_per_million, 12.0);
    }

    #[tokio::test]
    async fn test_alert_crud() {
        let (pool, _dir) = test_pool().await;

        let alert = UsageAlert {
            id: uuid::Uuid::new_v4().to_string(),
            user_id: "user-123".to_string(),
            project_id: Some("proj-456".to_string()),
            threshold_usd: 10.0,
            period: "daily".to_string(),
            last_triggered_at: None,
            enabled: 1,
            created_at: chrono::Utc::now().to_rfc3339(),
        };

        create_alert(&pool, &alert).await.expect("create");

        let retrieved = get_alert(&pool, &alert.id)
            .await
            .expect("get")
            .expect("exists");

        assert_eq!(retrieved.user_id, "user-123");
        assert_eq!(retrieved.threshold_usd, 10.0);
        assert_eq!(retrieved.period, "daily");
        assert_eq!(retrieved.enabled, 1);

        // Test update triggered
        update_alert_triggered(&pool, &alert.id)
            .await
            .expect("update");

        let updated = get_alert(&pool, &alert.id)
            .await
            .expect("get")
            .expect("exists");

        assert!(updated.last_triggered_at.is_some());

        // Test disable
        set_alert_enabled(&pool, &alert.id, false)
            .await
            .expect("disable");

        let disabled = get_alert(&pool, &alert.id)
            .await
            .expect("get")
            .expect("exists");

        assert_eq!(disabled.enabled, 0);

        // Test delete
        let deleted = delete_alert(&pool, &alert.id).await.expect("delete");
        assert!(deleted);

        let gone = get_alert(&pool, &alert.id).await.expect("get");
        assert!(gone.is_none());
    }

    #[tokio::test]
    async fn test_get_alerts_by_user() {
        let (pool, _dir) = test_pool().await;

        let user_id = "user-multi";

        for i in 0..3 {
            let alert = UsageAlert {
                id: uuid::Uuid::new_v4().to_string(),
                user_id: user_id.to_string(),
                project_id: None,
                threshold_usd: 10.0 * (i + 1) as f64,
                period: "weekly".to_string(),
                last_triggered_at: None,
                enabled: 1,
                created_at: chrono::Utc::now().to_rfc3339(),
            };
            create_alert(&pool, &alert).await.expect("create");
        }

        let alerts = get_alerts_by_user(&pool, user_id).await.expect("get");
        assert_eq!(alerts.len(), 3);
    }

    #[tokio::test]
    async fn test_get_enabled_alerts() {
        let (pool, _dir) = test_pool().await;

        // Create enabled alert
        let enabled_alert = UsageAlert {
            id: uuid::Uuid::new_v4().to_string(),
            user_id: "user-1".to_string(),
            project_id: None,
            threshold_usd: 50.0,
            period: "monthly".to_string(),
            last_triggered_at: None,
            enabled: 1,
            created_at: chrono::Utc::now().to_rfc3339(),
        };
        create_alert(&pool, &enabled_alert).await.expect("create");

        // Create disabled alert
        let disabled_alert = UsageAlert {
            id: uuid::Uuid::new_v4().to_string(),
            user_id: "user-2".to_string(),
            project_id: None,
            threshold_usd: 100.0,
            period: "monthly".to_string(),
            last_triggered_at: None,
            enabled: 0,
            created_at: chrono::Utc::now().to_rfc3339(),
        };
        create_alert(&pool, &disabled_alert).await.expect("create");

        let enabled = get_enabled_alerts(&pool).await.expect("get");
        assert_eq!(enabled.len(), 1);
        assert_eq!(enabled[0].id, enabled_alert.id);
    }

    #[tokio::test]
    async fn test_get_all_current_pricing() {
        let (pool, _dir) = test_pool().await;

        // Insert pricing for multiple models
        let models = [
            ("anthropic", "claude-opus-4-6", 15.0, 75.0),
            ("anthropic", "claude-sonnet-4-6", 3.0, 15.0),
            ("openai", "gpt-4o", 2.5, 10.0),
        ];

        for (provider, model, input, output) in models {
            let pricing = ModelPricing {
                id: uuid::Uuid::new_v4().to_string(),
                provider: provider.to_string(),
                model: model.to_string(),
                input_price_per_million: input,
                output_price_per_million: output,
                effective_from: "2024-01-01T00:00:00Z".to_string(),
                created_at: chrono::Utc::now().to_rfc3339(),
            };
            upsert_pricing(&pool, &pricing).await.expect("upsert");
        }

        let all_pricing = get_all_current_pricing(&pool).await.expect("get");
        assert_eq!(all_pricing.len(), 3);
    }

    #[tokio::test]
    async fn test_aggregated_by_day() {
        let (pool, _dir) = test_pool().await;

        let now = chrono::Utc::now();
        let yesterday = now - chrono::Duration::days(1);

        for (day, date) in [(now, "today"), (yesterday, "yesterday")] {
            for i in 0..3 {
                let usage = ApiUsage {
                    id: uuid::Uuid::new_v4().to_string(),
                    project_id: Some("proj-agg".to_string()),
                    session_id: None,
                    user_id: None,
                    provider: "anthropic".to_string(),
                    model: "claude-sonnet-4-6".to_string(),
                    input_tokens: 100 * (i + 1),
                    output_tokens: 50 * (i + 1),
                    request_type: "complete".to_string(),
                    latency_ms: None,
                    success: 1,
                    error_type: None,
                    created_at: day.to_rfc3339(),
                };
                insert_usage(&pool, &usage).await.expect(date);
            }
        }

        let from = (now - chrono::Duration::days(2)).to_rfc3339();
        let to = (now + chrono::Duration::hours(1)).to_rfc3339();

        let daily = aggregated_by_day(&pool, Some("proj-agg"), &from, &to)
            .await
            .expect("agg");

        assert_eq!(daily.len(), 2);
        for day in &daily {
            assert_eq!(day.input_tokens, 100 + 200 + 300);
            assert_eq!(day.output_tokens, 50 + 100 + 150);
            assert_eq!(day.request_count, 3);
        }
    }

    #[tokio::test]
    async fn test_aggregated_by_provider() {
        let (pool, _dir) = test_pool().await;

        let now = chrono::Utc::now();

        let providers = [
            ("anthropic", "claude-sonnet-4-6", 1000, 500),
            ("anthropic", "claude-opus-4-6", 2000, 1000),
            ("openai", "gpt-4o", 3000, 1500),
        ];

        for (provider, model, input, output) in providers {
            let usage = ApiUsage {
                id: uuid::Uuid::new_v4().to_string(),
                project_id: Some("proj-prov".to_string()),
                session_id: None,
                user_id: None,
                provider: provider.to_string(),
                model: model.to_string(),
                input_tokens: input,
                output_tokens: output,
                request_type: "complete".to_string(),
                latency_ms: None,
                success: 1,
                error_type: None,
                created_at: now.to_rfc3339(),
            };
            insert_usage(&pool, &usage).await.expect("insert");
        }

        let from = (now - chrono::Duration::hours(1)).to_rfc3339();
        let to = (now + chrono::Duration::hours(1)).to_rfc3339();

        let by_provider = aggregated_by_provider(&pool, Some("proj-prov"), &from, &to)
            .await
            .expect("agg");

        assert_eq!(by_provider.len(), 2);

        let anthropic = by_provider.iter().find(|p| p.provider == "anthropic");
        assert!(anthropic.is_some());
        let anthropic = anthropic.expect("anthropic");
        assert_eq!(anthropic.input_tokens, 3000);
        assert_eq!(anthropic.output_tokens, 1500);
        assert_eq!(anthropic.request_count, 2);

        let openai = by_provider.iter().find(|p| p.provider == "openai");
        assert!(openai.is_some());
        let openai = openai.expect("openai");
        assert_eq!(openai.input_tokens, 3000);
        assert_eq!(openai.output_tokens, 1500);
        assert_eq!(openai.request_count, 1);
    }

    #[tokio::test]
    async fn test_top_projects_by_tokens() {
        let (pool, _dir) = test_pool().await;

        let now = chrono::Utc::now();
        let user_id = "user-top";

        let projects = [
            ("proj-small", 100, 50),
            ("proj-medium", 500, 250),
            ("proj-large", 1000, 500),
        ];

        for (project, input, output) in projects {
            let usage = ApiUsage {
                id: uuid::Uuid::new_v4().to_string(),
                project_id: Some(project.to_string()),
                session_id: None,
                user_id: Some(user_id.to_string()),
                provider: "anthropic".to_string(),
                model: "claude-sonnet-4-6".to_string(),
                input_tokens: input,
                output_tokens: output,
                request_type: "complete".to_string(),
                latency_ms: None,
                success: 1,
                error_type: None,
                created_at: now.to_rfc3339(),
            };
            insert_usage(&pool, &usage).await.expect("insert");
        }

        let from = (now - chrono::Duration::hours(1)).to_rfc3339();
        let to = (now + chrono::Duration::hours(1)).to_rfc3339();

        let top = top_projects_by_tokens(&pool, user_id, &from, &to, 2)
            .await
            .expect("top");

        assert_eq!(top.len(), 2);
        assert_eq!(top[0].project_id, "proj-large");
        assert_eq!(top[1].project_id, "proj-medium");
    }

    #[tokio::test]
    async fn test_get_user_totals() {
        let (pool, _dir) = test_pool().await;

        let now = chrono::Utc::now();
        let user_id = "user-totals";

        for i in 0..5 {
            let usage = ApiUsage {
                id: uuid::Uuid::new_v4().to_string(),
                project_id: Some(format!("proj-{}", i)),
                session_id: None,
                user_id: Some(user_id.to_string()),
                provider: "anthropic".to_string(),
                model: "claude-sonnet-4-6".to_string(),
                input_tokens: 100,
                output_tokens: 50,
                request_type: "complete".to_string(),
                latency_ms: None,
                success: 1,
                error_type: None,
                created_at: now.to_rfc3339(),
            };
            insert_usage(&pool, &usage).await.expect("insert");
        }

        let from = (now - chrono::Duration::hours(1)).to_rfc3339();
        let to = (now + chrono::Duration::hours(1)).to_rfc3339();

        let (input, output, count) = get_user_totals(&pool, user_id, &from, &to)
            .await
            .expect("totals");

        assert_eq!(input, 500);
        assert_eq!(output, 250);
        assert_eq!(count, 5);
    }

    #[tokio::test]
    async fn test_get_project_totals() {
        let (pool, _dir) = test_pool().await;

        let now = chrono::Utc::now();
        let project_id = "proj-totals";

        for i in 0..3 {
            let usage = ApiUsage {
                id: uuid::Uuid::new_v4().to_string(),
                project_id: Some(project_id.to_string()),
                session_id: None,
                user_id: Some(format!("user-{}", i)),
                provider: "openai".to_string(),
                model: "gpt-4o".to_string(),
                input_tokens: 200,
                output_tokens: 100,
                request_type: "complete".to_string(),
                latency_ms: None,
                success: 1,
                error_type: None,
                created_at: now.to_rfc3339(),
            };
            insert_usage(&pool, &usage).await.expect("insert");
        }

        let from = (now - chrono::Duration::hours(1)).to_rfc3339();
        let to = (now + chrono::Duration::hours(1)).to_rfc3339();

        let (input, output, count) = get_project_totals(&pool, project_id, &from, &to)
            .await
            .expect("totals");

        assert_eq!(input, 600);
        assert_eq!(output, 300);
        assert_eq!(count, 3);
    }
}
