//! Session persistence: durable message storage and crash recovery.
//!
//! This module provides:
//! - Database persistence for conversation messages
//! - JSONL journaling for crash recovery
//! - Automatic session restore on reconnection

mod journal;
mod persistence;

pub use journal::{JournalEntry, SessionJournal};
pub use persistence::{ConversationPersistence, PersistenceConfig, RestoreResult};
