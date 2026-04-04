//! Usage statistics service combining queries and cost calculations.

use crate::costs::CostCalculator;
use crate::pricing::PricingConfig;
use chrono::{Duration, Utc};
use cuttlefish_db::usage::{self, DailyUsage, ProviderUsage};
use sqlx::SqlitePool;
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;

/// Errors from usage statistics operations.
#[derive(Debug, Error)]
pub enum StatsError {
    /// Database error.
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
}

/// Time period for aggregation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimePeriod {
    /// Last 24 hours.
    Daily,
    /// Last 7 days.
    Weekly,
    /// Last 30 days.
    Monthly,
}

impl TimePeriod {
    /// Get the date range for this period.
    pub fn range(&self) -> (String, String) {
        let now = Utc::now();
        let from = match self {
            TimePeriod::Daily => now - Duration::days(1),
            TimePeriod::Weekly => now - Duration::days(7),
            TimePeriod::Monthly => now - Duration::days(30),
        };
        (from.to_rfc3339(), now.to_rfc3339())
    }
}

/// Project usage summary.
#[derive(Debug, Clone)]
pub struct ProjectUsageSummary {
    /// Project ID.
    pub project_id: String,
    /// Time period.
    pub period: TimePeriod,
    /// Total requests.
    pub total_requests: u32,
    /// Total input tokens.
    pub total_input_tokens: u64,
    /// Total output tokens.
    pub total_output_tokens: u64,
    /// Total cost in USD.
    pub total_cost_usd: f64,
    /// Cost by provider.
    pub by_provider: HashMap<String, f64>,
    /// Daily breakdown.
    pub daily_breakdown: Vec<DailyUsage>,
}

/// User usage summary.
#[derive(Debug, Clone)]
pub struct UserUsageSummary {
    /// User ID.
    pub user_id: String,
    /// Time period.
    pub period: TimePeriod,
    /// Total requests.
    pub total_requests: u32,
    /// Total input tokens.
    pub total_input_tokens: u64,
    /// Total output tokens.
    pub total_output_tokens: u64,
    /// Total cost in USD.
    pub total_cost_usd: f64,
    /// Cost by provider.
    pub by_provider: HashMap<String, f64>,
}

/// Usage statistics service.
pub struct UsageStats {
    pool: Arc<SqlitePool>,
    calculator: CostCalculator,
}

impl UsageStats {
    /// Create a new usage stats service.
    pub fn new(pool: Arc<SqlitePool>, pricing: PricingConfig) -> Self {
        Self {
            pool,
            calculator: CostCalculator::new(pricing),
        }
    }

    /// Get project usage summary.
    pub async fn project_summary(
        &self,
        project_id: &str,
        period: TimePeriod,
    ) -> Result<ProjectUsageSummary, StatsError> {
        let (from, to) = period.range();

        let (input, output, count) =
            usage::get_project_totals(&self.pool, project_id, &from, &to).await?;

        let usages = usage::get_usage_by_project(&self.pool, project_id, &from, &to).await?;
        let breakdown = self.calculator.calculate_total_cost(&usages);

        let daily = usage::aggregated_by_day(&self.pool, Some(project_id), &from, &to).await?;

        Ok(ProjectUsageSummary {
            project_id: project_id.to_string(),
            period,
            total_requests: count as u32,
            total_input_tokens: input as u64,
            total_output_tokens: output as u64,
            total_cost_usd: breakdown.total_usd,
            by_provider: breakdown.by_provider,
            daily_breakdown: daily,
        })
    }

    /// Get user usage summary.
    pub async fn user_summary(
        &self,
        user_id: &str,
        period: TimePeriod,
    ) -> Result<UserUsageSummary, StatsError> {
        let (from, to) = period.range();

        let (input, output, count) =
            usage::get_user_totals(&self.pool, user_id, &from, &to).await?;

        let usages = usage::get_usage_by_user(&self.pool, user_id, &from, &to).await?;
        let breakdown = self.calculator.calculate_total_cost(&usages);

        Ok(UserUsageSummary {
            user_id: user_id.to_string(),
            period,
            total_requests: count as u32,
            total_input_tokens: input as u64,
            total_output_tokens: output as u64,
            total_cost_usd: breakdown.total_usd,
            by_provider: breakdown.by_provider,
        })
    }

    /// Get current period cost for a user.
    pub async fn current_period_cost(
        &self,
        user_id: &str,
        period: TimePeriod,
    ) -> Result<f64, StatsError> {
        let (from, to) = period.range();
        let usages = usage::get_usage_by_user(&self.pool, user_id, &from, &to).await?;
        let breakdown = self.calculator.calculate_total_cost(&usages);
        Ok(breakdown.total_usd)
    }

    /// Get daily usage breakdown.
    pub async fn daily_usage(
        &self,
        project_id: Option<&str>,
        period: TimePeriod,
    ) -> Result<Vec<DailyUsage>, StatsError> {
        let (from, to) = period.range();
        let daily = usage::aggregated_by_day(&self.pool, project_id, &from, &to).await?;
        Ok(daily)
    }

    /// Get provider usage breakdown.
    pub async fn provider_usage(
        &self,
        project_id: Option<&str>,
        period: TimePeriod,
    ) -> Result<Vec<ProviderUsage>, StatsError> {
        let (from, to) = period.range();
        let providers = usage::aggregated_by_provider(&self.pool, project_id, &from, &to).await?;
        Ok(providers)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cuttlefish_db::usage::{ApiUsage, insert_usage, run_usage_migrations};
    use tempfile::TempDir;

    async fn test_pool() -> (Arc<SqlitePool>, TempDir) {
        let dir = TempDir::new().expect("temp dir");
        let db_path = dir.path().join("test.db");
        let url = format!("sqlite://{}?mode=rwc", db_path.to_string_lossy());
        let pool = SqlitePool::connect(&url).await.expect("connect");
        run_usage_migrations(&pool).await.expect("migrations");
        (Arc::new(pool), dir)
    }

    fn make_usage(
        user_id: &str,
        project_id: &str,
        provider: &str,
        model: &str,
        input: i64,
        output: i64,
    ) -> ApiUsage {
        ApiUsage {
            id: uuid::Uuid::new_v4().to_string(),
            project_id: Some(project_id.to_string()),
            session_id: None,
            user_id: Some(user_id.to_string()),
            provider: provider.to_string(),
            model: model.to_string(),
            input_tokens: input,
            output_tokens: output,
            request_type: "complete".to_string(),
            latency_ms: Some(100),
            success: 1,
            error_type: None,
            created_at: Utc::now().to_rfc3339(),
        }
    }

    #[tokio::test]
    async fn test_project_summary() {
        let (pool, _dir) = test_pool().await;

        let usage = make_usage(
            "user-1",
            "proj-1",
            "anthropic",
            "claude-opus-4-6",
            1000,
            500,
        );
        insert_usage(&pool, &usage).await.expect("insert");

        let stats = UsageStats::new(pool, PricingConfig::with_defaults());
        let summary = stats
            .project_summary("proj-1", TimePeriod::Daily)
            .await
            .expect("summary");

        assert_eq!(summary.project_id, "proj-1");
        assert_eq!(summary.total_requests, 1);
        assert_eq!(summary.total_input_tokens, 1000);
        assert_eq!(summary.total_output_tokens, 500);
        assert!(summary.total_cost_usd > 0.0);
    }

    #[tokio::test]
    async fn test_user_summary() {
        let (pool, _dir) = test_pool().await;

        let usage1 = make_usage(
            "user-1",
            "proj-1",
            "anthropic",
            "claude-opus-4-6",
            1000,
            500,
        );
        let usage2 = make_usage("user-1", "proj-2", "openai", "gpt-5.4", 2000, 1000);
        insert_usage(&pool, &usage1).await.expect("insert");
        insert_usage(&pool, &usage2).await.expect("insert");

        let stats = UsageStats::new(pool, PricingConfig::with_defaults());
        let summary = stats
            .user_summary("user-1", TimePeriod::Daily)
            .await
            .expect("summary");

        assert_eq!(summary.user_id, "user-1");
        assert_eq!(summary.total_requests, 2);
        assert_eq!(summary.total_input_tokens, 3000);
        assert_eq!(summary.total_output_tokens, 1500);
        assert!(summary.by_provider.contains_key("anthropic"));
        assert!(summary.by_provider.contains_key("openai"));
    }

    #[tokio::test]
    async fn test_current_period_cost() {
        let (pool, _dir) = test_pool().await;

        let usage = make_usage(
            "user-1",
            "proj-1",
            "anthropic",
            "claude-opus-4-6",
            1000,
            500,
        );
        insert_usage(&pool, &usage).await.expect("insert");

        let stats = UsageStats::new(pool, PricingConfig::with_defaults());
        let cost = stats
            .current_period_cost("user-1", TimePeriod::Daily)
            .await
            .expect("cost");

        assert!(cost > 0.0);
    }

    #[test]
    fn test_time_period_range() {
        let (from_d, to_d) = TimePeriod::Daily.range();
        assert!(!from_d.is_empty());
        assert!(!to_d.is_empty());

        let (from_w, _to_w) = TimePeriod::Weekly.range();
        assert!(from_w < from_d);
    }
}
