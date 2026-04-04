//! Action gates with configurable thresholds for agent safety.
//!
//! This module provides gating logic to determine whether agent actions
//! should be auto-approved, require user confirmation, or be blocked.

use super::confidence::ConfidenceScore;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// Types of actions that agents can perform.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ActionType {
    /// Writing content to a file (create or overwrite).
    FileWrite,
    /// Deleting a file or directory.
    FileDelete,
    /// Executing a shell/bash command.
    BashCommand,
    /// Git operations (commit, push, branch, etc.).
    GitOperation,
    /// Modifying configuration files.
    ConfigChange,
}

impl ActionType {
    /// Get a human-readable name for this action type.
    pub fn name(&self) -> &'static str {
        match self {
            Self::FileWrite => "File Write",
            Self::FileDelete => "File Delete",
            Self::BashCommand => "Bash Command",
            Self::GitOperation => "Git Operation",
            Self::ConfigChange => "Config Change",
        }
    }

    /// Get the default auto-approve threshold for this action type.
    pub fn default_auto_approve_threshold(&self) -> f32 {
        match self {
            Self::FileWrite => 0.9,
            Self::FileDelete => 0.95,
            Self::BashCommand => 0.85,
            Self::GitOperation => 0.9,
            Self::ConfigChange => 0.95,
        }
    }

    /// Get the default prompt threshold for this action type.
    pub fn default_prompt_threshold(&self) -> f32 {
        match self {
            Self::FileWrite => 0.5,
            Self::FileDelete => 0.6,
            Self::BashCommand => 0.4,
            Self::GitOperation => 0.5,
            Self::ConfigChange => 0.6,
        }
    }
}

impl fmt::Display for ActionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Configuration for action gates.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateConfig {
    /// Default threshold above which actions are auto-approved.
    pub auto_approve_threshold: f32,
    /// Default threshold above which actions prompt the user (below = block).
    pub prompt_threshold: f32,
    /// Per-action-type threshold overrides.
    overrides: HashMap<ActionType, ThresholdOverride>,
    /// Actions that always require user approval regardless of confidence.
    always_prompt: Vec<ActionType>,
    /// Actions that are always blocked regardless of confidence.
    always_block: Vec<ActionType>,
}

/// Threshold overrides for a specific action type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThresholdOverride {
    /// Override for auto-approve threshold.
    pub auto_approve: Option<f32>,
    /// Override for prompt threshold.
    pub prompt: Option<f32>,
}

impl Default for GateConfig {
    fn default() -> Self {
        Self {
            auto_approve_threshold: 0.9,
            prompt_threshold: 0.5,
            overrides: HashMap::new(),
            always_prompt: Vec::new(),
            always_block: Vec::new(),
        }
    }
}

impl GateConfig {
    /// Create a new gate config with default thresholds.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a strict config that prompts for most actions.
    pub fn strict() -> Self {
        Self {
            auto_approve_threshold: 0.98,
            prompt_threshold: 0.7,
            overrides: HashMap::new(),
            always_prompt: vec![ActionType::FileDelete, ActionType::GitOperation],
            always_block: Vec::new(),
        }
    }

    /// Create a permissive config that auto-approves most actions.
    pub fn permissive() -> Self {
        Self {
            auto_approve_threshold: 0.7,
            prompt_threshold: 0.3,
            overrides: HashMap::new(),
            always_prompt: Vec::new(),
            always_block: Vec::new(),
        }
    }

    /// Set the auto-approve threshold.
    pub fn with_auto_approve_threshold(mut self, threshold: f32) -> Self {
        self.auto_approve_threshold = threshold.clamp(0.0, 1.0);
        self
    }

    /// Set the prompt threshold.
    pub fn with_prompt_threshold(mut self, threshold: f32) -> Self {
        self.prompt_threshold = threshold.clamp(0.0, 1.0);
        self
    }

    /// Add a threshold override for a specific action type.
    pub fn with_override(
        mut self,
        action_type: ActionType,
        auto_approve: Option<f32>,
        prompt: Option<f32>,
    ) -> Self {
        self.overrides.insert(
            action_type,
            ThresholdOverride {
                auto_approve: auto_approve.map(|v| v.clamp(0.0, 1.0)),
                prompt: prompt.map(|v| v.clamp(0.0, 1.0)),
            },
        );
        self
    }

    /// Mark an action type as always requiring user approval.
    pub fn always_prompt_for(mut self, action_type: ActionType) -> Self {
        if !self.always_prompt.contains(&action_type) {
            self.always_prompt.push(action_type);
        }
        self
    }

    /// Mark an action type as always blocked.
    pub fn always_block_for(mut self, action_type: ActionType) -> Self {
        if !self.always_block.contains(&action_type) {
            self.always_block.push(action_type);
        }
        self
    }

    /// Get the effective auto-approve threshold for an action type.
    pub fn get_auto_approve_threshold(&self, action_type: ActionType) -> f32 {
        self.overrides
            .get(&action_type)
            .and_then(|o| o.auto_approve)
            .unwrap_or(self.auto_approve_threshold)
    }

    /// Get the effective prompt threshold for an action type.
    pub fn get_prompt_threshold(&self, action_type: ActionType) -> f32 {
        self.overrides
            .get(&action_type)
            .and_then(|o| o.prompt)
            .unwrap_or(self.prompt_threshold)
    }

    /// Check if an action type always requires prompting.
    pub fn requires_prompt(&self, action_type: ActionType) -> bool {
        self.always_prompt.contains(&action_type)
    }

    /// Check if an action type is always blocked.
    pub fn is_blocked(&self, action_type: ActionType) -> bool {
        self.always_block.contains(&action_type)
    }
}

/// A preview of an action for user review.
#[derive(Debug, Clone)]
pub struct ActionPreview {
    /// Description of the action.
    pub description: String,
    /// The type of action.
    pub action_type: ActionType,
    /// Affected file path (if applicable).
    pub path: Option<String>,
    /// Command to execute (if applicable).
    pub command: Option<String>,
    /// Additional context for the user.
    pub context: Option<String>,
}

impl ActionPreview {
    /// Create a new action preview.
    pub fn new(description: impl Into<String>, action_type: ActionType) -> Self {
        Self {
            description: description.into(),
            action_type,
            path: None,
            command: None,
            context: None,
        }
    }

    /// Set the affected path.
    pub fn with_path(mut self, path: impl Into<String>) -> Self {
        self.path = Some(path.into());
        self
    }

    /// Set the command.
    pub fn with_command(mut self, command: impl Into<String>) -> Self {
        self.command = Some(command.into());
        self
    }

    /// Set additional context.
    pub fn with_context(mut self, context: impl Into<String>) -> Self {
        self.context = Some(context.into());
        self
    }
}

/// The decision made by an action gate.
#[derive(Debug, Clone)]
pub enum GateDecision {
    /// Action is automatically approved and can proceed.
    AutoApprove,
    /// Action requires user confirmation before proceeding.
    PromptUser {
        /// Preview of the action for user review.
        preview: ActionPreview,
        /// The confidence score that led to this decision.
        confidence: ConfidenceScore,
    },
    /// Action is blocked and cannot proceed.
    Block {
        /// Reason for blocking the action.
        reason: String,
    },
}

impl GateDecision {
    /// Check if this decision allows the action to proceed.
    pub fn allows_proceed(&self) -> bool {
        matches!(self, Self::AutoApprove)
    }

    /// Check if this decision requires user input.
    pub fn requires_user_input(&self) -> bool {
        matches!(self, Self::PromptUser { .. })
    }

    /// Check if this decision blocks the action.
    pub fn is_blocked(&self) -> bool {
        matches!(self, Self::Block { .. })
    }
}

impl fmt::Display for GateDecision {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AutoApprove => write!(f, "Auto-approved"),
            Self::PromptUser {
                preview,
                confidence,
            } => {
                write!(
                    f,
                    "Requires approval: {} (confidence: {:.0}%)",
                    preview.description,
                    confidence.value() * 100.0
                )
            }
            Self::Block { reason } => write!(f, "Blocked: {reason}"),
        }
    }
}

/// Gate for evaluating agent actions.
#[derive(Debug, Clone)]
pub struct ActionGate {
    config: GateConfig,
}

impl ActionGate {
    /// Create a new action gate with the given configuration.
    pub fn new(config: GateConfig) -> Self {
        Self { config }
    }

    /// Create a gate with default configuration.
    pub fn with_defaults() -> Self {
        Self::new(GateConfig::default())
    }

    /// Get the gate configuration.
    pub fn config(&self) -> &GateConfig {
        &self.config
    }

    /// Evaluate an action and return a gate decision.
    pub fn evaluate(
        &self,
        action_type: ActionType,
        confidence: &ConfidenceScore,
        preview: ActionPreview,
    ) -> GateDecision {
        // Check if action type is always blocked
        if self.config.is_blocked(action_type) {
            return GateDecision::Block {
                reason: format!(
                    "{} actions are blocked by configuration",
                    action_type.name()
                ),
            };
        }

        // Check if action type always requires prompting
        if self.config.requires_prompt(action_type) {
            return GateDecision::PromptUser {
                preview,
                confidence: confidence.clone(),
            };
        }

        let auto_threshold = self.config.get_auto_approve_threshold(action_type);
        let prompt_threshold = self.config.get_prompt_threshold(action_type);
        let value = confidence.value();

        if value >= auto_threshold {
            GateDecision::AutoApprove
        } else if value >= prompt_threshold {
            GateDecision::PromptUser {
                preview,
                confidence: confidence.clone(),
            }
        } else {
            GateDecision::Block {
                reason: format!(
                    "Confidence too low ({:.0}% < {:.0}% threshold)",
                    value * 100.0,
                    prompt_threshold * 100.0
                ),
            }
        }
    }

    /// Quick evaluation without creating a preview (for internal checks).
    pub fn quick_evaluate(&self, action_type: ActionType, confidence: f32) -> QuickDecision {
        if self.config.is_blocked(action_type) {
            return QuickDecision::Block;
        }

        if self.config.requires_prompt(action_type) {
            return QuickDecision::Prompt;
        }

        let auto_threshold = self.config.get_auto_approve_threshold(action_type);
        let prompt_threshold = self.config.get_prompt_threshold(action_type);

        if confidence >= auto_threshold {
            QuickDecision::AutoApprove
        } else if confidence >= prompt_threshold {
            QuickDecision::Prompt
        } else {
            QuickDecision::Block
        }
    }
}

impl Default for ActionGate {
    fn default() -> Self {
        Self::with_defaults()
    }
}

/// Simplified decision for quick checks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuickDecision {
    /// Action can proceed automatically.
    AutoApprove,
    /// Action needs user confirmation.
    Prompt,
    /// Action is blocked.
    Block,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_confidence(value: f32) -> ConfidenceScore {
        ConfidenceScore::new(value, vec![], "test")
    }

    fn make_preview(action_type: ActionType) -> ActionPreview {
        ActionPreview::new("Test action", action_type)
    }

    #[test]
    fn test_action_type_names() {
        assert_eq!(ActionType::FileWrite.name(), "File Write");
        assert_eq!(ActionType::FileDelete.name(), "File Delete");
        assert_eq!(ActionType::BashCommand.name(), "Bash Command");
        assert_eq!(ActionType::GitOperation.name(), "Git Operation");
        assert_eq!(ActionType::ConfigChange.name(), "Config Change");
    }

    #[test]
    fn test_gate_config_defaults() {
        let config = GateConfig::default();
        assert_eq!(config.auto_approve_threshold, 0.9);
        assert_eq!(config.prompt_threshold, 0.5);
    }

    #[test]
    fn test_gate_config_strict() {
        let config = GateConfig::strict();
        assert!(config.auto_approve_threshold > 0.95);
        assert!(config.requires_prompt(ActionType::FileDelete));
        assert!(config.requires_prompt(ActionType::GitOperation));
    }

    #[test]
    fn test_gate_config_permissive() {
        let config = GateConfig::permissive();
        assert!(config.auto_approve_threshold < 0.8);
        assert!(config.prompt_threshold < 0.4);
    }

    #[test]
    fn test_gate_config_overrides() {
        let config = GateConfig::new().with_override(ActionType::FileDelete, Some(0.99), Some(0.8));

        assert_eq!(
            config.get_auto_approve_threshold(ActionType::FileDelete),
            0.99
        );
        assert_eq!(config.get_prompt_threshold(ActionType::FileDelete), 0.8);
        // Other types use defaults
        assert_eq!(
            config.get_auto_approve_threshold(ActionType::FileWrite),
            0.9
        );
    }

    #[test]
    fn test_gate_config_always_prompt() {
        let config = GateConfig::new().always_prompt_for(ActionType::GitOperation);
        assert!(config.requires_prompt(ActionType::GitOperation));
        assert!(!config.requires_prompt(ActionType::FileWrite));
    }

    #[test]
    fn test_gate_config_always_block() {
        let config = GateConfig::new().always_block_for(ActionType::FileDelete);
        assert!(config.is_blocked(ActionType::FileDelete));
        assert!(!config.is_blocked(ActionType::FileWrite));
    }

    #[test]
    fn test_action_preview_builder() {
        let preview = ActionPreview::new("Write file", ActionType::FileWrite)
            .with_path("src/main.rs")
            .with_context("Adding new function");

        assert_eq!(preview.description, "Write file");
        assert_eq!(preview.action_type, ActionType::FileWrite);
        assert_eq!(preview.path, Some("src/main.rs".to_string()));
        assert_eq!(preview.context, Some("Adding new function".to_string()));
    }

    #[test]
    fn test_gate_decision_auto_approve() {
        let gate = ActionGate::with_defaults();
        let confidence = make_confidence(0.95);
        let preview = make_preview(ActionType::FileWrite);

        let decision = gate.evaluate(ActionType::FileWrite, &confidence, preview);
        assert!(matches!(decision, GateDecision::AutoApprove));
        assert!(decision.allows_proceed());
        assert!(!decision.requires_user_input());
    }

    #[test]
    fn test_gate_decision_prompt_user() {
        let gate = ActionGate::with_defaults();
        let confidence = make_confidence(0.7);
        let preview = make_preview(ActionType::FileWrite);

        let decision = gate.evaluate(ActionType::FileWrite, &confidence, preview);
        assert!(matches!(decision, GateDecision::PromptUser { .. }));
        assert!(!decision.allows_proceed());
        assert!(decision.requires_user_input());
    }

    #[test]
    fn test_gate_decision_block() {
        let gate = ActionGate::with_defaults();
        let confidence = make_confidence(0.3);
        let preview = make_preview(ActionType::FileWrite);

        let decision = gate.evaluate(ActionType::FileWrite, &confidence, preview);
        assert!(matches!(decision, GateDecision::Block { .. }));
        assert!(!decision.allows_proceed());
        assert!(!decision.requires_user_input());
        assert!(decision.is_blocked());
    }

    #[test]
    fn test_gate_always_blocked_action() {
        let config = GateConfig::new().always_block_for(ActionType::FileDelete);
        let gate = ActionGate::new(config);
        let confidence = make_confidence(1.0); // Even perfect confidence
        let preview = make_preview(ActionType::FileDelete);

        let decision = gate.evaluate(ActionType::FileDelete, &confidence, preview);
        assert!(matches!(decision, GateDecision::Block { .. }));
    }

    #[test]
    fn test_gate_always_prompt_action() {
        let config = GateConfig::new().always_prompt_for(ActionType::GitOperation);
        let gate = ActionGate::new(config);
        let confidence = make_confidence(1.0); // Even perfect confidence
        let preview = make_preview(ActionType::GitOperation);

        let decision = gate.evaluate(ActionType::GitOperation, &confidence, preview);
        assert!(matches!(decision, GateDecision::PromptUser { .. }));
    }

    #[test]
    fn test_quick_evaluate() {
        let gate = ActionGate::with_defaults();

        assert_eq!(
            gate.quick_evaluate(ActionType::FileWrite, 0.95),
            QuickDecision::AutoApprove
        );
        assert_eq!(
            gate.quick_evaluate(ActionType::FileWrite, 0.7),
            QuickDecision::Prompt
        );
        assert_eq!(
            gate.quick_evaluate(ActionType::FileWrite, 0.3),
            QuickDecision::Block
        );
    }

    #[test]
    fn test_gate_decision_display() {
        let decision = GateDecision::AutoApprove;
        assert_eq!(format!("{decision}"), "Auto-approved");

        let decision = GateDecision::Block {
            reason: "Too risky".to_string(),
        };
        assert!(format!("{decision}").contains("Blocked"));
        assert!(format!("{decision}").contains("Too risky"));
    }

    #[test]
    fn test_threshold_clamping() {
        let config = GateConfig::new()
            .with_auto_approve_threshold(1.5)
            .with_prompt_threshold(-0.5);

        assert_eq!(config.auto_approve_threshold, 1.0);
        assert_eq!(config.prompt_threshold, 0.0);
    }
}
