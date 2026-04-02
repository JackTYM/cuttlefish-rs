//! Git version control implementation using git2.

use async_trait::async_trait;
use cuttlefish_core::{
    error::VcsError,
    traits::vcs::{CommitInfo, VersionControl},
};
use std::path::{Path, PathBuf};
use tracing::{debug, info};

/// Result type for VCS operations.
type VcsResult<T> = Result<T, VcsError>;

/// Git repository operations via git2.
///
/// Wraps the `git2` crate to provide async-compatible Git operations.
/// All blocking git2 calls are offloaded to `tokio::task::spawn_blocking`.
pub struct GitRepository {
    /// Personal access token for authentication (optional).
    pat: Option<String>,
}

impl GitRepository {
    /// Create a new Git repository handler without authentication.
    pub fn new() -> Self {
        Self { pat: None }
    }

    /// Create a new Git repository handler with a PAT for authenticated operations.
    pub fn with_pat(token: impl Into<String>) -> Self {
        Self {
            pat: Some(token.into()),
        }
    }
}

impl Default for GitRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl VersionControl for GitRepository {
    async fn clone_repo(&self, url: &str, path: &Path) -> VcsResult<()> {
        debug!("Cloning {} to {}", url, path.display());
        let url = url.to_string();
        let path = path.to_owned();
        let pat = self.pat.clone();

        tokio::task::spawn_blocking(move || {
            let mut fetch_opts = git2::FetchOptions::new();
            if let Some(token) = &pat {
                let token = token.clone();
                let mut callbacks = git2::RemoteCallbacks::new();
                callbacks.credentials(move |_url, _username, _allowed| {
                    git2::Cred::userpass_plaintext("oauth2", &token)
                });
                fetch_opts.remote_callbacks(callbacks);
            }
            let mut builder = git2::build::RepoBuilder::new();
            builder.fetch_options(fetch_opts);
            builder
                .clone(&url, &path)
                .map_err(|e| VcsError(format!("Clone failed: {e}")))?;
            Ok::<_, VcsError>(())
        })
        .await
        .map_err(|e| VcsError(format!("Task join error: {e}")))?
    }

    async fn checkout_branch(&self, path: &Path, branch: &str, create: bool) -> VcsResult<()> {
        let path = path.to_owned();
        let branch = branch.to_string();

        tokio::task::spawn_blocking(move || {
            let repo = git2::Repository::open(&path)
                .map_err(|e| VcsError(format!("Failed to open repo: {e}")))?;

            if create {
                // Create a new branch from HEAD
                let head = repo
                    .head()
                    .map_err(|e| VcsError(format!("Failed to get HEAD: {e}")))?;
                let head_commit = head
                    .peel_to_commit()
                    .map_err(|e| VcsError(format!("Failed to get HEAD commit: {e}")))?;
                let branch_ref = repo
                    .branch(&branch, &head_commit, false)
                    .map_err(|e| VcsError(format!("Failed to create branch {branch}: {e}")))?;

                // Set HEAD to new branch
                let ref_name = branch_ref
                    .get()
                    .name()
                    .ok_or_else(|| VcsError("Branch ref name is not valid UTF-8".to_string()))?;
                repo.set_head(ref_name)
                    .map_err(|e| VcsError(format!("Failed to set HEAD: {e}")))?;

                // Checkout (update working directory)
                repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))
                    .map_err(|e| VcsError(format!("Failed to checkout: {e}")))?;
            } else {
                // Switch to existing branch
                let branch_ref = repo
                    .find_branch(&branch, git2::BranchType::Local)
                    .map_err(|e| VcsError(format!("Branch {branch} not found: {e}")))?;

                let ref_name = branch_ref
                    .get()
                    .name()
                    .ok_or_else(|| VcsError("Branch ref name is not valid UTF-8".to_string()))?;
                repo.set_head(ref_name)
                    .map_err(|e| VcsError(format!("Failed to set HEAD: {e}")))?;

                repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))
                    .map_err(|e| VcsError(format!("Failed to checkout: {e}")))?;
            }

            Ok::<_, VcsError>(())
        })
        .await
        .map_err(|e| VcsError(format!("Task join: {e}")))?
    }

    async fn commit(&self, path: &Path, message: &str, files: &[PathBuf]) -> VcsResult<String> {
        let path = path.to_owned();
        let message = message.to_string();
        let files = files.to_vec();

        tokio::task::spawn_blocking(move || {
            let repo = git2::Repository::open(&path)
                .map_err(|e| VcsError(format!("Failed to open repo: {e}")))?;

            let mut index = repo
                .index()
                .map_err(|e| VcsError(format!("Failed to get index: {e}")))?;

            // Stage specified files (or all if empty)
            if files.is_empty() {
                index
                    .add_all(["."].iter(), git2::IndexAddOption::DEFAULT, None)
                    .map_err(|e| VcsError(format!("Failed to stage all: {e}")))?;
            } else {
                for file in &files {
                    index.add_path(file).map_err(|e| {
                        VcsError(format!("Failed to stage {}: {e}", file.display()))
                    })?;
                }
            }
            index
                .write()
                .map_err(|e| VcsError(format!("Failed to write index: {e}")))?;

            let tree_oid = index
                .write_tree()
                .map_err(|e| VcsError(format!("Failed to write tree: {e}")))?;
            let tree = repo
                .find_tree(tree_oid)
                .map_err(|e| VcsError(format!("Failed to find tree: {e}")))?;

            let sig = repo.signature().unwrap_or_else(|_| {
                git2::Signature::now("Cuttlefish", "cuttlefish@ai")
                    .expect("creating default signature should not fail")
            });

            // Get parent commit (if HEAD exists)
            let parents = match repo.head() {
                Ok(head_ref) => {
                    let head_commit = head_ref
                        .peel_to_commit()
                        .map_err(|e| VcsError(format!("Failed to get parent commit: {e}")))?;
                    vec![head_commit]
                }
                Err(_) => vec![], // Initial commit
            };

            let parent_refs: Vec<&git2::Commit<'_>> = parents.iter().collect();

            let commit_oid = repo
                .commit(Some("HEAD"), &sig, &sig, &message, &tree, &parent_refs)
                .map_err(|e| VcsError(format!("Failed to create commit: {e}")))?;

            info!("Created commit {}", commit_oid);
            Ok::<_, VcsError>(commit_oid.to_string())
        })
        .await
        .map_err(|e| VcsError(format!("Task join: {e}")))?
    }

    async fn push(&self, path: &Path, remote: &str, branch: &str) -> VcsResult<()> {
        let path = path.to_owned();
        let remote = remote.to_string();
        let branch = branch.to_string();
        let pat = self.pat.clone();

        tokio::task::spawn_blocking(move || {
            let repo = git2::Repository::open(&path)
                .map_err(|e| VcsError(format!("Failed to open repo: {e}")))?;

            let mut remote_handle = repo
                .find_remote(&remote)
                .map_err(|e| VcsError(format!("Failed to find remote: {e}")))?;

            let refspec = format!("refs/heads/{branch}:refs/heads/{branch}");
            let mut push_opts = git2::PushOptions::new();

            if let Some(token) = &pat {
                let token = token.clone();
                let mut callbacks = git2::RemoteCallbacks::new();
                callbacks.credentials(move |_url, _username, _allowed| {
                    git2::Cred::userpass_plaintext("oauth2", &token)
                });
                push_opts.remote_callbacks(callbacks);
            }

            remote_handle
                .push(&[&refspec], Some(&mut push_opts))
                .map_err(|e| VcsError(format!("Push failed: {e}")))?;

            Ok::<_, VcsError>(())
        })
        .await
        .map_err(|e| VcsError(format!("Task join: {e}")))?
    }

    async fn diff(&self, path: &Path) -> VcsResult<String> {
        let path = path.to_owned();

        tokio::task::spawn_blocking(move || {
            let repo = git2::Repository::open(&path)
                .map_err(|e| VcsError(format!("Failed to open repo: {e}")))?;

            let head_tree = repo
                .head()
                .ok()
                .and_then(|h| h.peel_to_commit().ok())
                .and_then(|c| c.tree().ok());

            let diff = repo
                .diff_tree_to_workdir_with_index(head_tree.as_ref(), None)
                .map_err(|e| VcsError(format!("Failed to get diff: {e}")))?;

            let mut diff_text = String::new();
            diff.print(git2::DiffFormat::Patch, |_delta, _hunk, line| {
                let content = std::str::from_utf8(line.content()).unwrap_or("");
                let prefix = match line.origin() {
                    '+' => "+",
                    '-' => "-",
                    ' ' => " ",
                    _ => "",
                };
                diff_text.push_str(prefix);
                diff_text.push_str(content);
                true
            })
            .map_err(|e| VcsError(format!("Failed to format diff: {e}")))?;

            Ok::<_, VcsError>(diff_text)
        })
        .await
        .map_err(|e| VcsError(format!("Task join: {e}")))?
    }

    async fn log(&self, path: &Path, limit: usize) -> VcsResult<Vec<CommitInfo>> {
        let path = path.to_owned();

        tokio::task::spawn_blocking(move || {
            let repo = git2::Repository::open(&path)
                .map_err(|e| VcsError(format!("Failed to open repo: {e}")))?;

            let mut walk = repo
                .revwalk()
                .map_err(|e| VcsError(format!("Failed to create revwalk: {e}")))?;
            walk.push_head()
                .map_err(|e| VcsError(format!("Failed to push HEAD: {e}")))?;

            let mut commits = Vec::new();
            for (i, oid) in walk.enumerate() {
                if i >= limit {
                    break;
                }
                let oid = oid.map_err(|e| VcsError(format!("Revwalk error: {e}")))?;
                let commit = repo
                    .find_commit(oid)
                    .map_err(|e| VcsError(format!("Failed to find commit: {e}")))?;

                let hash = &oid.to_string()[..7]; // Short hash
                let message = commit.message().unwrap_or("").trim().to_string();
                let author = commit.author().name().unwrap_or("Unknown").to_string();
                let timestamp = commit.time().seconds();

                commits.push(CommitInfo {
                    hash: hash.to_string(),
                    message,
                    author,
                    timestamp,
                });
            }

            Ok::<_, VcsError>(commits)
        })
        .await
        .map_err(|e| VcsError(format!("Task join: {e}")))?
    }

    async fn current_branch(&self, path: &Path) -> VcsResult<String> {
        let path = path.to_owned();

        tokio::task::spawn_blocking(move || {
            let repo = git2::Repository::open(&path)
                .map_err(|e| VcsError(format!("Failed to open repo: {e}")))?;

            let head = repo
                .head()
                .map_err(|e| VcsError(format!("Failed to get HEAD: {e}")))?;

            let branch = head.shorthand().unwrap_or("HEAD").to_string();

            Ok::<_, VcsError>(branch)
        })
        .await
        .map_err(|e| VcsError(format!("Task join: {e}")))?
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn init_repo(dir: &TempDir) -> git2::Repository {
        let repo = git2::Repository::init(dir.path()).expect("init repo");
        // Configure git identity for commits
        let mut config = repo.config().expect("config");
        config.set_str("user.name", "Test User").expect("set name");
        config
            .set_str("user.email", "test@test.com")
            .expect("set email");
        repo
    }

    fn make_commit(
        repo: &git2::Repository,
        message: &str,
        filename: &str,
        content: &str,
    ) -> git2::Oid {
        let path = repo.workdir().expect("workdir").join(filename);
        fs::write(&path, content).expect("write file");

        let mut index = repo.index().expect("index");
        index.add_path(std::path::Path::new(filename)).expect("add");
        index.write().expect("write index");

        let tree_oid = index.write_tree().expect("write tree");
        let tree = repo.find_tree(tree_oid).expect("find tree");
        let sig = git2::Signature::now("Test", "test@test.com").expect("sig");

        let parents = match repo.head() {
            Ok(h) => vec![h.peel_to_commit().expect("peel")],
            Err(_) => vec![],
        };
        let parent_refs: Vec<&git2::Commit<'_>> = parents.iter().collect();

        repo.commit(Some("HEAD"), &sig, &sig, message, &tree, &parent_refs)
            .expect("commit")
    }

    #[tokio::test]
    async fn test_commit_creates_commit() {
        let dir = TempDir::new().expect("tmpdir");
        let repo = init_repo(&dir);

        // Write a file
        let file_path = dir.path().join("hello.txt");
        fs::write(&file_path, "hello world").expect("write");

        let vcs = GitRepository::new();
        let hash = vcs
            .commit(dir.path(), "Initial commit", &[PathBuf::from("hello.txt")])
            .await
            .expect("commit");

        assert_eq!(hash.len(), 40); // Full OID

        // Verify commit exists
        let oid = git2::Oid::from_str(&hash).expect("parse oid");
        let commit = repo.find_commit(oid).expect("find commit");
        assert_eq!(commit.message().expect("message"), "Initial commit");
    }

    #[tokio::test]
    async fn test_current_branch_on_main() {
        let dir = TempDir::new().expect("tmpdir");
        let repo = init_repo(&dir);
        make_commit(&repo, "First", "file.txt", "content");

        let vcs = GitRepository::new();
        let branch = vcs.current_branch(dir.path()).await.expect("branch");
        // Branch name depends on git config (master or main)
        assert!(!branch.is_empty());
    }

    #[tokio::test]
    async fn test_checkout_creates_new_branch() {
        let dir = TempDir::new().expect("tmpdir");
        let repo = init_repo(&dir);
        make_commit(&repo, "First", "file.txt", "content");

        let vcs = GitRepository::new();
        vcs.checkout_branch(dir.path(), "feature/test", true)
            .await
            .expect("create branch");

        let branch = vcs.current_branch(dir.path()).await.expect("current");
        assert_eq!(branch, "feature/test");
    }

    #[tokio::test]
    async fn test_diff_shows_changes() {
        let dir = TempDir::new().expect("tmpdir");
        let repo = init_repo(&dir);
        make_commit(&repo, "Initial", "file.txt", "original content");

        // Modify the file (not yet staged/committed)
        fs::write(dir.path().join("file.txt"), "modified content").expect("write");

        let vcs = GitRepository::new();
        let diff = vcs.diff(dir.path()).await.expect("diff");
        assert!(diff.contains("modified") || diff.contains("-original") || !diff.is_empty());
    }

    #[tokio::test]
    async fn test_log_returns_commits() {
        let dir = TempDir::new().expect("tmpdir");
        let repo = init_repo(&dir);
        make_commit(&repo, "First commit", "file1.txt", "content1");
        make_commit(&repo, "Second commit", "file2.txt", "content2");

        let vcs = GitRepository::new();
        let log = vcs.log(dir.path(), 10).await.expect("log");
        assert_eq!(log.len(), 2);
        assert_eq!(log[0].message, "Second commit");
        assert_eq!(log[1].message, "First commit");
    }

    #[tokio::test]
    async fn test_log_respects_limit() {
        let dir = TempDir::new().expect("tmpdir");
        let repo = init_repo(&dir);
        for i in 0..5 {
            make_commit(
                &repo,
                &format!("Commit {}", i),
                &format!("file{}.txt", i),
                "content",
            );
        }

        let vcs = GitRepository::new();
        let log = vcs.log(dir.path(), 3).await.expect("log");
        assert_eq!(log.len(), 3);
    }
}
