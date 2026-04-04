//! Decision log management for `.cuttlefish/decisions.jsonl`.

use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::memory::file::MemoryError;

const MAX_ENTRIES: usize = 1000;

/// Type of change recorded in the decision log.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChangeType {
    /// File created.
    Create,
    /// File modified.
    Modify,
    /// File deleted.
    Delete,
    /// Decision made (no file change).
    Decide,
    /// Architecture change.
    Architecture,
    /// Bug fix or workaround.
    Bugfix,
}

/// A single entry in the decision log.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionEntry {
    /// Unique identifier for this entry.
    pub id: Uuid,
    /// When the decision was made.
    pub timestamp: DateTime<Utc>,
    /// Conversation that led to this decision.
    pub conversation_id: String,
    /// Specific message in the conversation.
    pub message_id: String,
    /// File affected (if any).
    pub file_path: Option<String>,
    /// Type of change.
    pub change_type: ChangeType,
    /// Brief summary of the decision.
    pub summary: String,
    /// Reasoning behind the decision.
    pub reasoning: String,
    /// Agent that made the decision.
    pub agent: String,
    /// Confidence level (0.0 to 1.0).
    pub confidence: f32,
}

impl DecisionEntry {
    /// Create a new decision entry with current timestamp.
    pub fn new(
        conversation_id: impl Into<String>,
        message_id: impl Into<String>,
        change_type: ChangeType,
        summary: impl Into<String>,
        reasoning: impl Into<String>,
        agent: impl Into<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            conversation_id: conversation_id.into(),
            message_id: message_id.into(),
            file_path: None,
            change_type,
            summary: summary.into(),
            reasoning: reasoning.into(),
            agent: agent.into(),
            confidence: 1.0,
        }
    }

    /// Set the file path for this entry.
    pub fn with_file(mut self, path: impl Into<String>) -> Self {
        self.file_path = Some(path.into());
        self
    }

    /// Set the confidence level for this entry.
    pub fn with_confidence(mut self, confidence: f32) -> Self {
        self.confidence = confidence.clamp(0.0, 1.0);
        self
    }
}

/// Decision log stored in `.cuttlefish/decisions.jsonl`.
pub struct DecisionLog {
    path: PathBuf,
}

impl DecisionLog {
    /// Create a new decision log at the given path.
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    /// Get the default decision log path for a project root.
    pub fn default_path(project_root: impl AsRef<Path>) -> PathBuf {
        project_root
            .as_ref()
            .join(".cuttlefish")
            .join("decisions.jsonl")
    }

    /// Append an entry to the log (non-blocking via spawn).
    pub fn append(&self, entry: &DecisionEntry) -> Result<(), MemoryError> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)?;

        let json = serde_json::to_string(entry)
            .map_err(|e| MemoryError::Parse(format!("Failed to serialize entry: {e}")))?;
        writeln!(file, "{json}")?;

        self.rotate_if_needed()?;
        Ok(())
    }

    /// Read all entries from the log.
    pub fn read_all(&self) -> Result<Vec<DecisionEntry>, MemoryError> {
        if !self.path.exists() {
            return Ok(Vec::new());
        }

        let file = File::open(&self.path)?;
        let reader = BufReader::new(file);
        let mut entries = Vec::new();

        for line_result in reader.lines() {
            let line = line_result?;
            if line.trim().is_empty() {
                continue;
            }
            let entry: DecisionEntry = serde_json::from_str(&line)
                .map_err(|e| MemoryError::Parse(format!("Failed to parse entry: {e}")))?;
            entries.push(entry);
        }

        Ok(entries)
    }

    /// Find entries by file path.
    pub fn find_by_file(&self, path: &str) -> Result<Vec<DecisionEntry>, MemoryError> {
        let entries = self.read_all()?;
        Ok(entries
            .into_iter()
            .filter(|e| e.file_path.as_deref() == Some(path))
            .collect())
    }

    /// Find entries by conversation ID.
    pub fn find_by_conversation(
        &self,
        conversation_id: &str,
    ) -> Result<Vec<DecisionEntry>, MemoryError> {
        let entries = self.read_all()?;
        Ok(entries
            .into_iter()
            .filter(|e| e.conversation_id == conversation_id)
            .collect())
    }

    /// Count entries in the log.
    pub fn count(&self) -> Result<usize, MemoryError> {
        if !self.path.exists() {
            return Ok(0);
        }

        let file = File::open(&self.path)?;
        let reader = BufReader::new(file);
        Ok(reader
            .lines()
            .map_while(Result::ok)
            .filter(|l| !l.trim().is_empty())
            .count())
    }

    fn rotate_if_needed(&self) -> Result<(), MemoryError> {
        let count = self.count()?;
        if count <= MAX_ENTRIES {
            return Ok(());
        }

        let entries = self.read_all()?;
        let to_archive = entries.len().saturating_sub(MAX_ENTRIES);

        if to_archive == 0 {
            return Ok(());
        }

        let archive_path = self.path.with_extension("jsonl.archive");
        let mut archive_file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&archive_path)?;

        for entry in entries.iter().take(to_archive) {
            let json = serde_json::to_string(entry)
                .map_err(|e| MemoryError::Parse(format!("Failed to serialize entry: {e}")))?;
            writeln!(archive_file, "{json}")?;
        }

        let remaining: Vec<_> = entries.into_iter().skip(to_archive).collect();
        let mut file = File::create(&self.path)?;
        for entry in &remaining {
            let json = serde_json::to_string(entry)
                .map_err(|e| MemoryError::Parse(format!("Failed to serialize entry: {e}")))?;
            writeln!(file, "{json}")?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn test_entry() -> DecisionEntry {
        DecisionEntry::new(
            "conv-123",
            "msg-456",
            ChangeType::Create,
            "Created new module",
            "Needed for memory system",
            "coder",
        )
    }

    #[test]
    fn test_decision_entry_new() {
        let entry = test_entry();
        assert_eq!(entry.conversation_id, "conv-123");
        assert_eq!(entry.message_id, "msg-456");
        assert_eq!(entry.change_type, ChangeType::Create);
        assert!(entry.file_path.is_none());
        assert_eq!(entry.confidence, 1.0);
    }

    #[test]
    fn test_decision_entry_with_file() {
        let entry = test_entry().with_file("src/memory/mod.rs");
        assert_eq!(entry.file_path, Some("src/memory/mod.rs".to_string()));
    }

    #[test]
    fn test_decision_entry_with_confidence() {
        let entry = test_entry().with_confidence(0.8);
        assert!((entry.confidence - 0.8).abs() < f32::EPSILON);
    }

    #[test]
    fn test_confidence_clamped() {
        let entry = test_entry().with_confidence(1.5);
        assert!((entry.confidence - 1.0).abs() < f32::EPSILON);

        let entry = test_entry().with_confidence(-0.5);
        assert!(entry.confidence.abs() < f32::EPSILON);
    }

    #[test]
    fn test_append_and_read() {
        let temp_dir = TempDir::new().expect("create temp dir");
        let log_path = temp_dir.path().join("decisions.jsonl");
        let log = DecisionLog::new(&log_path);

        let entry1 = test_entry().with_file("file1.rs");
        let entry2 = test_entry().with_file("file2.rs");

        log.append(&entry1).expect("append entry1");
        log.append(&entry2).expect("append entry2");

        let entries = log.read_all().expect("read all");
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].file_path, Some("file1.rs".to_string()));
        assert_eq!(entries[1].file_path, Some("file2.rs".to_string()));
    }

    #[test]
    fn test_find_by_file() {
        let temp_dir = TempDir::new().expect("create temp dir");
        let log_path = temp_dir.path().join("decisions.jsonl");
        let log = DecisionLog::new(&log_path);

        log.append(&test_entry().with_file("src/a.rs"))
            .expect("append");
        log.append(&test_entry().with_file("src/b.rs"))
            .expect("append");
        log.append(&test_entry().with_file("src/a.rs"))
            .expect("append");

        let found = log.find_by_file("src/a.rs").expect("find");
        assert_eq!(found.len(), 2);
    }

    #[test]
    fn test_find_by_conversation() {
        let temp_dir = TempDir::new().expect("create temp dir");
        let log_path = temp_dir.path().join("decisions.jsonl");
        let log = DecisionLog::new(&log_path);

        log.append(&DecisionEntry::new(
            "conv-1",
            "msg-1",
            ChangeType::Create,
            "s",
            "r",
            "a",
        ))
        .expect("append");
        log.append(&DecisionEntry::new(
            "conv-2",
            "msg-2",
            ChangeType::Modify,
            "s",
            "r",
            "a",
        ))
        .expect("append");
        log.append(&DecisionEntry::new(
            "conv-1",
            "msg-3",
            ChangeType::Delete,
            "s",
            "r",
            "a",
        ))
        .expect("append");

        let found = log.find_by_conversation("conv-1").expect("find");
        assert_eq!(found.len(), 2);
    }

    #[test]
    fn test_count() {
        let temp_dir = TempDir::new().expect("create temp dir");
        let log_path = temp_dir.path().join("decisions.jsonl");
        let log = DecisionLog::new(&log_path);

        assert_eq!(log.count().expect("count"), 0);

        log.append(&test_entry()).expect("append");
        assert_eq!(log.count().expect("count"), 1);

        log.append(&test_entry()).expect("append");
        assert_eq!(log.count().expect("count"), 2);
    }

    #[test]
    fn test_read_empty_log() {
        let temp_dir = TempDir::new().expect("create temp dir");
        let log_path = temp_dir.path().join("decisions.jsonl");
        let log = DecisionLog::new(&log_path);

        let entries = log.read_all().expect("read all");
        assert!(entries.is_empty());
    }

    #[test]
    fn test_serialization_roundtrip() {
        let entry = test_entry().with_file("test.rs").with_confidence(0.95);

        let json = serde_json::to_string(&entry).expect("serialize");
        let parsed: DecisionEntry = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(parsed.id, entry.id);
        assert_eq!(parsed.conversation_id, entry.conversation_id);
        assert_eq!(parsed.file_path, entry.file_path);
        assert_eq!(parsed.change_type, entry.change_type);
    }

    #[test]
    fn test_change_type_serialization() {
        assert_eq!(
            serde_json::to_string(&ChangeType::Create).expect("serialize"),
            "\"create\""
        );
        assert_eq!(
            serde_json::to_string(&ChangeType::Architecture).expect("serialize"),
            "\"architecture\""
        );
    }
}
