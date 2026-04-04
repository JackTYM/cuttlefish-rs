#![deny(unsafe_code)]
#![warn(missing_docs)]
//! Model provider implementations for Cuttlefish.
//!
//! Provides [`ModelProvider`] implementations for:
//! - AWS Bedrock (Claude models via Bedrock API)
//! - Claude Code OAuth (direct Anthropic API via CLI emulation)
//! - Mock provider (for testing)
//!
//! # Usage
//! ```no_run
//! use cuttlefish_providers::bedrock::BedrockProvider;
//! use cuttlefish_providers::claude_oauth::ClaudeOAuthProvider;
//! ```

/// Anthropic direct API provider implementation.
pub mod anthropic;
/// AWS Bedrock provider implementation.
pub mod bedrock;
/// ChatGPT OAuth provider implementation.
pub mod chatgpt_oauth;
/// Claude Code OAuth provider implementation.
pub mod claude_oauth;
/// Google Gemini API provider implementation.
pub mod google;
/// MiniMax API provider implementation.
pub mod minimax;
/// Mock provider for testing.
pub mod mock;
/// Moonshot (Kimi) API provider implementation.
pub mod moonshot;
/// OAuth flow utilities (PKCE, CCH signing, token exchange).
pub mod oauth_flow;
/// Ollama local model provider implementation.
pub mod ollama;
/// OpenAI API provider implementation.
pub mod openai;
/// Tracked provider wrapper for usage logging.
pub mod tracked;
/// xAI (Grok) API provider implementation.
pub mod xai;
/// Zhipu (GLM) API provider implementation.
pub mod zhipu;

/// Provider registry for managing named provider instances.
pub mod registry;

pub use cuttlefish_core::traits::provider::ModelProvider;
pub use registry::ProviderRegistry;
pub use tracked::{TrackedProvider, UsageContext};
