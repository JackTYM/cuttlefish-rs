//! Safety system for agent actions.
//!
//! This module provides:
//! - Confidence scoring for agent actions
//! - Action gates with configurable thresholds
//! - Diff generation for file change previews
//! - Checkpoint system for state capture and rollback

mod checkpoint;
mod confidence;
mod diff;
mod gates;

pub use checkpoint::{
    Checkpoint, CheckpointComponents, CheckpointConfig, CheckpointError, CheckpointId,
    CheckpointManager, CheckpointResult, CheckpointStore, CheckpointTrigger,
    DEFAULT_CHECKPOINT_TIMEOUT_SECS, InMemoryCheckpointStore, MAX_CHECKPOINTS_PER_PROJECT,
    RollbackResult,
};
pub use confidence::{ConfidenceCalculator, ConfidenceFactor, ConfidenceScore, RiskFactor};
pub use diff::{
    ChangeType, DiffHunk, DiffLine, DiffStats, FileDiff, MAX_DIFF_FILE_SIZE, detect_language,
};
pub use gates::{
    ActionGate, ActionPreview, ActionType, GateConfig, GateDecision, QuickDecision,
    ThresholdOverride,
};
