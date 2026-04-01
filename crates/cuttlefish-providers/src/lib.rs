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

/// AWS Bedrock provider implementation.
pub mod bedrock;
/// Claude Code OAuth provider implementation.
pub mod claude_oauth;
/// Mock provider for testing.
pub mod mock;
/// OAuth flow utilities (PKCE, CCH signing, token exchange).
pub mod oauth_flow;

pub use cuttlefish_core::traits::provider::ModelProvider;
