//! Context management for agent conversations.
//!
//! This module provides tools for managing conversation context:
//! - Token counting to track context window usage
//! - Automatic compaction to prevent context exhaustion
//! - Strategies for preserving important information while reducing size

mod compactor;
mod counter;

pub use compactor::{CompactionConfig, CompactionResult, ContextCompactor};
pub use counter::{MessageCategory, TokenCounter};
