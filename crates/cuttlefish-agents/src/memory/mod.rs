//! Agent memory system for persistent project context.
//!
//! This module provides:
//! - Memory file management (`.cuttlefish/memory.md`)
//! - Decision logging (`.cuttlefish/decisions.jsonl`)
//! - Auto-update hooks for agent actions
//! - Decision indexing for fast lookups
//! - Why command for decision tracing
//! - State branching for project snapshots

mod branch;
pub mod file;
mod hooks;
mod index;
mod log;
mod why;

pub use branch::{
    BranchDiff, BranchError, BranchId, BranchStore, GitDiffSummary, MAX_BRANCHES_PER_PROJECT,
    StateBranch,
};
pub use file::{MemorySection, ProjectMemory};
pub use hooks::{MemoryHooks, MemoryTrigger, UpdateEvent};
pub use index::DecisionIndex;
pub use log::{ChangeType, DecisionEntry, DecisionLog};
pub use why::{
    ConversationExcerpt, ExcerptMessage, WhyExplanation, WhyTarget, get_conversation_excerpt,
    get_excerpts_for_decisions, redact_sensitive, why,
};
