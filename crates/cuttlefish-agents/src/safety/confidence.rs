//! Confidence scoring system for agent actions.
//!
//! This module provides confidence scoring to help determine whether
//! an action should be auto-approved, require user confirmation, or be blocked.

use std::fmt;

/// A confidence score for an agent action (0.0 to 1.0).
///
/// Higher scores indicate greater confidence that the action is safe and correct.
/// The score is computed from multiple factors that contribute to or detract from
/// the overall confidence.
#[derive(Debug, Clone)]
pub struct ConfidenceScore {
    /// The overall confidence value (0.0 to 1.0).
    value: f32,
    /// Factors that contributed to this score.
    factors: Vec<ConfidenceFactor>,
    /// Human-readable reasoning for the score.
    reasoning: String,
}

impl ConfidenceScore {
    /// Create a new confidence score with the given value and factors.
    ///
    /// The value is clamped to the range [0.0, 1.0].
    pub fn new(value: f32, factors: Vec<ConfidenceFactor>, reasoning: impl Into<String>) -> Self {
        Self {
            value: value.clamp(0.0, 1.0),
            factors,
            reasoning: reasoning.into(),
        }
    }

    /// Create a high confidence score (0.95) for safe operations.
    pub fn high(reasoning: impl Into<String>) -> Self {
        Self::new(0.95, vec![ConfidenceFactor::Precedent(0.95)], reasoning)
    }

    /// Create a medium confidence score (0.7) for typical operations.
    pub fn medium(reasoning: impl Into<String>) -> Self {
        Self::new(0.7, vec![ConfidenceFactor::PatternMatch(0.7)], reasoning)
    }

    /// Create a low confidence score (0.3) for risky operations.
    pub fn low(reasoning: impl Into<String>) -> Self {
        Self::new(0.3, vec![ConfidenceFactor::RiskLevel(0.3)], reasoning)
    }

    /// Get the confidence value.
    pub fn value(&self) -> f32 {
        self.value
    }

    /// Get the factors that contributed to this score.
    pub fn factors(&self) -> &[ConfidenceFactor] {
        &self.factors
    }

    /// Get the reasoning for this score.
    pub fn reasoning(&self) -> &str {
        &self.reasoning
    }

    /// Check if this is a high confidence score (>= 0.9).
    pub fn is_high(&self) -> bool {
        self.value >= 0.9
    }

    /// Check if this is a medium confidence score (0.5 to 0.9).
    pub fn is_medium(&self) -> bool {
        self.value >= 0.5 && self.value < 0.9
    }

    /// Check if this is a low confidence score (< 0.5).
    pub fn is_low(&self) -> bool {
        self.value < 0.5
    }
}

impl fmt::Display for ConfidenceScore {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.0}% - {}", self.value * 100.0, self.reasoning)
    }
}

/// Factors that contribute to a confidence score.
///
/// Each factor represents a different aspect of confidence assessment.
/// The associated f32 value indicates the factor's contribution (0.0 to 1.0).
#[derive(Debug, Clone, PartialEq)]
pub enum ConfidenceFactor {
    /// Confidence from the model's own certainty assessment.
    /// Higher values indicate the model is more certain about its output.
    ModelCertainty(f32),

    /// Confidence from pattern matching against known safe operations.
    /// Higher values indicate the action matches well-known patterns.
    PatternMatch(f32),

    /// Confidence based on test coverage for the affected code.
    /// Higher values indicate better test coverage.
    TestCoverage(f32),

    /// Inverse risk level - higher values mean lower risk.
    /// This factor decreases confidence for risky operations.
    RiskLevel(f32),

    /// Confidence from precedent - similar actions succeeded before.
    /// Higher values indicate strong historical precedent.
    Precedent(f32),
}

impl ConfidenceFactor {
    /// Get the contribution value of this factor.
    pub fn value(&self) -> f32 {
        match self {
            Self::ModelCertainty(v)
            | Self::PatternMatch(v)
            | Self::TestCoverage(v)
            | Self::RiskLevel(v)
            | Self::Precedent(v) => *v,
        }
    }

    /// Get a human-readable name for this factor.
    pub fn name(&self) -> &'static str {
        match self {
            Self::ModelCertainty(_) => "Model Certainty",
            Self::PatternMatch(_) => "Pattern Match",
            Self::TestCoverage(_) => "Test Coverage",
            Self::RiskLevel(_) => "Risk Level",
            Self::Precedent(_) => "Precedent",
        }
    }
}

impl fmt::Display for ConfidenceFactor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {:.0}%", self.name(), self.value() * 100.0)
    }
}

/// Risk factors that can reduce confidence in an action.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RiskFactor {
    /// Action modifies an existing file (vs creating new).
    ModifiesExisting,
    /// Action affects multiple files at once.
    AffectsMultipleFiles,
    /// Action involves a system command (sudo, rm -rf, etc.).
    SystemCommand,
    /// Action involves irreversible git operations (force push, reset --hard).
    IrreversibleGit,
    /// Action affects high-impact paths (src/, config, etc.).
    HighImpactPath,
    /// Action deletes files or directories.
    Destructive,
    /// Action modifies configuration files.
    ConfigChange,
    /// Action involves network operations.
    NetworkOperation,
}

impl RiskFactor {
    /// Get the confidence penalty for this risk factor (0.0 to 1.0).
    ///
    /// Higher values indicate a larger penalty (more risk).
    pub fn penalty(&self) -> f32 {
        match self {
            Self::ModifiesExisting => 0.1,
            Self::AffectsMultipleFiles => 0.15,
            Self::SystemCommand => 0.25,
            Self::IrreversibleGit => 0.4,
            Self::HighImpactPath => 0.15,
            Self::Destructive => 0.3,
            Self::ConfigChange => 0.2,
            Self::NetworkOperation => 0.1,
        }
    }

    /// Get a human-readable description of this risk factor.
    pub fn description(&self) -> &'static str {
        match self {
            Self::ModifiesExisting => "Modifies existing file",
            Self::AffectsMultipleFiles => "Affects multiple files",
            Self::SystemCommand => "System command with elevated risk",
            Self::IrreversibleGit => "Irreversible git operation",
            Self::HighImpactPath => "High-impact file path",
            Self::Destructive => "Destructive operation",
            Self::ConfigChange => "Configuration change",
            Self::NetworkOperation => "Network operation",
        }
    }
}

/// Calculator for confidence scores based on action characteristics.
#[derive(Debug, Default)]
pub struct ConfidenceCalculator {
    /// Base confidence for new file creation.
    base_new_file: f32,
    /// Base confidence for file modification.
    base_modify_file: f32,
    /// Base confidence for file deletion.
    base_delete_file: f32,
    /// Base confidence for bash commands.
    base_bash_command: f32,
    /// Base confidence for git operations.
    base_git_operation: f32,
    /// Base confidence for config changes.
    base_config_change: f32,
}

impl ConfidenceCalculator {
    /// Create a new calculator with default base confidences.
    pub fn new() -> Self {
        Self {
            base_new_file: 0.9,
            base_modify_file: 0.7,
            base_delete_file: 0.5,
            base_bash_command: 0.6,
            base_git_operation: 0.7,
            base_config_change: 0.6,
        }
    }

    /// Calculate confidence for creating a new file.
    pub fn for_new_file(&self, path: &str) -> ConfidenceScore {
        let mut confidence = self.base_new_file;
        let mut factors = vec![ConfidenceFactor::PatternMatch(self.base_new_file)];
        let mut risks = Vec::new();

        // Check for high-impact paths
        if is_high_impact_path(path) {
            confidence -= RiskFactor::HighImpactPath.penalty();
            risks.push(RiskFactor::HighImpactPath);
        }

        // Check for config files
        if is_config_file(path) {
            confidence -= RiskFactor::ConfigChange.penalty();
            risks.push(RiskFactor::ConfigChange);
        }

        factors.push(ConfidenceFactor::RiskLevel(1.0 - total_penalty(&risks)));

        let reasoning = if risks.is_empty() {
            format!("Creating new file: {path}")
        } else {
            format!(
                "Creating new file: {path} (risks: {})",
                risks
                    .iter()
                    .map(|r| r.description())
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        };

        ConfidenceScore::new(confidence, factors, reasoning)
    }

    /// Calculate confidence for modifying an existing file.
    pub fn for_modify_file(&self, path: &str, lines_changed: usize) -> ConfidenceScore {
        let mut confidence = self.base_modify_file;
        let mut factors = vec![ConfidenceFactor::PatternMatch(self.base_modify_file)];
        let mut risks = vec![RiskFactor::ModifiesExisting];

        // Penalize large changes
        if lines_changed > 100 {
            confidence -= 0.2;
        } else if lines_changed > 50 {
            confidence -= 0.1;
        }

        // Check for high-impact paths
        if is_high_impact_path(path) {
            confidence -= RiskFactor::HighImpactPath.penalty();
            risks.push(RiskFactor::HighImpactPath);
        }

        // Check for config files
        if is_config_file(path) {
            confidence -= RiskFactor::ConfigChange.penalty();
            risks.push(RiskFactor::ConfigChange);
        }

        factors.push(ConfidenceFactor::RiskLevel(1.0 - total_penalty(&risks)));

        let reasoning = format!(
            "Modifying {path} ({lines_changed} lines, risks: {})",
            risks
                .iter()
                .map(|r| r.description())
                .collect::<Vec<_>>()
                .join(", ")
        );

        ConfidenceScore::new(confidence, factors, reasoning)
    }

    /// Calculate confidence for deleting a file.
    pub fn for_delete_file(&self, path: &str) -> ConfidenceScore {
        let mut confidence = self.base_delete_file;
        let mut factors = vec![ConfidenceFactor::PatternMatch(self.base_delete_file)];
        let mut risks = vec![RiskFactor::Destructive];

        // Check for high-impact paths
        if is_high_impact_path(path) {
            confidence -= RiskFactor::HighImpactPath.penalty();
            risks.push(RiskFactor::HighImpactPath);
        }

        // Check for config files
        if is_config_file(path) {
            confidence -= RiskFactor::ConfigChange.penalty();
            risks.push(RiskFactor::ConfigChange);
        }

        factors.push(ConfidenceFactor::RiskLevel(1.0 - total_penalty(&risks)));

        let reasoning = format!(
            "Deleting file: {path} (risks: {})",
            risks
                .iter()
                .map(|r| r.description())
                .collect::<Vec<_>>()
                .join(", ")
        );

        ConfidenceScore::new(confidence, factors, reasoning)
    }

    /// Calculate confidence for a bash command.
    pub fn for_bash_command(&self, command: &str) -> ConfidenceScore {
        let mut confidence = self.base_bash_command;
        let mut factors = vec![ConfidenceFactor::PatternMatch(self.base_bash_command)];
        let mut risks = Vec::new();

        // Check for dangerous commands
        if is_dangerous_command(command) {
            confidence -= RiskFactor::SystemCommand.penalty();
            risks.push(RiskFactor::SystemCommand);
        }

        // Check for destructive commands
        if is_destructive_command(command) {
            confidence -= RiskFactor::Destructive.penalty();
            risks.push(RiskFactor::Destructive);
        }

        // Check for network commands
        if is_network_command(command) {
            confidence -= RiskFactor::NetworkOperation.penalty();
            risks.push(RiskFactor::NetworkOperation);
        }

        // Safe commands get a boost
        if is_safe_command(command) {
            confidence = (confidence + 0.2).min(0.95);
        }

        factors.push(ConfidenceFactor::RiskLevel(1.0 - total_penalty(&risks)));

        let reasoning = if risks.is_empty() {
            format!("Executing command: {}", truncate_command(command))
        } else {
            format!(
                "Executing command: {} (risks: {})",
                truncate_command(command),
                risks
                    .iter()
                    .map(|r| r.description())
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        };

        ConfidenceScore::new(confidence, factors, reasoning)
    }

    /// Calculate confidence for a git operation.
    pub fn for_git_operation(&self, operation: &str) -> ConfidenceScore {
        let mut confidence = self.base_git_operation;
        let mut factors = vec![ConfidenceFactor::PatternMatch(self.base_git_operation)];
        let mut risks = Vec::new();

        // Check for irreversible operations
        if is_irreversible_git(operation) {
            confidence -= RiskFactor::IrreversibleGit.penalty();
            risks.push(RiskFactor::IrreversibleGit);
        }

        // Safe git operations get a boost
        if is_safe_git(operation) {
            confidence = (confidence + 0.2).min(0.95);
        }

        factors.push(ConfidenceFactor::RiskLevel(1.0 - total_penalty(&risks)));

        let reasoning = if risks.is_empty() {
            format!("Git operation: {operation}")
        } else {
            format!(
                "Git operation: {operation} (risks: {})",
                risks
                    .iter()
                    .map(|r| r.description())
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        };

        ConfidenceScore::new(confidence, factors, reasoning)
    }

    /// Calculate confidence for a config change.
    pub fn for_config_change(&self, config_path: &str) -> ConfidenceScore {
        let mut confidence = self.base_config_change;
        let mut factors = vec![ConfidenceFactor::PatternMatch(self.base_config_change)];
        let risks = vec![RiskFactor::ConfigChange];

        confidence -= RiskFactor::ConfigChange.penalty();
        factors.push(ConfidenceFactor::RiskLevel(1.0 - total_penalty(&risks)));

        let reasoning = format!(
            "Config change: {config_path} (risks: {})",
            risks
                .iter()
                .map(|r| r.description())
                .collect::<Vec<_>>()
                .join(", ")
        );

        ConfidenceScore::new(confidence, factors, reasoning)
    }

    /// Calculate confidence for a tool call based on tool name and input.
    pub fn calculate_for_tool_call(
        &self,
        call: &cuttlefish_core::traits::provider::ToolCall,
    ) -> ConfidenceScore {
        match call.name.as_str() {
            "write_file" => {
                let path = call.input["path"].as_str().unwrap_or("unknown");
                let content = call.input["content"].as_str().unwrap_or("");
                if is_config_file(path) {
                    self.for_config_change(path)
                } else {
                    let lines = content.lines().count();
                    self.for_modify_file(path, lines)
                }
            }
            "edit_file" => {
                let path = call.input["path"].as_str().unwrap_or("unknown");
                let edits = call.input["edits"].as_array().map(|a| a.len()).unwrap_or(1);
                self.for_modify_file(path, edits)
            }
            "execute_command" => {
                let cmd = call.input["command"].as_str().unwrap_or("");
                if cmd.starts_with("git ") {
                    self.for_git_operation(cmd)
                } else {
                    self.for_bash_command(cmd)
                }
            }
            "read_file" | "list_directory" | "search_files" => {
                ConfidenceScore::high("Read-only operation")
            }
            _ => ConfidenceScore::medium(format!("Unknown tool: {}", call.name)),
        }
    }

    /// Parse confidence from model output text.
    ///
    /// Looks for patterns like "confidence: 0.8" or "I am 80% confident".
    pub fn parse_from_model_output(&self, output: &str) -> Option<ConfidenceScore> {
        // Try to find explicit confidence statements
        let output_lower = output.to_lowercase();

        // Pattern: "confidence: X.X" or "confidence: XX%"
        if let Some(idx) = output_lower.find("confidence:") {
            let rest = &output[idx + 11..];
            if let Some(value) = parse_confidence_value(rest) {
                return Some(ConfidenceScore::new(
                    value,
                    vec![ConfidenceFactor::ModelCertainty(value)],
                    "Parsed from model output",
                ));
            }
        }

        // Pattern: "X% confident" or "XX% certain"
        for pattern in ["% confident", "% certain", "% sure"] {
            if let Some(idx) = output_lower.find(pattern) {
                // Look backwards for the number
                let before = &output[..idx];
                if let Some(value) = parse_percentage_before(before) {
                    return Some(ConfidenceScore::new(
                        value,
                        vec![ConfidenceFactor::ModelCertainty(value)],
                        "Parsed from model output",
                    ));
                }
            }
        }

        // Pattern: "highly confident" / "very confident" / "somewhat confident" / "not confident"
        if output_lower.contains("highly confident") || output_lower.contains("very confident") {
            return Some(ConfidenceScore::new(
                0.9,
                vec![ConfidenceFactor::ModelCertainty(0.9)],
                "Model expressed high confidence",
            ));
        }
        if output_lower.contains("fairly confident")
            || output_lower.contains("reasonably confident")
        {
            return Some(ConfidenceScore::new(
                0.7,
                vec![ConfidenceFactor::ModelCertainty(0.7)],
                "Model expressed moderate confidence",
            ));
        }
        if output_lower.contains("somewhat confident") {
            return Some(ConfidenceScore::new(
                0.5,
                vec![ConfidenceFactor::ModelCertainty(0.5)],
                "Model expressed some confidence",
            ));
        }
        if output_lower.contains("not confident") || output_lower.contains("uncertain") {
            return Some(ConfidenceScore::new(
                0.3,
                vec![ConfidenceFactor::ModelCertainty(0.3)],
                "Model expressed low confidence",
            ));
        }

        None
    }
}

// Helper functions

fn is_high_impact_path(path: &str) -> bool {
    let high_impact = [
        "src/main",
        "src/lib",
        "Cargo.toml",
        "package.json",
        "pyproject.toml",
        ".env",
        "config/",
        "settings",
        "docker",
        "Dockerfile",
        ".github/",
        "ci/",
        "deploy",
    ];
    high_impact.iter().any(|p| path.contains(p))
}

fn is_config_file(path: &str) -> bool {
    let config_extensions = [
        ".toml", ".yaml", ".yml", ".json", ".ini", ".cfg", ".conf", ".env",
    ];
    let config_names = [
        "config",
        "settings",
        "Cargo.toml",
        "package.json",
        "tsconfig",
        "pyproject",
    ];

    config_extensions.iter().any(|ext| path.ends_with(ext))
        || config_names.iter().any(|name| path.contains(name))
}

fn is_dangerous_command(command: &str) -> bool {
    let dangerous = [
        "sudo",
        "rm -rf",
        "rm -r",
        "chmod 777",
        "chown",
        "> /dev/",
        "dd if=",
        "mkfs",
        "fdisk",
        "kill -9",
        "pkill",
        "shutdown",
        "reboot",
        "systemctl",
        "service",
    ];
    dangerous.iter().any(|d| command.contains(d))
}

fn is_destructive_command(command: &str) -> bool {
    let destructive = ["rm ", "rmdir", "del ", "unlink", "truncate", "shred"];
    destructive.iter().any(|d| command.contains(d))
}

fn is_network_command(command: &str) -> bool {
    let network = [
        "curl", "wget", "ssh", "scp", "rsync", "ftp", "nc ", "netcat", "nmap",
    ];
    network.iter().any(|n| command.contains(n))
}

fn is_safe_command(command: &str) -> bool {
    let safe = [
        "ls",
        "cat",
        "echo",
        "pwd",
        "cd",
        "head",
        "tail",
        "grep",
        "find",
        "cargo test",
        "cargo build",
        "cargo clippy",
        "cargo fmt",
        "npm test",
        "npm run",
        "pytest",
        "go test",
        "make test",
        "git status",
        "git log",
        "git diff",
        "git branch",
    ];
    safe.iter()
        .any(|s| command.starts_with(s) || command.contains(s))
}

fn is_irreversible_git(operation: &str) -> bool {
    let irreversible = [
        "push --force",
        "push -f",
        "reset --hard",
        "clean -fd",
        "checkout --",
        "rebase",
        "filter-branch",
    ];
    irreversible.iter().any(|i| operation.contains(i))
}

fn is_safe_git(operation: &str) -> bool {
    let safe = [
        "status", "log", "diff", "branch", "fetch", "pull", "add", "commit", "push",
    ];
    // Only safe if it doesn't contain force flags
    safe.iter().any(|s| operation.contains(s))
        && !operation.contains("--force")
        && !operation.contains("-f")
}

fn total_penalty(risks: &[RiskFactor]) -> f32 {
    risks.iter().map(|r| r.penalty()).sum::<f32>().min(0.8)
}

fn truncate_command(command: &str) -> String {
    if command.len() > 50 {
        format!("{}...", &command[..47])
    } else {
        command.to_string()
    }
}

fn parse_confidence_value(text: &str) -> Option<f32> {
    let text = text.trim();

    // Try to parse as decimal (0.8)
    if let Some(end) = text.find(|c: char| !c.is_ascii_digit() && c != '.') {
        let num_str = &text[..end];
        if let Ok(value) = num_str.parse::<f32>()
            && (0.0..=1.0).contains(&value)
        {
            return Some(value);
        }
    }

    // Try to parse as percentage (80%)
    if let Some(pct_idx) = text.find('%') {
        let num_str = text[..pct_idx].trim();
        if let Ok(value) = num_str.parse::<f32>() {
            return Some(value / 100.0);
        }
    }

    None
}

fn parse_percentage_before(text: &str) -> Option<f32> {
    // Find the last number in the text
    let mut num_end = text.len();
    let mut num_start = num_end;

    for (i, c) in text.char_indices().rev() {
        if c.is_ascii_digit() {
            if num_start == num_end {
                num_end = i + 1;
            }
            num_start = i;
        } else if num_start != num_end {
            break;
        }
    }

    if num_start < num_end {
        let num_str = &text[num_start..num_end];
        if let Ok(value) = num_str.parse::<f32>() {
            return Some(value / 100.0);
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_confidence_score_clamping() {
        let score = ConfidenceScore::new(1.5, vec![], "test");
        assert_eq!(score.value(), 1.0);

        let score = ConfidenceScore::new(-0.5, vec![], "test");
        assert_eq!(score.value(), 0.0);
    }

    #[test]
    fn test_confidence_score_levels() {
        let high = ConfidenceScore::high("test");
        assert!(high.is_high());
        assert!(!high.is_medium());
        assert!(!high.is_low());

        let medium = ConfidenceScore::medium("test");
        assert!(!medium.is_high());
        assert!(medium.is_medium());
        assert!(!medium.is_low());

        let low = ConfidenceScore::low("test");
        assert!(!low.is_high());
        assert!(!low.is_medium());
        assert!(low.is_low());
    }

    #[test]
    fn test_confidence_factor_values() {
        let factor = ConfidenceFactor::ModelCertainty(0.8);
        assert_eq!(factor.value(), 0.8);
        assert_eq!(factor.name(), "Model Certainty");
    }

    #[test]
    fn test_risk_factor_penalties() {
        assert!(RiskFactor::IrreversibleGit.penalty() > RiskFactor::ModifiesExisting.penalty());
        assert!(RiskFactor::Destructive.penalty() > RiskFactor::NetworkOperation.penalty());
    }

    #[test]
    fn test_calculator_new_file() {
        let calc = ConfidenceCalculator::new();

        // New file in safe location
        let score = calc.for_new_file("tests/test_helper.rs");
        assert!(score.value() > 0.8);

        // New file in high-impact location
        let score = calc.for_new_file("src/main.rs");
        assert!(score.value() < 0.9);
    }

    #[test]
    fn test_calculator_modify_file() {
        let calc = ConfidenceCalculator::new();

        // Small modification
        let score = calc.for_modify_file("src/lib.rs", 10);
        assert!(score.value() > 0.4);

        // Large modification
        let score = calc.for_modify_file("src/lib.rs", 150);
        assert!(score.value() < 0.6);
    }

    #[test]
    fn test_calculator_delete_file() {
        let calc = ConfidenceCalculator::new();

        let score = calc.for_delete_file("temp/test.txt");
        assert!(score.is_low() || score.is_medium());

        let score = calc.for_delete_file("src/main.rs");
        assert!(score.is_low());
    }

    #[test]
    fn test_calculator_bash_command() {
        let calc = ConfidenceCalculator::new();

        // Safe command
        let score = calc.for_bash_command("cargo test");
        assert!(score.value() > 0.7);

        // Dangerous command
        let score = calc.for_bash_command("sudo rm -rf /");
        assert!(score.is_low());
    }

    #[test]
    fn test_calculator_git_operation() {
        let calc = ConfidenceCalculator::new();

        // Safe git operation
        let score = calc.for_git_operation("git status");
        assert!(score.value() > 0.8);

        // Irreversible git operation
        let score = calc.for_git_operation("git push --force");
        assert!(score.is_low());
    }

    #[test]
    fn test_parse_confidence_decimal() {
        let calc = ConfidenceCalculator::new();

        let score = calc
            .parse_from_model_output("I have confidence: 0.85 in this change")
            .expect("should parse");
        assert!((score.value() - 0.85).abs() < 0.01);
    }

    #[test]
    fn test_parse_confidence_percentage() {
        let calc = ConfidenceCalculator::new();

        let score = calc
            .parse_from_model_output("I am 85% confident this is correct")
            .expect("should parse");
        assert!((score.value() - 0.85).abs() < 0.01);
    }

    #[test]
    fn test_parse_confidence_qualitative() {
        let calc = ConfidenceCalculator::new();

        let score = calc
            .parse_from_model_output("I am highly confident this will work")
            .expect("should parse");
        assert!(score.value() >= 0.9);

        let score = calc
            .parse_from_model_output("I am not confident about this approach")
            .expect("should parse");
        assert!(score.value() <= 0.4);
    }

    #[test]
    fn test_parse_confidence_none() {
        let calc = ConfidenceCalculator::new();

        let result = calc.parse_from_model_output("This is a regular message without confidence");
        assert!(result.is_none());
    }

    #[test]
    fn test_confidence_score_display() {
        let score = ConfidenceScore::new(0.85, vec![], "Test reasoning");
        let display = format!("{score}");
        assert!(display.contains("85%"));
        assert!(display.contains("Test reasoning"));
    }

    #[test]
    fn test_high_impact_paths() {
        assert!(is_high_impact_path("src/main.rs"));
        assert!(is_high_impact_path("Cargo.toml"));
        assert!(is_high_impact_path(".github/workflows/ci.yml"));
        assert!(!is_high_impact_path("tests/helper.rs"));
    }

    #[test]
    fn test_config_files() {
        assert!(is_config_file("config.toml"));
        assert!(is_config_file("settings.yaml"));
        assert!(is_config_file("Cargo.toml"));
        assert!(!is_config_file("main.rs"));
    }

    #[test]
    fn test_calculate_for_tool_call_write_file() {
        use cuttlefish_core::traits::provider::ToolCall;

        let calc = ConfidenceCalculator::new();
        let call = ToolCall {
            id: "1".to_string(),
            name: "write_file".to_string(),
            input: serde_json::json!({"path": "test.rs", "content": "fn main() {}"}),
        };
        let score = calc.calculate_for_tool_call(&call);
        assert!(score.value() > 0.0);
    }

    #[test]
    fn test_calculate_for_tool_call_execute_command() {
        use cuttlefish_core::traits::provider::ToolCall;

        let calc = ConfidenceCalculator::new();

        let safe_call = ToolCall {
            id: "2".to_string(),
            name: "execute_command".to_string(),
            input: serde_json::json!({"command": "cargo test"}),
        };
        let score = calc.calculate_for_tool_call(&safe_call);
        assert!(score.value() > 0.7);

        let git_call = ToolCall {
            id: "3".to_string(),
            name: "execute_command".to_string(),
            input: serde_json::json!({"command": "git status"}),
        };
        let score = calc.calculate_for_tool_call(&git_call);
        assert!(score.value() > 0.8);
    }

    #[test]
    fn test_calculate_for_tool_call_read_only() {
        use cuttlefish_core::traits::provider::ToolCall;

        let calc = ConfidenceCalculator::new();
        let call = ToolCall {
            id: "4".to_string(),
            name: "read_file".to_string(),
            input: serde_json::json!({"path": "/etc/passwd"}),
        };
        let score = calc.calculate_for_tool_call(&call);
        assert!(score.is_high());
    }
}
