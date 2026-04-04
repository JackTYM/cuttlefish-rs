//! In-memory indexes for fast decision lookup.

use std::collections::{BTreeMap, HashMap};

use chrono::{DateTime, Utc};

use crate::memory::file::MemoryError;
use crate::memory::log::{DecisionEntry, DecisionLog};

/// In-memory index for fast decision lookups.
pub struct DecisionIndex {
    file_index: HashMap<String, Vec<DecisionEntry>>,
    conversation_index: HashMap<String, Vec<DecisionEntry>>,
    time_index: BTreeMap<DateTime<Utc>, Vec<DecisionEntry>>,
    all_entries: Vec<DecisionEntry>,
}

impl DecisionIndex {
    /// Create a new empty index.
    pub fn new() -> Self {
        Self {
            file_index: HashMap::new(),
            conversation_index: HashMap::new(),
            time_index: BTreeMap::new(),
            all_entries: Vec::new(),
        }
    }

    /// Build index from a decision log.
    pub fn from_log(log: &DecisionLog) -> Result<Self, MemoryError> {
        let entries = log.read_all()?;
        let mut index = Self::new();
        for entry in entries {
            index.add_entry(entry);
        }
        Ok(index)
    }

    /// Add an entry to the index.
    pub fn add_entry(&mut self, entry: DecisionEntry) {
        if let Some(ref path) = entry.file_path {
            self.file_index
                .entry(path.clone())
                .or_default()
                .push(entry.clone());
        }

        self.conversation_index
            .entry(entry.conversation_id.clone())
            .or_default()
            .push(entry.clone());

        self.time_index
            .entry(entry.timestamp)
            .or_default()
            .push(entry.clone());

        self.all_entries.push(entry);
    }

    /// Find entries by file path.
    pub fn find_by_file(&self, path: &str) -> Vec<&DecisionEntry> {
        self.file_index
            .get(path)
            .map(|v| v.iter().collect())
            .unwrap_or_default()
    }

    /// Find entries by conversation ID.
    pub fn find_by_conversation(&self, conversation_id: &str) -> Vec<&DecisionEntry> {
        self.conversation_index
            .get(conversation_id)
            .map(|v| v.iter().collect())
            .unwrap_or_default()
    }

    /// Find entries in a time range (inclusive).
    pub fn find_in_range(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> Vec<&DecisionEntry> {
        self.time_index
            .range(start..=end)
            .flat_map(|(_, entries)| entries.iter())
            .collect()
    }

    /// Fuzzy search on summary and reasoning fields.
    pub fn search(&self, query: &str) -> Vec<&DecisionEntry> {
        let query_lower = query.to_lowercase();
        let query_terms: Vec<&str> = query_lower.split_whitespace().collect();

        if query_terms.is_empty() {
            return Vec::new();
        }

        self.all_entries
            .iter()
            .filter(|entry| {
                let summary_lower = entry.summary.to_lowercase();
                let reasoning_lower = entry.reasoning.to_lowercase();
                let agent_lower = entry.agent.to_lowercase();

                query_terms.iter().any(|term| {
                    summary_lower.contains(term)
                        || reasoning_lower.contains(term)
                        || agent_lower.contains(term)
                })
            })
            .collect()
    }

    /// Get all entries.
    pub fn all(&self) -> &[DecisionEntry] {
        &self.all_entries
    }

    /// Get the number of indexed entries.
    pub fn len(&self) -> usize {
        self.all_entries.len()
    }

    /// Check if the index is empty.
    pub fn is_empty(&self) -> bool {
        self.all_entries.is_empty()
    }

    /// Get unique file paths in the index.
    pub fn file_paths(&self) -> Vec<&str> {
        self.file_index.keys().map(|s| s.as_str()).collect()
    }

    /// Get unique conversation IDs in the index.
    pub fn conversation_ids(&self) -> Vec<&str> {
        self.conversation_index.keys().map(|s| s.as_str()).collect()
    }
}

impl Default for DecisionIndex {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::log::ChangeType;
    use chrono::Duration;
    use tempfile::TempDir;

    fn test_entry(conv_id: &str, summary: &str) -> DecisionEntry {
        DecisionEntry::new(
            conv_id,
            "msg-1",
            ChangeType::Create,
            summary,
            "reasoning",
            "coder",
        )
    }

    #[test]
    fn test_new_index_is_empty() {
        let index = DecisionIndex::new();
        assert!(index.is_empty());
        assert_eq!(index.len(), 0);
    }

    #[test]
    fn test_add_entry() {
        let mut index = DecisionIndex::new();
        let entry = test_entry("conv-1", "Created module").with_file("src/lib.rs");
        index.add_entry(entry);

        assert_eq!(index.len(), 1);
        assert!(!index.is_empty());
    }

    #[test]
    fn test_find_by_file() {
        let mut index = DecisionIndex::new();
        index.add_entry(test_entry("conv-1", "Created A").with_file("src/a.rs"));
        index.add_entry(test_entry("conv-2", "Created B").with_file("src/b.rs"));
        index.add_entry(test_entry("conv-3", "Modified A").with_file("src/a.rs"));

        let found = index.find_by_file("src/a.rs");
        assert_eq!(found.len(), 2);

        let found = index.find_by_file("src/b.rs");
        assert_eq!(found.len(), 1);

        let found = index.find_by_file("src/c.rs");
        assert!(found.is_empty());
    }

    #[test]
    fn test_find_by_conversation() {
        let mut index = DecisionIndex::new();
        index.add_entry(test_entry("conv-1", "First"));
        index.add_entry(test_entry("conv-2", "Second"));
        index.add_entry(test_entry("conv-1", "Third"));

        let found = index.find_by_conversation("conv-1");
        assert_eq!(found.len(), 2);

        let found = index.find_by_conversation("conv-2");
        assert_eq!(found.len(), 1);

        let found = index.find_by_conversation("conv-3");
        assert!(found.is_empty());
    }

    #[test]
    fn test_find_in_range() {
        let mut index = DecisionIndex::new();
        let now = Utc::now();

        let mut entry1 = test_entry("conv-1", "Old entry");
        entry1.timestamp = now - Duration::hours(2);
        index.add_entry(entry1);

        let mut entry2 = test_entry("conv-2", "Recent entry");
        entry2.timestamp = now - Duration::minutes(30);
        index.add_entry(entry2);

        let mut entry3 = test_entry("conv-3", "Very recent");
        entry3.timestamp = now;
        index.add_entry(entry3);

        let found = index.find_in_range(now - Duration::hours(1), now);
        assert_eq!(found.len(), 2);

        let found = index.find_in_range(now - Duration::hours(3), now - Duration::hours(1));
        assert_eq!(found.len(), 1);
    }

    #[test]
    fn test_search_summary() {
        let mut index = DecisionIndex::new();
        index.add_entry(test_entry("conv-1", "Created memory module"));
        index.add_entry(test_entry("conv-2", "Fixed database bug"));
        index.add_entry(test_entry("conv-3", "Updated memory tests"));

        let found = index.search("memory");
        assert_eq!(found.len(), 2);

        let found = index.search("database");
        assert_eq!(found.len(), 1);

        let found = index.search("nonexistent");
        assert!(found.is_empty());
    }

    #[test]
    fn test_search_case_insensitive() {
        let mut index = DecisionIndex::new();
        index.add_entry(test_entry("conv-1", "Created MEMORY module"));

        let found = index.search("memory");
        assert_eq!(found.len(), 1);

        let found = index.search("MEMORY");
        assert_eq!(found.len(), 1);
    }

    #[test]
    fn test_search_multiple_terms() {
        let mut index = DecisionIndex::new();
        index.add_entry(test_entry("conv-1", "Created memory module"));
        index.add_entry(test_entry("conv-2", "Fixed database bug"));

        let found = index.search("memory database");
        assert_eq!(found.len(), 2);
    }

    #[test]
    fn test_search_empty_query() {
        let mut index = DecisionIndex::new();
        index.add_entry(test_entry("conv-1", "Created module"));

        let found = index.search("");
        assert!(found.is_empty());

        let found = index.search("   ");
        assert!(found.is_empty());
    }

    #[test]
    fn test_from_log() {
        let temp_dir = TempDir::new().expect("create temp dir");
        let log_path = temp_dir.path().join("decisions.jsonl");
        let log = DecisionLog::new(&log_path);

        log.append(&test_entry("conv-1", "First").with_file("a.rs"))
            .expect("append");
        log.append(&test_entry("conv-2", "Second").with_file("b.rs"))
            .expect("append");

        let index = DecisionIndex::from_log(&log).expect("build index");
        assert_eq!(index.len(), 2);
        assert_eq!(index.find_by_file("a.rs").len(), 1);
    }

    #[test]
    fn test_file_paths() {
        let mut index = DecisionIndex::new();
        index.add_entry(test_entry("conv-1", "A").with_file("src/a.rs"));
        index.add_entry(test_entry("conv-2", "B").with_file("src/b.rs"));
        index.add_entry(test_entry("conv-3", "A2").with_file("src/a.rs"));

        let paths = index.file_paths();
        assert_eq!(paths.len(), 2);
        assert!(paths.contains(&"src/a.rs"));
        assert!(paths.contains(&"src/b.rs"));
    }

    #[test]
    fn test_conversation_ids() {
        let mut index = DecisionIndex::new();
        index.add_entry(test_entry("conv-1", "A"));
        index.add_entry(test_entry("conv-2", "B"));
        index.add_entry(test_entry("conv-1", "C"));

        let ids = index.conversation_ids();
        assert_eq!(ids.len(), 2);
        assert!(ids.contains(&"conv-1"));
        assert!(ids.contains(&"conv-2"));
    }
}
