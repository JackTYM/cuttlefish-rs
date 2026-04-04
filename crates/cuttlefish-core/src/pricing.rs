//! Pricing configuration for model cost calculation.
//!
//! Provides configurable pricing per model with support for:
//! - Default pricing loaded from configuration
//! - Per-provider and per-model pricing
//! - TOML configuration support

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Price configuration for a single model.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModelPrice {
    /// Price per million input tokens in USD.
    pub input_per_million: f64,
    /// Price per million output tokens in USD.
    pub output_per_million: f64,
}

impl ModelPrice {
    /// Create a new model price.
    pub fn new(input_per_million: f64, output_per_million: f64) -> Self {
        Self {
            input_per_million,
            output_per_million,
        }
    }

    /// Calculate cost for given token counts.
    pub fn calculate_cost(&self, input_tokens: u64, output_tokens: u64) -> f64 {
        let input_cost = (input_tokens as f64 / 1_000_000.0) * self.input_per_million;
        let output_cost = (output_tokens as f64 / 1_000_000.0) * self.output_per_million;
        input_cost + output_cost
    }
}

/// Pricing section for a single provider in TOML config.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PricingSection {
    /// Model prices keyed by model name.
    #[serde(flatten)]
    pub models: HashMap<String, ModelPriceToml>,
}

/// Model price in TOML format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPriceToml {
    /// Input price per million tokens.
    pub input: f64,
    /// Output price per million tokens.
    pub output: f64,
}

impl From<ModelPriceToml> for ModelPrice {
    fn from(toml: ModelPriceToml) -> Self {
        Self {
            input_per_million: toml.input,
            output_per_million: toml.output,
        }
    }
}

/// Complete pricing configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PricingConfig {
    /// Pricing by provider, then by model.
    #[serde(default)]
    providers: HashMap<String, HashMap<String, ModelPrice>>,
}

impl PricingConfig {
    /// Create an empty pricing config.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a pricing config with default prices for common models.
    pub fn with_defaults() -> Self {
        let mut config = Self::new();

        // Anthropic pricing (as of 2024)
        config.set_price("anthropic", "claude-opus-4-6", ModelPrice::new(15.0, 75.0));
        config.set_price("anthropic", "claude-sonnet-4-6", ModelPrice::new(3.0, 15.0));
        config.set_price("anthropic", "claude-haiku-4-5", ModelPrice::new(0.25, 1.25));

        // OpenAI pricing
        config.set_price("openai", "gpt-5.4", ModelPrice::new(10.0, 30.0));
        config.set_price("openai", "gpt-4o", ModelPrice::new(2.5, 10.0));
        config.set_price("openai", "gpt-5-nano", ModelPrice::new(0.15, 0.60));

        // Google pricing
        config.set_price("google", "gemini-2.0-flash", ModelPrice::new(0.075, 0.30));
        config.set_price("google", "gemini-1.5-pro", ModelPrice::new(1.25, 5.0));

        config
    }

    /// Load pricing from TOML configuration.
    pub fn from_toml(toml_str: &str) -> Result<Self, toml::de::Error> {
        #[derive(Deserialize)]
        struct TomlConfig {
            #[serde(default)]
            pricing: HashMap<String, PricingSection>,
        }

        let config: TomlConfig = toml::from_str(toml_str)?;
        let mut pricing = Self::new();

        for (provider, section) in config.pricing {
            for (model, price_toml) in section.models {
                pricing.set_price(&provider, &model, price_toml.into());
            }
        }

        Ok(pricing)
    }

    /// Set price for a specific provider and model.
    pub fn set_price(&mut self, provider: &str, model: &str, price: ModelPrice) {
        self.providers
            .entry(provider.to_string())
            .or_default()
            .insert(model.to_string(), price);
    }

    /// Get price for a specific provider and model.
    pub fn get_price(&self, provider: &str, model: &str) -> Option<&ModelPrice> {
        self.providers.get(provider)?.get(model)
    }

    /// Get all provider names.
    pub fn providers(&self) -> impl Iterator<Item = &str> {
        self.providers.keys().map(|s| s.as_str())
    }

    /// Get all model names for a provider.
    pub fn models(&self, provider: &str) -> Option<impl Iterator<Item = &str>> {
        self.providers
            .get(provider)
            .map(|models| models.keys().map(|s| s.as_str()))
    }

    /// Calculate cost for a request.
    pub fn calculate_cost(
        &self,
        provider: &str,
        model: &str,
        input_tokens: u64,
        output_tokens: u64,
    ) -> Option<f64> {
        self.get_price(provider, model)
            .map(|price| price.calculate_cost(input_tokens, output_tokens))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_price_calculate_cost() {
        let price = ModelPrice::new(15.0, 75.0);

        // 1000 input, 500 output
        // (1000/1M)*15 + (500/1M)*75 = 0.015 + 0.0375 = 0.0525
        let cost = price.calculate_cost(1000, 500);
        assert!((cost - 0.0525).abs() < 0.0001);
    }

    #[test]
    fn test_model_price_zero_tokens() {
        let price = ModelPrice::new(15.0, 75.0);
        let cost = price.calculate_cost(0, 0);
        assert_eq!(cost, 0.0);
    }

    #[test]
    fn test_pricing_config_with_defaults() {
        let config = PricingConfig::with_defaults();

        let opus = config.get_price("anthropic", "claude-opus-4-6");
        assert!(opus.is_some());
        assert_eq!(opus.map(|p| p.input_per_million), Some(15.0));

        let gpt = config.get_price("openai", "gpt-5.4");
        assert!(gpt.is_some());
    }

    #[test]
    fn test_pricing_config_set_and_get() {
        let mut config = PricingConfig::new();
        config.set_price("test", "model-1", ModelPrice::new(1.0, 2.0));

        let price = config.get_price("test", "model-1");
        assert!(price.is_some());
        assert_eq!(price.map(|p| p.input_per_million), Some(1.0));
        assert_eq!(price.map(|p| p.output_per_million), Some(2.0));
    }

    #[test]
    fn test_pricing_config_missing_price() {
        let config = PricingConfig::new();
        assert!(config.get_price("nonexistent", "model").is_none());
    }

    #[test]
    fn test_pricing_config_from_toml() {
        let toml = r#"
[pricing.anthropic]
"claude-opus-4-6" = { input = 15.0, output = 75.0 }
"claude-sonnet-4-6" = { input = 3.0, output = 15.0 }

[pricing.openai]
"gpt-5.4" = { input = 10.0, output = 30.0 }
"#;

        let config = PricingConfig::from_toml(toml).expect("parse toml");

        let opus = config.get_price("anthropic", "claude-opus-4-6");
        assert!(opus.is_some());
        assert_eq!(opus.map(|p| p.input_per_million), Some(15.0));

        let gpt = config.get_price("openai", "gpt-5.4");
        assert!(gpt.is_some());
        assert_eq!(gpt.map(|p| p.input_per_million), Some(10.0));
    }

    #[test]
    fn test_pricing_config_calculate_cost() {
        let config = PricingConfig::with_defaults();

        let cost = config.calculate_cost("anthropic", "claude-opus-4-6", 1000, 500);
        assert!(cost.is_some());
        assert!((cost.unwrap_or(0.0) - 0.0525).abs() < 0.0001);
    }

    #[test]
    fn test_pricing_config_providers_iterator() {
        let config = PricingConfig::with_defaults();
        let providers: Vec<_> = config.providers().collect();

        assert!(providers.contains(&"anthropic"));
        assert!(providers.contains(&"openai"));
        assert!(providers.contains(&"google"));
    }

    #[test]
    fn test_pricing_config_models_iterator() {
        let config = PricingConfig::with_defaults();
        let models: Vec<_> = config.models("anthropic").unwrap().collect();

        assert!(models.contains(&"claude-opus-4-6"));
        assert!(models.contains(&"claude-sonnet-4-6"));
        assert!(models.contains(&"claude-haiku-4-5"));
    }

    #[test]
    fn test_model_price_toml_conversion() {
        let toml = ModelPriceToml {
            input: 5.0,
            output: 10.0,
        };
        let price: ModelPrice = toml.into();

        assert_eq!(price.input_per_million, 5.0);
        assert_eq!(price.output_per_million, 10.0);
    }
}
