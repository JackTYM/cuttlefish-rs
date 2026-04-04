//! Spending alert system for usage monitoring.

use crate::stats::{TimePeriod, UsageStats};
use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use cuttlefish_db::usage::{self, UsageAlert};
use sqlx::SqlitePool;
use std::sync::Arc;
use thiserror::Error;
use tracing::{info, warn};

/// Errors from alert operations.
#[derive(Debug, Error)]
pub enum AlertError {
    /// Database error.
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
    /// Notification error.
    #[error("notification error: {0}")]
    Notification(String),
}

/// A triggered alert with current cost information.
#[derive(Debug, Clone)]
pub struct TriggeredAlert {
    /// The alert that was triggered.
    pub alert: UsageAlert,
    /// Current cost that triggered the alert.
    pub current_cost: f64,
    /// Percentage of threshold reached.
    pub percentage: f64,
}

/// Trait for alert notification channels.
#[async_trait]
pub trait AlertNotifier: Send + Sync {
    /// Send notification for a triggered alert.
    async fn notify(&self, alert: &UsageAlert, current_cost: f64) -> Result<(), AlertError>;
}

/// No-op notifier for testing.
pub struct NoopNotifier;

#[async_trait]
impl AlertNotifier for NoopNotifier {
    async fn notify(&self, _alert: &UsageAlert, _current_cost: f64) -> Result<(), AlertError> {
        Ok(())
    }
}

/// Logging notifier that writes to tracing.
pub struct LogNotifier;

#[async_trait]
impl AlertNotifier for LogNotifier {
    async fn notify(&self, alert: &UsageAlert, current_cost: f64) -> Result<(), AlertError> {
        warn!(
            user_id = %alert.user_id,
            threshold = alert.threshold_usd,
            current = current_cost,
            period = %alert.period,
            "Spending alert triggered"
        );
        Ok(())
    }
}

/// Alert checker that monitors usage against thresholds.
pub struct AlertChecker {
    pool: Arc<SqlitePool>,
    stats: Arc<UsageStats>,
    notifier: Box<dyn AlertNotifier>,
    cooldown_hours: i64,
}

impl AlertChecker {
    /// Create a new alert checker.
    pub fn new(
        pool: Arc<SqlitePool>,
        stats: Arc<UsageStats>,
        notifier: Box<dyn AlertNotifier>,
    ) -> Self {
        Self {
            pool,
            stats,
            notifier,
            cooldown_hours: 24,
        }
    }

    /// Set the cooldown period between repeated alerts.
    pub fn with_cooldown_hours(mut self, hours: i64) -> Self {
        self.cooldown_hours = hours;
        self
    }

    /// Check all active alerts and trigger notifications for those exceeding thresholds.
    pub async fn check_alerts(&self) -> Result<Vec<TriggeredAlert>, AlertError> {
        let alerts = usage::get_enabled_alerts(&self.pool).await?;
        let mut triggered = Vec::new();

        for alert in alerts {
            if self.should_skip_alert(&alert) {
                continue;
            }

            let period = self.parse_period(&alert.period);
            let current_cost = self
                .stats
                .current_period_cost(&alert.user_id, period)
                .await
                .unwrap_or(0.0);

            if current_cost >= alert.threshold_usd {
                let percentage = (current_cost / alert.threshold_usd) * 100.0;

                self.notifier.notify(&alert, current_cost).await?;
                usage::update_alert_triggered(&self.pool, &alert.id).await?;

                info!(
                    alert_id = %alert.id,
                    user_id = %alert.user_id,
                    threshold = alert.threshold_usd,
                    current = current_cost,
                    "Alert triggered"
                );

                triggered.push(TriggeredAlert {
                    alert,
                    current_cost,
                    percentage,
                });
            }
        }

        Ok(triggered)
    }

    fn should_skip_alert(&self, alert: &UsageAlert) -> bool {
        if let Some(ref last_triggered) = alert.last_triggered_at
            && let Ok(last_time) = DateTime::parse_from_rfc3339(last_triggered)
        {
            let last_time_utc: DateTime<Utc> = last_time.into();
            let cooldown = Duration::hours(self.cooldown_hours);
            if Utc::now() - last_time_utc < cooldown {
                return true;
            }
        }
        false
    }

    fn parse_period(&self, period: &str) -> TimePeriod {
        match period {
            "daily" => TimePeriod::Daily,
            "weekly" => TimePeriod::Weekly,
            _ => TimePeriod::Monthly,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pricing::PricingConfig;
    use cuttlefish_db::usage::{create_alert, insert_usage, run_usage_migrations, ApiUsage};
    use std::sync::atomic::{AtomicU32, Ordering};
    use tempfile::TempDir;

    struct CountingNotifier {
        count: AtomicU32,
    }

    impl CountingNotifier {
        fn new() -> Self {
            Self {
                count: AtomicU32::new(0),
            }
        }

        fn count(&self) -> u32 {
            self.count.load(Ordering::SeqCst)
        }
    }

    #[async_trait]
    impl AlertNotifier for CountingNotifier {
        async fn notify(&self, _alert: &UsageAlert, _current_cost: f64) -> Result<(), AlertError> {
            self.count.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
    }

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
        provider: &str,
        model: &str,
        input: i64,
        output: i64,
    ) -> ApiUsage {
        ApiUsage {
            id: uuid::Uuid::new_v4().to_string(),
            project_id: Some("proj-1".to_string()),
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
    async fn test_alert_triggers_when_threshold_exceeded() {
        let (pool, _dir) = test_pool().await;

        let usage = make_usage("user-1", "anthropic", "claude-opus-4-6", 100_000, 50_000);
        insert_usage(&pool, &usage).await.expect("insert");

        let alert = UsageAlert {
            id: uuid::Uuid::new_v4().to_string(),
            user_id: "user-1".to_string(),
            project_id: None,
            threshold_usd: 0.01,
            period: "daily".to_string(),
            last_triggered_at: None,
            enabled: 1,
            created_at: Utc::now().to_rfc3339(),
        };
        create_alert(&pool, &alert).await.expect("create alert");

        let stats = Arc::new(UsageStats::new(pool.clone(), PricingConfig::with_defaults()));

        let checker = AlertChecker::new(pool, stats, Box::new(CountingNotifier::new()));
        let triggered = checker.check_alerts().await.expect("check");

        assert_eq!(triggered.len(), 1);
        assert!(triggered[0].current_cost >= 0.01);
    }

    #[tokio::test]
    async fn test_alert_does_not_trigger_below_threshold() {
        let (pool, _dir) = test_pool().await;

        let usage = make_usage("user-2", "anthropic", "claude-haiku-4-5", 100, 50);
        insert_usage(&pool, &usage).await.expect("insert");

        let alert = UsageAlert {
            id: uuid::Uuid::new_v4().to_string(),
            user_id: "user-2".to_string(),
            project_id: None,
            threshold_usd: 1000.0,
            period: "daily".to_string(),
            last_triggered_at: None,
            enabled: 1,
            created_at: Utc::now().to_rfc3339(),
        };
        create_alert(&pool, &alert).await.expect("create alert");

        let stats = Arc::new(UsageStats::new(pool.clone(), PricingConfig::with_defaults()));
        let checker = AlertChecker::new(pool, stats, Box::new(NoopNotifier));
        let triggered = checker.check_alerts().await.expect("check");

        assert!(triggered.is_empty());
    }

    #[tokio::test]
    async fn test_alert_respects_cooldown() {
        let (pool, _dir) = test_pool().await;

        let usage = make_usage("user-3", "anthropic", "claude-opus-4-6", 100_000, 50_000);
        insert_usage(&pool, &usage).await.expect("insert");

        let alert = UsageAlert {
            id: uuid::Uuid::new_v4().to_string(),
            user_id: "user-3".to_string(),
            project_id: None,
            threshold_usd: 0.01,
            period: "daily".to_string(),
            last_triggered_at: Some(Utc::now().to_rfc3339()),
            enabled: 1,
            created_at: Utc::now().to_rfc3339(),
        };
        create_alert(&pool, &alert).await.expect("create alert");

        let stats = Arc::new(UsageStats::new(pool.clone(), PricingConfig::with_defaults()));
        let checker = AlertChecker::new(pool, stats, Box::new(NoopNotifier)).with_cooldown_hours(24);
        let triggered = checker.check_alerts().await.expect("check");

        assert!(triggered.is_empty());
    }

    #[tokio::test]
    async fn test_disabled_alerts_not_checked() {
        let (pool, _dir) = test_pool().await;

        let usage = make_usage("user-4", "anthropic", "claude-opus-4-6", 100_000, 50_000);
        insert_usage(&pool, &usage).await.expect("insert");

        let alert = UsageAlert {
            id: uuid::Uuid::new_v4().to_string(),
            user_id: "user-4".to_string(),
            project_id: None,
            threshold_usd: 0.01,
            period: "daily".to_string(),
            last_triggered_at: None,
            enabled: 0,
            created_at: Utc::now().to_rfc3339(),
        };
        create_alert(&pool, &alert).await.expect("create alert");

        let stats = Arc::new(UsageStats::new(pool.clone(), PricingConfig::with_defaults()));
        let checker = AlertChecker::new(pool, stats, Box::new(NoopNotifier));
        let triggered = checker.check_alerts().await.expect("check");

        assert!(triggered.is_empty());
    }

    #[test]
    fn test_parse_period_values() {
        assert_eq!(
            match "daily" {
                "daily" => TimePeriod::Daily,
                "weekly" => TimePeriod::Weekly,
                _ => TimePeriod::Monthly,
            },
            TimePeriod::Daily
        );
        assert_eq!(
            match "weekly" {
                "daily" => TimePeriod::Daily,
                "weekly" => TimePeriod::Weekly,
                _ => TimePeriod::Monthly,
            },
            TimePeriod::Weekly
        );
        assert_eq!(
            match "monthly" {
                "daily" => TimePeriod::Daily,
                "weekly" => TimePeriod::Weekly,
                _ => TimePeriod::Monthly,
            },
            TimePeriod::Monthly
        );
        assert_eq!(
            match "unknown" {
                "daily" => TimePeriod::Daily,
                "weekly" => TimePeriod::Weekly,
                _ => TimePeriod::Monthly,
            },
            TimePeriod::Monthly
        );
    }
}
