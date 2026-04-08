//! Session persistence coordinating database storage and JSONL journaling.

use std::path::Path;
use std::sync::Arc;

use chrono::Utc;
use cuttlefish_core::traits::provider::{Message, MessageRole};
use cuttlefish_db::Database;
use tracing::{debug, info, warn};

use super::journal::{JournalEntry, JournalEvent, SessionJournal};

/// Configuration for session persistence.
#[derive(Debug, Clone)]
pub struct PersistenceConfig {
    /// Path to the journal directory.
    pub journal_dir: std::path::PathBuf,
    /// Maximum journal file size (bytes).
    pub journal_max_size: u64,
    /// Number of rotated journals to keep.
    pub journal_max_rotations: usize,
    /// Whether to sync to database after each message.
    pub sync_on_each_message: bool,
    /// Number of messages to keep in sliding window.
    pub sliding_window_size: i64,
}

impl Default for PersistenceConfig {
    fn default() -> Self {
        Self {
            journal_dir: std::path::PathBuf::from("data/journals"),
            journal_max_size: 1024 * 1024,  // 1MB
            journal_max_rotations: 10,
            sync_on_each_message: true,
            sliding_window_size: 100,
        }
    }
}

/// Result of restoring a session.
#[derive(Debug)]
pub struct RestoreResult {
    /// Messages restored from database.
    pub messages: Vec<Message>,
    /// Number of messages loaded from database.
    pub from_database: usize,
    /// Number of messages recovered from journal.
    pub from_journal: usize,
    /// Whether crash recovery was performed.
    pub crash_recovery: bool,
}

/// Coordinates database persistence and JSONL journaling.
pub struct ConversationPersistence {
    db: Arc<Database>,
    journal: SessionJournal,
    config: PersistenceConfig,
}

impl ConversationPersistence {
    /// Create a new persistence coordinator.
    pub fn new(db: Arc<Database>, config: PersistenceConfig) -> std::io::Result<Self> {
        let journal_path = config.journal_dir.join("session.jsonl");
        let journal = SessionJournal::with_config(
            &journal_path,
            config.journal_max_size,
            config.journal_max_rotations,
        )?;

        Ok(Self { db, journal, config })
    }

    /// Create with a specific journal path (for testing or custom setups).
    pub fn with_journal_path(
        db: Arc<Database>,
        journal_path: impl AsRef<Path>,
        config: PersistenceConfig,
    ) -> std::io::Result<Self> {
        let journal = SessionJournal::with_config(
            journal_path,
            config.journal_max_size,
            config.journal_max_rotations,
        )?;

        Ok(Self { db, journal, config })
    }

    /// Save a message to both database and journal.
    pub async fn save_message(
        &mut self,
        project_id: &str,
        message: &Message,
        token_count: i64,
        model_used: Option<&str>,
    ) -> Result<String, PersistenceError> {
        let message_id = uuid::Uuid::new_v4().to_string();

        // Write to journal first (write-ahead log)
        let entry = JournalEntry {
            timestamp: Utc::now(),
            project_id: project_id.to_string(),
            event: JournalEvent::MessageAdded {
                message_id: message_id.clone(),
                role: role_to_string(&message.role),
                content: message.content.clone(),
                token_count,
                model_used: model_used.map(String::from),
            },
        };
        self.journal.append(&entry)?;

        // Then write to database
        if self.config.sync_on_each_message {
            self.db
                .insert_message(
                    &message_id,
                    project_id,
                    &role_to_string(&message.role),
                    &message.content,
                    model_used,
                    token_count,
                )
                .await
                .map_err(PersistenceError::database)?;

            debug!(
                message_id = %message_id,
                project_id = %project_id,
                role = ?message.role,
                "Message saved to database"
            );
        }

        Ok(message_id)
    }

    /// Restore a session from database and journal.
    ///
    /// First loads from database, then checks journal for any messages
    /// that weren't synced before a crash.
    pub async fn restore_session(
        &mut self,
        project_id: &str,
    ) -> Result<RestoreResult, PersistenceError> {
        // Load from database
        let db_messages = self
            .db
            .get_recent_messages_chrono(project_id, self.config.sliding_window_size)
            .await
            .map_err(PersistenceError::database)?;

        let from_database = db_messages.len();
        let mut messages: Vec<Message> = db_messages
            .into_iter()
            .map(|m| Message {
                role: string_to_role(&m.role),
                content: m.content,
            })
            .collect();

        // Check journal for crash recovery
        let journal_entries = self.journal.read_for_project(project_id)?;
        let mut from_journal = 0;
        let mut crash_recovery = false;

        // Find messages in journal that aren't in database
        // This happens if the process crashed after journaling but before DB commit
        if !journal_entries.is_empty() {
            // Get message IDs from database to check for duplicates
            let db_message_ids: std::collections::HashSet<String> = self
                .db
                .get_recent_messages(project_id, self.config.sliding_window_size)
                .await
                .map_err(PersistenceError::database)?
                .into_iter()
                .map(|m| m.id)
                .collect();

            for entry in journal_entries {
                if let JournalEvent::MessageAdded {
                    message_id,
                    role,
                    content,
                    token_count,
                    model_used,
                } = entry.event
                {
                    if !db_message_ids.contains(&message_id) {
                        // This message was journaled but not committed to DB
                        crash_recovery = true;
                        from_journal += 1;

                        // Replay to database
                        if let Err(e) = self
                            .db
                            .insert_message(
                                &message_id,
                                project_id,
                                &role,
                                &content,
                                model_used.as_deref(),
                                token_count,
                            )
                            .await
                        {
                            warn!(
                                message_id = %message_id,
                                error = %e,
                                "Failed to replay journaled message to database"
                            );
                        } else {
                            messages.push(Message {
                                role: string_to_role(&role),
                                content,
                            });
                            info!(message_id = %message_id, "Recovered message from journal");
                        }
                    }
                }
            }
        }

        if crash_recovery {
            info!(
                project_id = %project_id,
                recovered = from_journal,
                "Crash recovery: replayed journal entries to database"
            );
        }

        // Log the restore event
        let restore_entry = JournalEntry {
            timestamp: Utc::now(),
            project_id: project_id.to_string(),
            event: JournalEvent::SessionRestored {
                message_count: messages.len(),
            },
        };
        let _ = self.journal.append(&restore_entry);

        Ok(RestoreResult {
            messages,
            from_database,
            from_journal,
            crash_recovery,
        })
    }

    /// Record a compaction event.
    pub fn record_compaction(
        &mut self,
        project_id: &str,
        messages_removed: usize,
        tokens_before: usize,
        tokens_after: usize,
    ) -> Result<(), PersistenceError> {
        let entry = JournalEntry {
            timestamp: Utc::now(),
            project_id: project_id.to_string(),
            event: JournalEvent::ContextCompacted {
                messages_removed,
                tokens_before,
                tokens_after,
            },
        };
        self.journal.append(&entry)?;
        Ok(())
    }

    /// Archive old messages using the database's archive_and_summarize.
    pub async fn archive_old_messages(
        &mut self,
        project_id: &str,
        keep_recent: i64,
        summary: &str,
    ) -> Result<u64, PersistenceError> {
        // Get the timestamp of the Nth most recent message
        let cutoff = self
            .db
            .get_nth_recent_message_timestamp(project_id, keep_recent)
            .await
            .map_err(PersistenceError::database)?;

        let Some(cutoff_ts) = cutoff else {
            // Not enough messages to archive
            return Ok(0);
        };

        let summary_id = uuid::Uuid::new_v4().to_string();
        let archived = self
            .db
            .archive_and_summarize(project_id, &cutoff_ts, &summary_id, summary)
            .await
            .map_err(PersistenceError::database)?;

        // Log the archive event
        let entry = JournalEntry {
            timestamp: Utc::now(),
            project_id: project_id.to_string(),
            event: JournalEvent::SessionArchived {
                message_count: archived as usize,
            },
        };
        let _ = self.journal.append(&entry);

        info!(
            project_id = %project_id,
            archived = archived,
            "Archived old messages"
        );

        Ok(archived)
    }

    /// Clear the journal after successful database sync.
    ///
    /// Call this periodically after confirming all journal entries
    /// have been committed to the database.
    pub fn clear_journal(&mut self) -> Result<(), PersistenceError> {
        self.journal.clear()?;
        Ok(())
    }

    /// Get the database reference.
    pub fn db(&self) -> &Database {
        &self.db
    }

    /// Get the configuration.
    pub fn config(&self) -> &PersistenceConfig {
        &self.config
    }
}

/// Persistence errors.
#[derive(Debug, thiserror::Error)]
pub enum PersistenceError {
    /// Database error.
    #[error("Database error: {0}")]
    Database(String),
    /// Journal I/O error.
    #[error("Journal I/O error: {0}")]
    Journal(#[from] std::io::Error),
}

impl PersistenceError {
    /// Create a database error from any error type.
    pub fn database(err: impl std::fmt::Display) -> Self {
        Self::Database(err.to_string())
    }
}

/// Convert MessageRole to database string.
fn role_to_string(role: &MessageRole) -> String {
    match role {
        MessageRole::User => "user".to_string(),
        MessageRole::Assistant => "assistant".to_string(),
        MessageRole::System => "system".to_string(),
    }
}

/// Convert database string to MessageRole.
fn string_to_role(s: &str) -> MessageRole {
    match s {
        "user" => MessageRole::User,
        "assistant" => MessageRole::Assistant,
        "system" => MessageRole::System,
        _ => MessageRole::User,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    async fn test_setup() -> (Arc<Database>, TempDir) {
        let dir = TempDir::new().expect("temp dir");
        let db_path = dir.path().join("test.db");
        let db = Database::open(&db_path).await.expect("open db");
        (Arc::new(db), dir)
    }

    #[tokio::test]
    async fn test_save_and_restore() {
        let (db, dir) = test_setup().await;
        let journal_path = dir.path().join("journal.jsonl");

        // Create a project
        let project_id = uuid::Uuid::new_v4().to_string();
        db.create_project(&project_id, "test-project", "Test", None)
            .await
            .expect("create project");

        let config = PersistenceConfig {
            journal_dir: dir.path().to_path_buf(),
            ..Default::default()
        };

        let mut persistence =
            ConversationPersistence::with_journal_path(db.clone(), &journal_path, config.clone())
                .expect("create persistence");

        // Save some messages
        let msg1 = Message {
            role: MessageRole::User,
            content: "Hello".to_string(),
        };
        persistence
            .save_message(&project_id, &msg1, 10, None)
            .await
            .expect("save");

        let msg2 = Message {
            role: MessageRole::Assistant,
            content: "Hi there!".to_string(),
        };
        persistence
            .save_message(&project_id, &msg2, 15, Some("gpt-4"))
            .await
            .expect("save");

        // Restore session
        let mut persistence2 =
            ConversationPersistence::with_journal_path(db.clone(), &journal_path, config)
                .expect("create persistence");

        let result = persistence2
            .restore_session(&project_id)
            .await
            .expect("restore");

        assert_eq!(result.messages.len(), 2);
        assert_eq!(result.from_database, 2);
        assert_eq!(result.from_journal, 0);
        assert!(!result.crash_recovery);
    }

    #[tokio::test]
    async fn test_crash_recovery() {
        let (db, dir) = test_setup().await;
        let journal_path = dir.path().join("journal.jsonl");

        let project_id = uuid::Uuid::new_v4().to_string();
        db.create_project(&project_id, "test-project", "Test", None)
            .await
            .expect("create project");

        // Simulate: write to journal but "crash" before DB commit
        let mut journal = SessionJournal::new(&journal_path).expect("create journal");
        let entry = JournalEntry {
            timestamp: Utc::now(),
            project_id: project_id.clone(),
            event: JournalEvent::MessageAdded {
                message_id: "orphan-msg".to_string(),
                role: "user".to_string(),
                content: "Lost message".to_string(),
                token_count: 10,
                model_used: None,
            },
        };
        journal.append(&entry).expect("append");
        drop(journal);

        // Now "restart" and restore
        let config = PersistenceConfig {
            journal_dir: dir.path().to_path_buf(),
            ..Default::default()
        };

        let mut persistence =
            ConversationPersistence::with_journal_path(db.clone(), &journal_path, config)
                .expect("create persistence");

        let result = persistence
            .restore_session(&project_id)
            .await
            .expect("restore");

        // Should have recovered the orphaned message
        assert!(result.crash_recovery);
        assert_eq!(result.from_journal, 1);
        assert_eq!(result.messages.len(), 1);
        assert_eq!(result.messages[0].content, "Lost message");
    }
}
