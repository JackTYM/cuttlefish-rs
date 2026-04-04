//! Cost calculation engine for usage tracking.

use crate::pricing::PricingConfig;
use cuttlefish_db::usage::ApiUsage;
use std::collections::HashMap;
use thiserror::Error;
use tracing::warn;

/// Errors from cost calculation.
#[derive(Debug, Error)]
pub enum CostError {
    /// Pricing not found for model.
    #[error("pricing not found for {provider}/{model}")]
    PricingNotFound {
        /// Provider name.
        provider: String,
        /// Model name.
        model: String,
    },
}

/// Cost breakdown for a set of usage records.
#[derive(Debug, Clone, Default)]
pub struct CostBreakdown {
    /// Total cost in USD.
    pub total_usd: f64,
    /// Cost by provider.
    pub by_provider: HashMap<String, f64>,
    /// Cost by model (provider/model format).
    pub by_model: HashMap<String, f64>,
    /// Total input tokens.
    pub input_tokens_total: u64,
    /// Total output tokens.
    pub output_tokens_total: u64,
    /// Total request count.
    pub request_count: u32,
}

impl CostBreakdown {
    /// Add a single request's cost to the breakdown.
    pub fn add(&mut self, provider: &str, model: &str, input: u64, output: u64, cost: f64) {
        self.total_usd += cost;
        self.input_tokens_total += input;
        self.output_tokens_total += output;
        self.request_count += 1;

        *self.by_provider.entry(provider.to_string()).or_default() += cost;

        let model_key = format!("{}/{}", provider, model);
        *self.by_model.entry(model_key).or_default() += cost;
    }
}

/// Calculator for computing costs from usage records.
#[derive(Debug, Clone)]
pub struct CostCalculator {
    pricing: PricingConfig,
}

impl CostCalculator {
    /// Create a new cost calculator with the given pricing config.
    pub fn new(pricing: PricingConfig) -> Self {
        Self { pricing }
    }

    /// Calculate cost for a single usage record.
    pub fn calculate_request_cost(&self, usage: &ApiUsage) -> Result<f64, CostError> {
        let input = usage.input_tokens.max(0) as u64;
        let output = usage.output_tokens.max(0) as u64;

        self.pricing
            .calculate_cost(&usage.provider, &usage.model, input, output)
            .ok_or_else(|| CostError::PricingNotFound {
                provider: usage.provider.clone(),
                model: usage.model.clone(),
            })
    }

    /// Calculate cost for a single usage record, returning 0 if pricing not found.
    pub fn calculate_request_cost_or_zero(&self, usage: &ApiUsage) -> f64 {
        self.calculate_request_cost(usage).unwrap_or_else(|e| {
            warn!("{}", e);
            0.0
        })
    }

    /// Calculate total cost breakdown for multiple usage records.
    pub fn calculate_total_cost(&self, usages: &[ApiUsage]) -> CostBreakdown {
        let mut breakdown = CostBreakdown::default();

        for usage in usages {
            let input = usage.input_tokens.max(0) as u64;
            let output = usage.output_tokens.max(0) as u64;
            let cost = self.calculate_request_cost_or_zero(usage);

            breakdown.add(&usage.provider, &usage.model, input, output, cost);
        }

        breakdown
    }

    /// Get the underlying pricing config.
    pub fn pricing(&self) -> &PricingConfig {
        &self.pricing
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_usage(provider: &str, model: &str, input: i64, output: i64) -> ApiUsage {
        ApiUsage {
            id: "test-id".to_string(),
            project_id: Some("proj-1".to_string()),
            session_id: None,
            user_id: Some("user-1".to_string()),
            provider: provider.to_string(),
            model: model.to_string(),
            input_tokens: input,
            output_tokens: output,
            request_type: "complete".to_string(),
            latency_ms: Some(100),
            success: 1,
            error_type: None,
            created_at: "2024-01-01T00:00:00Z".to_string(),
        }
    }

    #[test]
    fn test_calculate_request_cost() {
        let pricing = PricingConfig::with_defaults();
        let calculator = CostCalculator::new(pricing);

        let usage = make_usage("anthropic", "claude-opus-4-6", 1000, 500);
        let cost = calculator.calculate_request_cost(&usage);

        assert!(cost.is_ok());
        let cost = cost.expect("cost");
        assert!((cost - 0.0525).abs() < 0.0001);
    }

    #[test]
    fn test_calculate_request_cost_missing_pricing() {
        let pricing = PricingConfig::new();
        let calculator = CostCalculator::new(pricing);

        let usage = make_usage("unknown", "model", 1000, 500);
        let cost = calculator.calculate_request_cost(&usage);

        assert!(cost.is_err());
    }

    #[test]
    fn test_calculate_request_cost_or_zero() {
        let pricing = PricingConfig::new();
        let calculator = CostCalculator::new(pricing);

        let usage = make_usage("unknown", "model", 1000, 500);
        let cost = calculator.calculate_request_cost_or_zero(&usage);

        assert_eq!(cost, 0.0);
    }

    #[test]
    fn test_calculate_total_cost() {
        let pricing = PricingConfig::with_defaults();
        let calculator = CostCalculator::new(pricing);

        let usages = vec![
            make_usage("anthropic", "claude-opus-4-6", 1000, 500),
            make_usage("anthropic", "claude-opus-4-6", 2000, 1000),
            make_usage("openai", "gpt-5.4", 1000, 500),
        ];

        let breakdown = calculator.calculate_total_cost(&usages);

        assert_eq!(breakdown.request_count, 3);
        assert_eq!(breakdown.input_tokens_total, 4000);
        assert_eq!(breakdown.output_tokens_total, 2000);
        assert!(breakdown.total_usd > 0.0);
        assert!(breakdown.by_provider.contains_key("anthropic"));
        assert!(breakdown.by_provider.contains_key("openai"));
    }

    #[test]
    fn test_cost_breakdown_add() {
        let mut breakdown = CostBreakdown::default();

        breakdown.add("anthropic", "claude-opus-4-6", 1000, 500, 0.05);
        breakdown.add("anthropic", "claude-opus-4-6", 2000, 1000, 0.10);
        breakdown.add("openai", "gpt-5.4", 1000, 500, 0.03);

        assert_eq!(breakdown.request_count, 3);
        assert_eq!(breakdown.input_tokens_total, 4000);
        assert_eq!(breakdown.output_tokens_total, 2000);
        assert!((breakdown.total_usd - 0.18).abs() < 0.0001);
        assert!((breakdown.by_provider["anthropic"] - 0.15).abs() < 0.0001);
        assert!((breakdown.by_provider["openai"] - 0.03).abs() < 0.0001);
    }

    #[test]
    fn test_cost_breakdown_by_model() {
        let mut breakdown = CostBreakdown::default();

        breakdown.add("anthropic", "claude-opus-4-6", 1000, 500, 0.05);
        breakdown.add("anthropic", "claude-sonnet-4-6", 1000, 500, 0.02);

        assert!(breakdown.by_model.contains_key("anthropic/claude-opus-4-6"));
        assert!(
            breakdown
                .by_model
                .contains_key("anthropic/claude-sonnet-4-6")
        );
    }

    #[test]
    fn test_negative_tokens_handled() {
        let pricing = PricingConfig::with_defaults();
        let calculator = CostCalculator::new(pricing);

        let usage = make_usage("anthropic", "claude-opus-4-6", -100, -50);
        let cost = calculator.calculate_request_cost(&usage);

        assert!(cost.is_ok());
        assert_eq!(cost.expect("cost"), 0.0);
    }
}
