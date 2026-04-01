#![deny(unsafe_code)]
#![warn(missing_docs)]

//! Version control system implementations for Cuttlefish.
//!
//! Provides [`VersionControl`] implementations for Git repositories,
//! supporting both local operations and remote authentication via PAT,
//! and a [`GitHubClient`] for GitHub REST API operations.

/// Git repository implementation using git2.
pub mod git;
/// GitHub REST API client.
pub mod github;

pub use cuttlefish_core::traits::vcs::{CommitInfo, VersionControl};
pub use git::GitRepository;
pub use github::GitHubClient;
