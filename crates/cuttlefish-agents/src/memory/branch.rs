//! State branching for project snapshots.
//!
//! This module provides the ability to fork entire project state before risky changes,
//! including git state, container snapshots, and memory files.

use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

/// Maximum number of branches per project.
pub const MAX_BRANCHES_PER_PROJECT: usize = 10;

/// Branch ID newtype for type safety.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BranchId(pub Uuid);

impl BranchId {
    /// Create a new random branch ID.
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Create from an existing UUID.
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl Default for BranchId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for BranchId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// State branch error.
#[derive(Error, Debug)]
pub enum BranchError {
    /// I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    /// JSON serialization error.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    /// Branch not found.
    #[error("Branch not found: {0}")]
    NotFound(String),
    /// Branch already exists.
    #[error("Branch already exists: {0}")]
    AlreadyExists(String),
    /// Too many branches.
    #[error("Maximum branches ({MAX_BRANCHES_PER_PROJECT}) reached for project")]
    TooManyBranches,
    /// Invalid branch name.
    #[error("Invalid branch name: {0}")]
    InvalidName(String),
    /// Git operation failed.
    #[error("Git operation failed: {0}")]
    GitError(String),
    /// Uncommitted changes present.
    #[error("Uncommitted changes present. Use create_backup=true or commit/stash first")]
    UncommittedChanges,
}

/// A complete state branch capturing git, container, and memory state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateBranch {
    /// Unique identifier for this branch.
    pub id: BranchId,
    /// Human-readable branch name.
    pub name: String,
    /// Project this branch belongs to.
    pub project_id: String,
    /// When the branch was created.
    pub created_at: DateTime<Utc>,
    /// Optional description of why this branch was created.
    pub description: Option<String>,
    /// Git branch or commit reference.
    pub git_ref: String,
    /// Docker container snapshot ID (if applicable).
    pub container_snapshot: Option<String>,
    /// Path to memory file backup relative to .cuttlefish/branches/.
    pub memory_snapshot: String,
    /// Path to decision log backup relative to .cuttlefish/branches/.
    pub decisions_snapshot: String,
}

impl StateBranch {
    /// Create a new state branch.
    pub fn new(
        name: impl Into<String>,
        project_id: impl Into<String>,
        description: Option<String>,
        git_ref: impl Into<String>,
    ) -> Self {
        let name = name.into();
        let id = BranchId::new();
        Self {
            memory_snapshot: format!("{}/memory.md", name),
            decisions_snapshot: format!("{}/decisions.jsonl", name),
            id,
            name,
            project_id: project_id.into(),
            created_at: Utc::now(),
            description,
            git_ref: git_ref.into(),
            container_snapshot: None,
        }
    }

    /// Set the container snapshot ID.
    pub fn with_container_snapshot(mut self, snapshot_id: impl Into<String>) -> Self {
        self.container_snapshot = Some(snapshot_id.into());
        self
    }
}

/// Difference between two branches.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchDiff {
    /// Branch A name.
    pub branch_a: String,
    /// Branch B name.
    pub branch_b: String,
    /// Git diff summary (files changed, insertions, deletions).
    pub git_diff_summary: GitDiffSummary,
    /// Memory file differences.
    pub memory_diff: Vec<String>,
    /// Decision count difference.
    pub decision_count_diff: i32,
}

/// Summary of git differences.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GitDiffSummary {
    /// Number of files changed.
    pub files_changed: usize,
    /// Number of insertions.
    pub insertions: usize,
    /// Number of deletions.
    pub deletions: usize,
    /// List of changed file paths.
    pub changed_files: Vec<String>,
}

/// Branch store managing all branches for a project.
#[derive(Debug)]
pub struct BranchStore {
    /// Project root directory.
    project_root: PathBuf,
    /// Cached branches (loaded on demand).
    branches: HashMap<String, StateBranch>,
}

impl BranchStore {
    /// Create a new branch store for a project.
    pub fn new(project_root: impl AsRef<Path>) -> Self {
        Self {
            project_root: project_root.as_ref().to_path_buf(),
            branches: HashMap::new(),
        }
    }

    /// Get the branches directory path.
    fn branches_dir(&self) -> PathBuf {
        self.project_root.join(".cuttlefish").join("branches")
    }

    /// Get the branch metadata file path.
    fn metadata_path(&self) -> PathBuf {
        self.branches_dir().join("branches.json")
    }

    /// Ensure the branches directory exists.
    fn ensure_branches_dir(&self) -> Result<(), BranchError> {
        let dir = self.branches_dir();
        if !dir.exists() {
            fs::create_dir_all(&dir)?;
        }
        Ok(())
    }

    /// Load branches from disk.
    pub fn load(&mut self) -> Result<(), BranchError> {
        let path = self.metadata_path();
        if !path.exists() {
            self.branches.clear();
            return Ok(());
        }

        let file = File::open(&path)?;
        let reader = BufReader::new(file);
        let branches: Vec<StateBranch> = serde_json::from_reader(reader)?;

        self.branches.clear();
        for branch in branches {
            self.branches.insert(branch.name.clone(), branch);
        }

        Ok(())
    }

    /// Save branches to disk.
    pub fn save(&self) -> Result<(), BranchError> {
        self.ensure_branches_dir()?;

        let path = self.metadata_path();
        let file = File::create(&path)?;
        let writer = BufWriter::new(file);
        let branches: Vec<&StateBranch> = self.branches.values().collect();
        serde_json::to_writer_pretty(writer, &branches)?;

        Ok(())
    }

    /// Validate a branch name.
    fn validate_name(name: &str) -> Result<(), BranchError> {
        if name.is_empty() {
            return Err(BranchError::InvalidName("Name cannot be empty".into()));
        }
        if name.len() > 64 {
            return Err(BranchError::InvalidName(
                "Name too long (max 64 chars)".into(),
            ));
        }
        // Allow alphanumeric, dash, underscore
        if !name
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
        {
            return Err(BranchError::InvalidName(
                "Name can only contain alphanumeric, dash, underscore".into(),
            ));
        }
        Ok(())
    }

    /// Create a new branch, forking the current state.
    ///
    /// Copies memory file and decision log to the branch directory.
    /// Git operations should be handled by the caller (via git2 or command).
    pub fn create_branch(
        &mut self,
        project_id: &str,
        name: &str,
        description: Option<&str>,
        git_ref: &str,
    ) -> Result<StateBranch, BranchError> {
        Self::validate_name(name)?;

        let project_branch_count = self
            .branches
            .values()
            .filter(|b| b.project_id == project_id)
            .count();

        if project_branch_count >= MAX_BRANCHES_PER_PROJECT {
            return Err(BranchError::TooManyBranches);
        }

        if self.branches.contains_key(name) {
            return Err(BranchError::AlreadyExists(name.to_string()));
        }

        let branch_dir = self.branches_dir().join(name);
        fs::create_dir_all(&branch_dir)?;

        self.copy_or_create_file(
            &self.project_root.join(".cuttlefish").join("memory.md"),
            &branch_dir.join("memory.md"),
        )?;

        self.copy_or_create_file(
            &self
                .project_root
                .join(".cuttlefish")
                .join("decisions.jsonl"),
            &branch_dir.join("decisions.jsonl"),
        )?;

        let branch = StateBranch::new(name, project_id, description.map(String::from), git_ref);
        self.branches.insert(name.to_string(), branch.clone());
        self.save()?;

        Ok(branch)
    }

    fn copy_or_create_file(&self, src: &Path, dst: &Path) -> Result<(), BranchError> {
        if src.exists() {
            fs::copy(src, dst)?;
        } else {
            File::create(dst)?;
        }
        Ok(())
    }

    /// Get a branch by name.
    pub fn get_branch(&self, name: &str) -> Result<&StateBranch, BranchError> {
        self.branches
            .get(name)
            .ok_or_else(|| BranchError::NotFound(name.to_string()))
    }

    /// List all branches for a project.
    pub fn list_branches(&self, project_id: &str) -> Vec<&StateBranch> {
        self.branches
            .values()
            .filter(|b| b.project_id == project_id)
            .collect()
    }

    /// Delete a branch and all associated files.
    pub fn delete_branch(&mut self, name: &str) -> Result<(), BranchError> {
        if !self.branches.contains_key(name) {
            return Err(BranchError::NotFound(name.to_string()));
        }

        let branch_dir = self.branches_dir().join(name);
        if branch_dir.exists() {
            fs::remove_dir_all(&branch_dir)?;
        }

        self.branches.remove(name);
        self.save()?;

        Ok(())
    }

    /// Restore a branch, copying its memory and decisions back to the project.
    ///
    /// If `create_backup` is true, creates a backup branch of current state first.
    /// Git operations should be handled by the caller.
    pub fn restore_branch(
        &mut self,
        project_id: &str,
        name: &str,
        create_backup: bool,
    ) -> Result<StateBranch, BranchError> {
        let branch = self.get_branch(name)?.clone();

        if create_backup {
            let backup_name = format!(
                "backup-before-restore-{}",
                Utc::now().format("%Y%m%d-%H%M%S")
            );
            self.create_branch(
                project_id,
                &backup_name,
                Some("Auto-backup before restore"),
                "HEAD",
            )?;
        }

        let branch_dir = self.branches_dir().join(name);
        let cuttlefish_dir = self.project_root.join(".cuttlefish");

        let memory_src = branch_dir.join("memory.md");
        if memory_src.exists() {
            fs::copy(&memory_src, cuttlefish_dir.join("memory.md"))?;
        }

        let decisions_src = branch_dir.join("decisions.jsonl");
        if decisions_src.exists() {
            fs::copy(&decisions_src, cuttlefish_dir.join("decisions.jsonl"))?;
        }

        Ok(branch)
    }

    /// Compare two branches and return their differences.
    pub fn compare_branches(
        &self,
        branch_a: &str,
        branch_b: &str,
    ) -> Result<BranchDiff, BranchError> {
        let _a = self.get_branch(branch_a)?;
        let _b = self.get_branch(branch_b)?;

        let branch_dir_a = self.branches_dir().join(branch_a);
        let branch_dir_b = self.branches_dir().join(branch_b);

        let memory_a = self.read_file_lines(&branch_dir_a.join("memory.md"))?;
        let memory_b = self.read_file_lines(&branch_dir_b.join("memory.md"))?;
        let memory_diff = self.simple_diff(&memory_a, &memory_b);

        let decisions_a = self.count_lines(&branch_dir_a.join("decisions.jsonl"))?;
        let decisions_b = self.count_lines(&branch_dir_b.join("decisions.jsonl"))?;

        Ok(BranchDiff {
            branch_a: branch_a.to_string(),
            branch_b: branch_b.to_string(),
            git_diff_summary: GitDiffSummary::default(),
            memory_diff,
            decision_count_diff: decisions_b as i32 - decisions_a as i32,
        })
    }

    /// Read file as lines.
    fn read_file_lines(&self, path: &Path) -> Result<Vec<String>, BranchError> {
        if !path.exists() {
            return Ok(Vec::new());
        }
        let content = fs::read_to_string(path)?;
        Ok(content.lines().map(String::from).collect())
    }

    /// Count lines in a file.
    fn count_lines(&self, path: &Path) -> Result<usize, BranchError> {
        if !path.exists() {
            return Ok(0);
        }
        let content = fs::read_to_string(path)?;
        Ok(content.lines().count())
    }

    fn simple_diff(&self, lines_a: &[String], lines_b: &[String]) -> Vec<String> {
        let mut diff = Vec::new();

        for line in lines_a {
            if !lines_b.contains(line) && !line.trim().is_empty() {
                diff.push(format!("- {}", line));
            }
        }

        for line in lines_b {
            if !lines_a.contains(line) && !line.trim().is_empty() {
                diff.push(format!("+ {}", line));
            }
        }

        diff
    }

    /// Get branch count for a project.
    pub fn branch_count(&self, project_id: &str) -> usize {
        self.branches
            .values()
            .filter(|b| b.project_id == project_id)
            .count()
    }

    /// Check if a branch exists.
    pub fn branch_exists(&self, name: &str) -> bool {
        self.branches.contains_key(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_test_project() -> (TempDir, BranchStore) {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let cuttlefish_dir = temp_dir.path().join(".cuttlefish");
        fs::create_dir_all(&cuttlefish_dir).expect("Failed to create .cuttlefish dir");

        // Create sample memory file
        let memory_content = "# Project Memory: Test\n\n## Summary\nTest project\n";
        fs::write(cuttlefish_dir.join("memory.md"), memory_content)
            .expect("Failed to write memory file");

        // Create sample decisions file
        let decisions_content = r#"{"id":"00000000-0000-0000-0000-000000000001","timestamp":"2024-01-01T00:00:00Z","conversation_id":"conv1","message_id":"msg1","file_path":null,"change_type":"decide","summary":"Test decision","reasoning":"Test reason","agent":"test","confidence":1.0}"#;
        fs::write(cuttlefish_dir.join("decisions.jsonl"), decisions_content)
            .expect("Failed to write decisions file");

        let store = BranchStore::new(temp_dir.path());
        (temp_dir, store)
    }

    #[test]
    fn test_branch_id_creation() {
        let id1 = BranchId::new();
        let id2 = BranchId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_branch_id_display() {
        let uuid =
            Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").expect("Failed to parse UUID");
        let id = BranchId::from_uuid(uuid);
        assert_eq!(id.to_string(), "550e8400-e29b-41d4-a716-446655440000");
    }

    #[test]
    fn test_state_branch_creation() {
        let branch = StateBranch::new(
            "test-branch",
            "project-1",
            Some("Test description".into()),
            "main",
        );

        assert_eq!(branch.name, "test-branch");
        assert_eq!(branch.project_id, "project-1");
        assert_eq!(branch.description, Some("Test description".to_string()));
        assert_eq!(branch.git_ref, "main");
        assert!(branch.container_snapshot.is_none());
        assert_eq!(branch.memory_snapshot, "test-branch/memory.md");
        assert_eq!(branch.decisions_snapshot, "test-branch/decisions.jsonl");
    }

    #[test]
    fn test_state_branch_with_container() {
        let branch =
            StateBranch::new("test", "proj", None, "main").with_container_snapshot("snapshot-123");

        assert_eq!(branch.container_snapshot, Some("snapshot-123".to_string()));
    }

    #[test]
    fn test_validate_name_valid() {
        assert!(BranchStore::validate_name("valid-name").is_ok());
        assert!(BranchStore::validate_name("valid_name").is_ok());
        assert!(BranchStore::validate_name("ValidName123").is_ok());
        assert!(BranchStore::validate_name("a").is_ok());
    }

    #[test]
    fn test_validate_name_invalid() {
        assert!(BranchStore::validate_name("").is_err());
        assert!(BranchStore::validate_name("invalid name").is_err());
        assert!(BranchStore::validate_name("invalid/name").is_err());
        assert!(BranchStore::validate_name("invalid.name").is_err());

        // Too long
        let long_name = "a".repeat(65);
        assert!(BranchStore::validate_name(&long_name).is_err());
    }

    #[test]
    fn test_create_branch() {
        let (_temp_dir, mut store) = setup_test_project();

        let branch = store
            .create_branch("project-1", "test-branch", Some("Test"), "main")
            .expect("Failed to create branch");

        assert_eq!(branch.name, "test-branch");
        assert_eq!(branch.project_id, "project-1");
        assert!(store.branch_exists("test-branch"));
    }

    #[test]
    fn test_create_branch_copies_files() {
        let (temp_dir, mut store) = setup_test_project();

        store
            .create_branch("project-1", "test-branch", None, "main")
            .expect("Failed to create branch");

        // Check that files were copied
        let branch_dir = temp_dir
            .path()
            .join(".cuttlefish")
            .join("branches")
            .join("test-branch");
        assert!(branch_dir.join("memory.md").exists());
        assert!(branch_dir.join("decisions.jsonl").exists());

        // Verify content
        let memory_content =
            fs::read_to_string(branch_dir.join("memory.md")).expect("Failed to read memory");
        assert!(memory_content.contains("Test project"));
    }

    #[test]
    fn test_create_branch_duplicate() {
        let (_temp_dir, mut store) = setup_test_project();

        store
            .create_branch("project-1", "test-branch", None, "main")
            .expect("Failed to create branch");

        let result = store.create_branch("project-1", "test-branch", None, "main");
        assert!(matches!(result, Err(BranchError::AlreadyExists(_))));
    }

    #[test]
    fn test_create_branch_limit() {
        let (_temp_dir, mut store) = setup_test_project();

        // Create max branches
        for i in 0..MAX_BRANCHES_PER_PROJECT {
            store
                .create_branch("project-1", &format!("branch-{}", i), None, "main")
                .expect("Failed to create branch");
        }

        // Try to create one more
        let result = store.create_branch("project-1", "one-too-many", None, "main");
        assert!(matches!(result, Err(BranchError::TooManyBranches)));
    }

    #[test]
    fn test_get_branch() {
        let (_temp_dir, mut store) = setup_test_project();

        store
            .create_branch("project-1", "test-branch", Some("Description"), "main")
            .expect("Failed to create branch");

        let branch = store
            .get_branch("test-branch")
            .expect("Failed to get branch");
        assert_eq!(branch.name, "test-branch");
        assert_eq!(branch.description, Some("Description".to_string()));
    }

    #[test]
    fn test_get_branch_not_found() {
        let (_temp_dir, store) = setup_test_project();

        let result = store.get_branch("nonexistent");
        assert!(matches!(result, Err(BranchError::NotFound(_))));
    }

    #[test]
    fn test_list_branches() {
        let (_temp_dir, mut store) = setup_test_project();

        store
            .create_branch("project-1", "branch-a", None, "main")
            .expect("create a");
        store
            .create_branch("project-1", "branch-b", None, "main")
            .expect("create b");
        store
            .create_branch("project-2", "branch-c", None, "main")
            .expect("create c");

        let project1_branches = store.list_branches("project-1");
        assert_eq!(project1_branches.len(), 2);

        let project2_branches = store.list_branches("project-2");
        assert_eq!(project2_branches.len(), 1);
    }

    #[test]
    fn test_delete_branch() {
        let (temp_dir, mut store) = setup_test_project();

        store
            .create_branch("project-1", "test-branch", None, "main")
            .expect("Failed to create branch");

        let branch_dir = temp_dir
            .path()
            .join(".cuttlefish")
            .join("branches")
            .join("test-branch");
        assert!(branch_dir.exists());

        store
            .delete_branch("test-branch")
            .expect("Failed to delete branch");

        assert!(!store.branch_exists("test-branch"));
        assert!(!branch_dir.exists());
    }

    #[test]
    fn test_delete_branch_not_found() {
        let (_temp_dir, mut store) = setup_test_project();

        let result = store.delete_branch("nonexistent");
        assert!(matches!(result, Err(BranchError::NotFound(_))));
    }

    #[test]
    fn test_restore_branch() {
        let (temp_dir, mut store) = setup_test_project();

        // Create branch
        store
            .create_branch("project-1", "test-branch", None, "main")
            .expect("Failed to create branch");

        // Modify current memory file
        let memory_path = temp_dir.path().join(".cuttlefish").join("memory.md");
        fs::write(&memory_path, "# Modified content").expect("Failed to modify memory");

        // Restore branch
        store
            .restore_branch("project-1", "test-branch", false)
            .expect("Failed to restore branch");

        // Verify memory was restored
        let restored_content = fs::read_to_string(&memory_path).expect("Failed to read memory");
        assert!(restored_content.contains("Test project"));
    }

    #[test]
    fn test_restore_branch_with_backup() {
        let (_temp_dir, mut store) = setup_test_project();

        // Create branch
        store
            .create_branch("project-1", "test-branch", None, "main")
            .expect("Failed to create branch");

        // Restore with backup
        store
            .restore_branch("project-1", "test-branch", true)
            .expect("Failed to restore branch");

        // Should have 2 branches now (original + backup)
        let branches = store.list_branches("project-1");
        assert_eq!(branches.len(), 2);

        // One should be a backup
        let backup_exists = branches
            .iter()
            .any(|b| b.name.starts_with("backup-before-restore-"));
        assert!(backup_exists);
    }

    #[test]
    fn test_compare_branches() {
        let (temp_dir, mut store) = setup_test_project();

        // Create first branch
        store
            .create_branch("project-1", "branch-a", None, "main")
            .expect("Failed to create branch-a");

        // Modify memory and create second branch
        let memory_path = temp_dir.path().join(".cuttlefish").join("memory.md");
        fs::write(
            &memory_path,
            "# Modified\n\n## Summary\nDifferent content\n",
        )
        .expect("Failed to modify memory");

        store
            .create_branch("project-1", "branch-b", None, "main")
            .expect("Failed to create branch-b");

        // Compare
        let diff = store
            .compare_branches("branch-a", "branch-b")
            .expect("Failed to compare branches");

        assert_eq!(diff.branch_a, "branch-a");
        assert_eq!(diff.branch_b, "branch-b");
        assert!(!diff.memory_diff.is_empty());
    }

    #[test]
    fn test_save_and_load() {
        let (temp_dir, mut store) = setup_test_project();

        store
            .create_branch("project-1", "test-branch", Some("Test"), "main")
            .expect("Failed to create branch");

        // Create new store and load
        let mut store2 = BranchStore::new(temp_dir.path());
        store2.load().expect("Failed to load");

        assert!(store2.branch_exists("test-branch"));
        let branch = store2
            .get_branch("test-branch")
            .expect("Failed to get branch");
        assert_eq!(branch.description, Some("Test".to_string()));
    }

    #[test]
    fn test_branch_count() {
        let (_temp_dir, mut store) = setup_test_project();

        assert_eq!(store.branch_count("project-1"), 0);

        store
            .create_branch("project-1", "branch-1", None, "main")
            .expect("create 1");
        assert_eq!(store.branch_count("project-1"), 1);

        store
            .create_branch("project-1", "branch-2", None, "main")
            .expect("create 2");
        assert_eq!(store.branch_count("project-1"), 2);

        store
            .create_branch("project-2", "branch-3", None, "main")
            .expect("create 3");
        assert_eq!(store.branch_count("project-1"), 2);
        assert_eq!(store.branch_count("project-2"), 1);
    }

    #[test]
    fn test_branch_error_display() {
        let err = BranchError::NotFound("test".to_string());
        assert_eq!(err.to_string(), "Branch not found: test");

        let err = BranchError::TooManyBranches;
        assert!(err.to_string().contains("Maximum branches"));
    }
}
