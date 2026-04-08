//! JSONL journal for crash recovery.
//!
//! Writes session events to a file using newline-delimited JSON.
//! Events are flushed immediately to ensure durability.

use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, warn};

/// A journal entry representing a session event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JournalEntry {
    /// Timestamp of the event.
    pub timestamp: DateTime<Utc>,
    /// Project ID this event belongs to.
    pub project_id: String,
    /// Type of event.
    pub event: JournalEvent,
}

/// Types of events that can be journaled.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum JournalEvent {
    /// A message was added to the conversation.
    MessageAdded {
        /// Message ID (UUID).
        message_id: String,
        /// Role: user, assistant, system.
        role: String,
        /// Message content.
        content: String,
        /// Token count for this message.
        token_count: i64,
        /// Model used (if assistant message).
        model_used: Option<String>,
    },
    /// Context was compacted.
    ContextCompacted {
        /// Number of messages removed.
        messages_removed: usize,
        /// Tokens before compaction.
        tokens_before: usize,
        /// Tokens after compaction.
        tokens_after: usize,
    },
    /// Session was restored from database.
    SessionRestored {
        /// Number of messages restored.
        message_count: usize,
    },
    /// Session was archived to database.
    SessionArchived {
        /// Number of messages archived.
        message_count: usize,
    },
}

/// JSONL journal for session events.
pub struct SessionJournal {
    /// Path to the journal file.
    path: PathBuf,
    /// Buffered writer for appending.
    writer: Option<BufWriter<File>>,
    /// Maximum file size before rotation (in bytes).
    max_size: u64,
    /// Number of rotated files to keep.
    max_rotations: usize,
}

impl SessionJournal {
    /// Create a new journal at the given path.
    ///
    /// Creates the file if it doesn't exist, or opens for append if it does.
    pub fn new(path: impl AsRef<Path>) -> std::io::Result<Self> {
        Self::with_config(path, 1024 * 1024, 10) // 1MB, 10 files
    }

    /// Create a journal with custom rotation settings.
    ///
    /// # Arguments
    /// * `path` - Path to the journal file
    /// * `max_size` - Maximum file size before rotation (bytes)
    /// * `max_rotations` - Number of rotated files to keep
    pub fn with_config(
        path: impl AsRef<Path>,
        max_size: u64,
        max_rotations: usize,
    ) -> std::io::Result<Self> {
        let path = path.as_ref().to_path_buf();

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)?;

        Ok(Self {
            path,
            writer: Some(BufWriter::new(file)),
            max_size,
            max_rotations,
        })
    }

    /// Append an entry to the journal.
    ///
    /// Entry is serialized as JSON and flushed immediately.
    pub fn append(&mut self, entry: &JournalEntry) -> std::io::Result<()> {
        // Check if rotation is needed
        if self.needs_rotation()? {
            self.rotate()?;
        }

        if let Some(writer) = &mut self.writer {
            serde_json::to_writer(&mut *writer, entry)?;
            writer.write_all(b"\n")?;
            writer.flush()?;
            debug!(project_id = %entry.project_id, "Journal entry written");
        }

        Ok(())
    }

    /// Read all entries from the journal.
    ///
    /// Used for crash recovery to replay events.
    pub fn read_all(&self) -> std::io::Result<Vec<JournalEntry>> {
        if !self.path.exists() {
            return Ok(Vec::new());
        }

        let file = File::open(&self.path)?;
        let reader = BufReader::new(file);
        let mut entries = Vec::new();

        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }

            match serde_json::from_str::<JournalEntry>(&line) {
                Ok(entry) => entries.push(entry),
                Err(e) => {
                    warn!(line = line_num + 1, error = %e, "Skipping corrupted journal entry");
                }
            }
        }

        info!(count = entries.len(), "Read journal entries");
        Ok(entries)
    }

    /// Read entries for a specific project.
    pub fn read_for_project(&self, project_id: &str) -> std::io::Result<Vec<JournalEntry>> {
        let all = self.read_all()?;
        Ok(all
            .into_iter()
            .filter(|e| e.project_id == project_id)
            .collect())
    }

    /// Clear the journal (after successful database sync).
    pub fn clear(&mut self) -> std::io::Result<()> {
        // Close existing writer
        self.writer = None;

        // Truncate the file
        let file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(&self.path)?;

        self.writer = Some(BufWriter::new(file));
        info!(path = %self.path.display(), "Journal cleared");
        Ok(())
    }

    /// Check if the journal file needs rotation.
    fn needs_rotation(&self) -> std::io::Result<bool> {
        if !self.path.exists() {
            return Ok(false);
        }
        let metadata = std::fs::metadata(&self.path)?;
        Ok(metadata.len() >= self.max_size)
    }

    /// Rotate the journal file.
    fn rotate(&mut self) -> std::io::Result<()> {
        // Close current writer
        self.writer = None;

        // Delete oldest rotation if at limit
        let oldest = self.path.with_extension(format!("jsonl.{}", self.max_rotations));
        if oldest.exists() {
            std::fs::remove_file(&oldest)?;
        }

        // Shift existing rotations
        for i in (1..self.max_rotations).rev() {
            let old = self.path.with_extension(format!("jsonl.{}", i));
            let new = self.path.with_extension(format!("jsonl.{}", i + 1));
            if old.exists() {
                std::fs::rename(&old, &new)?;
            }
        }

        // Rotate current file
        let rotated = self.path.with_extension("jsonl.1");
        std::fs::rename(&self.path, &rotated)?;

        // Create new file
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)?;

        self.writer = Some(BufWriter::new(file));
        info!(path = %self.path.display(), "Journal rotated");
        Ok(())
    }

    /// Get the journal file path.
    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for SessionJournal {
    fn drop(&mut self) {
        if let Some(writer) = &mut self.writer {
            if let Err(e) = writer.flush() {
                error!(error = %e, "Failed to flush journal on drop");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn temp_journal() -> (SessionJournal, TempDir) {
        let dir = TempDir::new().expect("temp dir");
        let path = dir.path().join("test.jsonl");
        let journal = SessionJournal::new(&path).expect("create journal");
        (journal, dir)
    }

    #[test]
    fn test_append_and_read() {
        let (mut journal, _dir) = temp_journal();

        let entry = JournalEntry {
            timestamp: Utc::now(),
            project_id: "proj-1".to_string(),
            event: JournalEvent::MessageAdded {
                message_id: "msg-1".to_string(),
                role: "user".to_string(),
                content: "Hello".to_string(),
                token_count: 10,
                model_used: None,
            },
        };

        journal.append(&entry).expect("append");

        let entries = journal.read_all().expect("read");
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].project_id, "proj-1");
    }

    #[test]
    fn test_read_for_project() {
        let (mut journal, _dir) = temp_journal();

        for (project, i) in [("proj-1", 1), ("proj-2", 2), ("proj-1", 3)] {
            let entry = JournalEntry {
                timestamp: Utc::now(),
                project_id: project.to_string(),
                event: JournalEvent::MessageAdded {
                    message_id: format!("msg-{i}"),
                    role: "user".to_string(),
                    content: "Hello".to_string(),
                    token_count: 10,
                    model_used: None,
                },
            };
            journal.append(&entry).expect("append");
        }

        let proj1_entries = journal.read_for_project("proj-1").expect("read");
        assert_eq!(proj1_entries.len(), 2);

        let proj2_entries = journal.read_for_project("proj-2").expect("read");
        assert_eq!(proj2_entries.len(), 1);
    }

    #[test]
    fn test_clear() {
        let (mut journal, _dir) = temp_journal();

        let entry = JournalEntry {
            timestamp: Utc::now(),
            project_id: "proj-1".to_string(),
            event: JournalEvent::SessionRestored { message_count: 5 },
        };
        journal.append(&entry).expect("append");

        assert_eq!(journal.read_all().expect("read").len(), 1);

        journal.clear().expect("clear");

        assert_eq!(journal.read_all().expect("read").len(), 0);
    }

    #[test]
    fn test_rotation() {
        let dir = TempDir::new().expect("temp dir");
        let path = dir.path().join("test.jsonl");

        // Small max size to trigger rotation
        let mut journal = SessionJournal::with_config(&path, 100, 3).expect("create");

        // Write enough entries to trigger rotation
        for i in 0..20 {
            let entry = JournalEntry {
                timestamp: Utc::now(),
                project_id: "proj-1".to_string(),
                event: JournalEvent::MessageAdded {
                    message_id: format!("msg-{i}"),
                    role: "user".to_string(),
                    content: "This is a longer message to fill up the journal quickly".to_string(),
                    token_count: 100,
                    model_used: None,
                },
            };
            journal.append(&entry).expect("append");
        }

        // Check that rotation files exist
        let rotated = path.with_extension("jsonl.1");
        assert!(rotated.exists(), "Rotation file should exist");
    }
}
