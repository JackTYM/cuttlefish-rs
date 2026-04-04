//! Rich embed builders for Discord messages.
//!
//! This module provides embed builders for:
//! - Agent status displays with color-coded states
//! - Task completion summaries
//! - Error messages with context
//! - Question prompts for user input
//! - Progress indicators for long-running operations

use serenity::builder::CreateEmbed;
use serenity::model::Colour;

// =============================================================================
// Design System Colors
// =============================================================================

/// Discord embed color palette for consistent theming.
pub mod colors {
    use super::Colour;

    /// Primary brand color (Discord Blurple).
    pub const PRIMARY: Colour = Colour::from_rgb(88, 101, 242);

    /// Success/complete state (green).
    pub const SUCCESS: Colour = Colour::from_rgb(87, 242, 135);

    /// Working/in-progress state (amber).
    pub const WORKING: Colour = Colour::from_rgb(251, 191, 36);

    /// Error/failed state (red).
    pub const ERROR: Colour = Colour::from_rgb(239, 83, 80);

    /// Waiting/idle state (gray).
    pub const WAITING: Colour = Colour::from_rgb(128, 128, 128);

    /// Info/neutral state (blue).
    pub const INFO: Colour = Colour::from_rgb(66, 165, 245);
}

// =============================================================================
// Agent Status Embeds
// =============================================================================

/// Agent status states with corresponding emoji and color.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentStatus {
    /// Agent is actively working on a task.
    Working,
    /// Agent is waiting for input or resources.
    Waiting,
    /// Agent encountered an error.
    Error,
    /// Agent completed its task.
    Complete,
}

impl AgentStatus {
    /// Get the emoji for this status.
    pub fn emoji(&self) -> &'static str {
        match self {
            AgentStatus::Working => "🟢",
            AgentStatus::Waiting => "🟡",
            AgentStatus::Error => "🔴",
            AgentStatus::Complete => "✅",
        }
    }

    /// Get the color for this status.
    pub fn color(&self) -> Colour {
        match self {
            AgentStatus::Working => colors::WORKING,
            AgentStatus::Waiting => colors::WAITING,
            AgentStatus::Error => colors::ERROR,
            AgentStatus::Complete => colors::SUCCESS,
        }
    }

    /// Get the display text for this status.
    pub fn text(&self) -> &'static str {
        match self {
            AgentStatus::Working => "Working",
            AgentStatus::Waiting => "Waiting",
            AgentStatus::Error => "Error",
            AgentStatus::Complete => "Complete",
        }
    }
}

/// Agent types with corresponding emoji identifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentType {
    /// Orchestrator agent.
    Orchestrator,
    /// Planner agent.
    Planner,
    /// Coder agent.
    Coder,
    /// Critic agent.
    Critic,
    /// Explorer agent.
    Explorer,
    /// Librarian agent.
    Librarian,
    /// DevOps agent.
    DevOps,
    /// Generic/unknown agent.
    Generic,
}

impl AgentType {
    /// Get the emoji for this agent type.
    pub fn emoji(&self) -> &'static str {
        match self {
            AgentType::Orchestrator => "🎭",
            AgentType::Planner => "📋",
            AgentType::Coder => "💻",
            AgentType::Critic => "🔍",
            AgentType::Explorer => "🔎",
            AgentType::Librarian => "📚",
            AgentType::DevOps => "🚀",
            AgentType::Generic => "🤖",
        }
    }

    /// Get the display name for this agent type.
    pub fn name(&self) -> &'static str {
        match self {
            AgentType::Orchestrator => "Orchestrator",
            AgentType::Planner => "Planner",
            AgentType::Coder => "Coder",
            AgentType::Critic => "Critic",
            AgentType::Explorer => "Explorer",
            AgentType::Librarian => "Librarian",
            AgentType::DevOps => "DevOps",
            AgentType::Generic => "Agent",
        }
    }
}

/// Builder for agent status embeds.
#[derive(Debug, Clone)]
pub struct AgentStatusEmbed {
    /// Agent type.
    agent_type: AgentType,
    /// Current status.
    status: AgentStatus,
    /// Current action description.
    action: String,
    /// Duration of current operation.
    duration: Option<String>,
    /// Sub-agent status (if delegating).
    sub_agents: Vec<SubAgentStatus>,
}

/// Status of a sub-agent.
#[derive(Debug, Clone)]
pub struct SubAgentStatus {
    /// Sub-agent type.
    pub agent_type: AgentType,
    /// Sub-agent status.
    pub status: AgentStatus,
    /// Current action.
    pub action: String,
}

impl AgentStatusEmbed {
    /// Create a new agent status embed.
    pub fn new(agent_type: AgentType, status: AgentStatus) -> Self {
        Self {
            agent_type,
            status,
            action: String::new(),
            duration: None,
            sub_agents: Vec::new(),
        }
    }

    /// Set the current action description.
    pub fn action(mut self, action: impl Into<String>) -> Self {
        self.action = action.into();
        self
    }

    /// Set the duration of the current operation.
    pub fn duration(mut self, duration: impl Into<String>) -> Self {
        self.duration = Some(duration.into());
        self
    }

    /// Add a sub-agent status.
    pub fn sub_agent(mut self, sub: SubAgentStatus) -> Self {
        self.sub_agents.push(sub);
        self
    }

    /// Build the embed.
    pub fn build(&self) -> CreateEmbed {
        let title = format!(
            "{} {} — {}",
            self.agent_type.emoji(),
            self.agent_type.name(),
            self.status.emoji()
        );

        let mut embed = CreateEmbed::new()
            .title(title)
            .colour(self.status.color())
            .field(
                "Status",
                format!("{} {}", self.status.emoji(), self.status.text()),
                true,
            );

        if let Some(ref duration) = self.duration {
            embed = embed.field("Duration", duration, true);
        }

        if !self.action.is_empty() {
            embed = embed.field("Current Action", &self.action, false);
        }

        if !self.sub_agents.is_empty() {
            let sub_status: String = self
                .sub_agents
                .iter()
                .map(|s| {
                    format!(
                        "{} {} — {} {}",
                        s.agent_type.emoji(),
                        s.agent_type.name(),
                        s.status.emoji(),
                        s.status.text()
                    )
                })
                .collect::<Vec<_>>()
                .join("\n");

            embed = embed.field("Sub-Agents", sub_status, false);
        }

        embed
    }
}

// =============================================================================
// Task Completion Embeds
// =============================================================================

/// Builder for task completion embeds.
#[derive(Debug, Clone)]
pub struct TaskCompletionEmbed {
    /// Task summary.
    summary: String,
    /// Files changed.
    files_changed: Vec<String>,
    /// Test results.
    test_results: Option<TestResults>,
    /// Duration of the task.
    duration: Option<String>,
}

/// Test results summary.
#[derive(Debug, Clone)]
pub struct TestResults {
    /// Number of tests passed.
    pub passed: u32,
    /// Number of tests failed.
    pub failed: u32,
    /// Number of tests skipped.
    pub skipped: u32,
}

impl TestResults {
    /// Create new test results.
    pub fn new(passed: u32, failed: u32, skipped: u32) -> Self {
        Self {
            passed,
            failed,
            skipped,
        }
    }

    /// Check if all tests passed.
    pub fn all_passed(&self) -> bool {
        self.failed == 0
    }

    /// Get total test count.
    pub fn total(&self) -> u32 {
        self.passed + self.failed + self.skipped
    }

    /// Format as a string.
    pub fn format(&self) -> String {
        format!(
            "✅ {} passed  ❌ {} failed  ⏭️ {} skipped",
            self.passed, self.failed, self.skipped
        )
    }
}

impl TaskCompletionEmbed {
    /// Create a new task completion embed.
    pub fn new(summary: impl Into<String>) -> Self {
        Self {
            summary: summary.into(),
            files_changed: Vec::new(),
            test_results: None,
            duration: None,
        }
    }

    /// Add a changed file.
    pub fn file_changed(mut self, file: impl Into<String>) -> Self {
        self.files_changed.push(file.into());
        self
    }

    /// Set multiple changed files.
    pub fn files_changed(mut self, files: Vec<String>) -> Self {
        self.files_changed = files;
        self
    }

    /// Set test results.
    pub fn test_results(mut self, results: TestResults) -> Self {
        self.test_results = Some(results);
        self
    }

    /// Set the task duration.
    pub fn duration(mut self, duration: impl Into<String>) -> Self {
        self.duration = Some(duration.into());
        self
    }

    /// Build the embed.
    pub fn build(&self) -> CreateEmbed {
        let color = self
            .test_results
            .as_ref()
            .map(|t| {
                if t.all_passed() {
                    colors::SUCCESS
                } else {
                    colors::ERROR
                }
            })
            .unwrap_or(colors::SUCCESS);

        let mut embed = CreateEmbed::new()
            .title("✅ Task Complete")
            .colour(color)
            .description(&self.summary);

        if !self.files_changed.is_empty() {
            let files_list = if self.files_changed.len() <= 10 {
                self.files_changed.join("\n")
            } else {
                let count = self.files_changed.len();
                let shown: String = self
                    .files_changed
                    .iter()
                    .take(10)
                    .cloned()
                    .collect::<Vec<_>>()
                    .join("\n");
                format!("{shown}\n_...and {} more files_", count - 10)
            };
            embed = embed.field("Files Changed", files_list, false);
        }

        if let Some(ref results) = self.test_results {
            embed = embed.field("Tests", results.format(), false);
        }

        if let Some(ref duration) = self.duration {
            embed = embed.field("Duration", duration, true);
        }

        embed
    }
}

// =============================================================================
// Error Embeds
// =============================================================================

/// Builder for error embeds.
#[derive(Debug, Clone)]
pub struct ErrorEmbed {
    /// Error message.
    message: String,
    /// Stack trace (truncated).
    stack_trace: Option<String>,
    /// Suggested action.
    suggestion: Option<String>,
    /// Error code.
    error_code: Option<String>,
}

impl ErrorEmbed {
    /// Create a new error embed.
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            stack_trace: None,
            suggestion: None,
            error_code: None,
        }
    }

    /// Set the stack trace (will be truncated if too long).
    pub fn stack_trace(mut self, trace: impl Into<String>) -> Self {
        let trace = trace.into();
        // Discord field value limit is 1024 chars
        const MAX_TRACE_LEN: usize = 1000;
        self.stack_trace = if trace.len() > MAX_TRACE_LEN {
            Some(format!("{}...\n_(truncated)_", &trace[..MAX_TRACE_LEN]))
        } else {
            Some(trace)
        };
        self
    }

    /// Set a suggested action.
    pub fn suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestion = Some(suggestion.into());
        self
    }

    /// Set an error code.
    pub fn error_code(mut self, code: impl Into<String>) -> Self {
        self.error_code = Some(code.into());
        self
    }

    /// Build the embed.
    pub fn build(&self) -> CreateEmbed {
        let mut embed = CreateEmbed::new()
            .title("❌ Error")
            .colour(colors::ERROR)
            .description(&self.message);

        if let Some(ref code) = self.error_code {
            embed = embed.field("Error Code", format!("`{code}`"), true);
        }

        if let Some(ref trace) = self.stack_trace {
            embed = embed.field("Stack Trace", format!("```\n{trace}\n```"), false);
        }

        if let Some(ref suggestion) = self.suggestion {
            embed = embed.field("💡 Suggested Action", suggestion, false);
        }

        embed
    }
}

// =============================================================================
// Question Embeds
// =============================================================================

/// Builder for question embeds.
#[derive(Debug, Clone)]
pub struct QuestionEmbed {
    /// The question being asked.
    question: String,
    /// Available options (if any).
    options: Vec<String>,
    /// Timeout for response.
    timeout: Option<String>,
    /// Context for the question.
    context: Option<String>,
}

impl QuestionEmbed {
    /// Create a new question embed.
    pub fn new(question: impl Into<String>) -> Self {
        Self {
            question: question.into(),
            options: Vec::new(),
            timeout: None,
            context: None,
        }
    }

    /// Add an option.
    pub fn option(mut self, option: impl Into<String>) -> Self {
        self.options.push(option.into());
        self
    }

    /// Set multiple options.
    pub fn options(mut self, options: Vec<String>) -> Self {
        self.options = options;
        self
    }

    /// Set the timeout for response.
    pub fn timeout(mut self, timeout: impl Into<String>) -> Self {
        self.timeout = Some(timeout.into());
        self
    }

    /// Set context for the question.
    pub fn context(mut self, context: impl Into<String>) -> Self {
        self.context = Some(context.into());
        self
    }

    /// Build the embed.
    pub fn build(&self) -> CreateEmbed {
        let mut embed = CreateEmbed::new()
            .title("❓ Agent Question")
            .colour(colors::WAITING)
            .description(&self.question);

        if !self.options.is_empty() {
            let options_text = self
                .options
                .iter()
                .enumerate()
                .map(|(i, opt)| format!("**{}.** {}", i + 1, opt))
                .collect::<Vec<_>>()
                .join("\n");
            embed = embed.field("Options", options_text, false);
        }

        if let Some(ref timeout) = self.timeout {
            embed = embed.field("⏱️ Timeout", timeout, true);
        }

        if let Some(ref context) = self.context {
            embed = embed.field("Context", context, false);
        }

        embed.footer(serenity::builder::CreateEmbedFooter::new(
            "Reply with your answer or use /approve /reject",
        ))
    }
}

// =============================================================================
// Progress Embeds
// =============================================================================

/// Status of a single step in a multi-step process.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepStatus {
    /// Step is complete.
    Complete,
    /// Step is in progress.
    InProgress,
    /// Step is pending.
    Pending,
    /// Step failed.
    Failed,
}

impl StepStatus {
    /// Get the emoji for this step status.
    pub fn emoji(&self) -> &'static str {
        match self {
            StepStatus::Complete => "✅",
            StepStatus::InProgress => "⏳",
            StepStatus::Pending => "⬜",
            StepStatus::Failed => "❌",
        }
    }
}

/// A single step in a multi-step progress display.
#[derive(Debug, Clone)]
pub struct ProgressStep {
    /// Step name.
    pub name: String,
    /// Step status.
    pub status: StepStatus,
    /// Optional detail text.
    pub detail: Option<String>,
}

impl ProgressStep {
    /// Create a new progress step.
    pub fn new(name: impl Into<String>, status: StepStatus) -> Self {
        Self {
            name: name.into(),
            status,
            detail: None,
        }
    }

    /// Add detail text.
    pub fn detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }
}

/// Builder for multi-step progress embeds.
#[derive(Debug, Clone)]
pub struct ProgressEmbed {
    /// Task description.
    task: String,
    /// Steps in the process.
    steps: Vec<ProgressStep>,
    /// Current step index (0-based).
    current_step: usize,
    /// Estimated time remaining.
    eta: Option<String>,
}

impl ProgressEmbed {
    /// Create a new progress embed.
    pub fn new(task: impl Into<String>) -> Self {
        Self {
            task: task.into(),
            steps: Vec::new(),
            current_step: 0,
            eta: None,
        }
    }

    /// Add a step.
    pub fn step(mut self, step: ProgressStep) -> Self {
        self.steps.push(step);
        self
    }

    /// Set multiple steps.
    pub fn steps(mut self, steps: Vec<ProgressStep>) -> Self {
        self.steps = steps;
        self
    }

    /// Set the current step index.
    pub fn current_step(mut self, index: usize) -> Self {
        self.current_step = index;
        self
    }

    /// Set estimated time remaining.
    pub fn eta(mut self, eta: impl Into<String>) -> Self {
        self.eta = Some(eta.into());
        self
    }

    /// Build the embed.
    pub fn build(&self) -> CreateEmbed {
        let completed = self
            .steps
            .iter()
            .filter(|s| s.status == StepStatus::Complete)
            .count();
        let total = self.steps.len();

        let progress_bar = Self::build_progress_bar(completed, total);

        let steps_display: String = self
            .steps
            .iter()
            .enumerate()
            .map(|(i, s)| {
                let detail = s
                    .detail
                    .as_ref()
                    .map(|d| format!("\n   └─ {d}"))
                    .unwrap_or_default();
                format!("{} **{}.** {}{}", s.status.emoji(), i + 1, s.name, detail)
            })
            .collect::<Vec<_>>()
            .join("\n");

        let mut embed = CreateEmbed::new()
            .title(format!("⏳ Progress: {}", self.task))
            .colour(colors::WORKING)
            .description(format!("{progress_bar}\n\n{steps_display}"))
            .field("Progress", format!("{}/{} steps", completed, total), true);

        if let Some(ref eta) = self.eta {
            embed = embed.field("ETA", eta, true);
        }

        embed
    }

    /// Build a text-based progress bar.
    fn build_progress_bar(completed: usize, total: usize) -> String {
        const BAR_LENGTH: usize = 10;

        if total == 0 {
            return "░░░░░░░░░░ 0%".to_string();
        }

        let percentage = (completed * 100) / total;
        let filled = (completed * BAR_LENGTH) / total;

        let bar: String = (0..BAR_LENGTH)
            .map(|i| if i < filled { '▓' } else { '░' })
            .collect();

        format!("{} {}%", bar, percentage)
    }

    /// Create a final success embed.
    pub fn build_success(&self) -> CreateEmbed {
        let steps_display: String = self
            .steps
            .iter()
            .enumerate()
            .map(|(i, s)| {
                let emoji = if s.status == StepStatus::Complete {
                    "✅"
                } else {
                    s.status.emoji()
                };
                format!("{} **{}.** {}", emoji, i + 1, s.name)
            })
            .collect::<Vec<_>>()
            .join("\n");

        CreateEmbed::new()
            .title(format!("✅ Complete: {}", self.task))
            .colour(colors::SUCCESS)
            .description(steps_display)
    }

    /// Create a failure embed.
    pub fn build_failure(&self, error: &str) -> CreateEmbed {
        let steps_display: String = self
            .steps
            .iter()
            .enumerate()
            .map(|(i, s)| format!("{} **{}.** {}", s.status.emoji(), i + 1, s.name))
            .collect::<Vec<_>>()
            .join("\n");

        CreateEmbed::new()
            .title(format!("❌ Failed: {}", self.task))
            .colour(colors::ERROR)
            .description(format!(
                "{}\n\n**Error:**\n```\n{}\n```",
                steps_display, error
            ))
    }
}

// =============================================================================
// Build Progress Embeds
// =============================================================================

/// Builder for build/test progress embeds with live updates.
#[derive(Debug, Clone)]
pub struct BuildProgressEmbed {
    /// Build target or test name.
    target: String,
    /// Current output lines.
    output: Vec<String>,
    /// Whether the build is complete.
    complete: bool,
    /// Whether the build succeeded.
    success: bool,
    /// Error message if failed.
    error: Option<String>,
}

impl BuildProgressEmbed {
    /// Create a new build progress embed.
    pub fn new(target: impl Into<String>) -> Self {
        Self {
            target: target.into(),
            output: Vec::new(),
            complete: false,
            success: false,
            error: None,
        }
    }

    /// Add output line.
    pub fn output(mut self, line: impl Into<String>) -> Self {
        self.output.push(line.into());
        self
    }

    /// Set multiple output lines.
    pub fn outputs(mut self, lines: Vec<String>) -> Self {
        self.output = lines;
        self
    }

    /// Mark as complete with success status.
    pub fn complete(mut self, success: bool) -> Self {
        self.complete = true;
        self.success = success;
        self
    }

    /// Set error message.
    pub fn error(mut self, error: impl Into<String>) -> Self {
        self.error = Some(error.into());
        self
    }

    /// Build the embed.
    pub fn build(&self) -> CreateEmbed {
        let (title, color) = if !self.complete {
            (format!("⏳ Building: {}", self.target), colors::WORKING)
        } else if self.success {
            (
                format!("✅ Build Succeeded: {}", self.target),
                colors::SUCCESS,
            )
        } else {
            (format!("❌ Build Failed: {}", self.target), colors::ERROR)
        };

        // Truncate output to fit Discord limits
        const MAX_OUTPUT_LINES: usize = 15;
        const MAX_OUTPUT_CHARS: usize = 900;

        let output_text = if self.output.is_empty() {
            "_No output yet..._".to_string()
        } else {
            let mut text = String::new();
            for line in self.output.iter().rev().take(MAX_OUTPUT_LINES).rev() {
                if text.len() + line.len() + 1 > MAX_OUTPUT_CHARS {
                    text.push_str("...\n_(output truncated)_");
                    break;
                }
                text.push_str(line);
                text.push('\n');
            }
            text
        };

        let mut embed = CreateEmbed::new()
            .title(title)
            .colour(color)
            .description(format!("```\n{}\n```", output_text.trim_end()));

        if let Some(ref error) = self.error {
            embed = embed.field("Error", format!("```\n{}\n```", error), false);
        }

        embed
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_status_emoji() {
        assert_eq!(AgentStatus::Working.emoji(), "🟢");
        assert_eq!(AgentStatus::Waiting.emoji(), "🟡");
        assert_eq!(AgentStatus::Error.emoji(), "🔴");
        assert_eq!(AgentStatus::Complete.emoji(), "✅");
    }

    #[test]
    fn test_agent_type_emoji() {
        assert_eq!(AgentType::Orchestrator.emoji(), "🎭");
        assert_eq!(AgentType::Coder.emoji(), "💻");
        assert_eq!(AgentType::Critic.emoji(), "🔍");
    }

    #[test]
    fn test_agent_status_embed_builds() {
        let embed = AgentStatusEmbed::new(AgentType::Coder, AgentStatus::Working)
            .action("Implementing feature X")
            .duration("2m 30s")
            .build();

        let json = serde_json::to_value(&embed).expect("should serialize");
        assert!(json.get("title").is_some());
        assert!(json.get("fields").is_some());
    }

    #[test]
    fn test_agent_status_embed_with_sub_agents() {
        let embed = AgentStatusEmbed::new(AgentType::Orchestrator, AgentStatus::Working)
            .action("Coordinating agents")
            .sub_agent(SubAgentStatus {
                agent_type: AgentType::Coder,
                status: AgentStatus::Working,
                action: "Writing code".to_string(),
            })
            .build();

        let json = serde_json::to_value(&embed).expect("should serialize");
        let fields = json
            .get("fields")
            .and_then(|f| f.as_array())
            .expect("has fields");
        assert!(
            fields.len() >= 3,
            "Should have status, duration, action, and sub-agents"
        );
    }

    #[test]
    fn test_task_completion_embed() {
        let embed = TaskCompletionEmbed::new("Implemented user authentication")
            .file_changed("src/auth.rs")
            .file_changed("src/models/user.rs")
            .test_results(TestResults::new(10, 0, 2))
            .duration("5m 30s")
            .build();

        let json = serde_json::to_value(&embed).expect("should serialize");
        assert_eq!(
            json.get("color").and_then(|c| c.as_u64()),
            Some(u64::from(colors::SUCCESS.0))
        );
    }

    #[test]
    fn test_task_completion_with_failed_tests() {
        let embed = TaskCompletionEmbed::new("Feature implementation")
            .test_results(TestResults::new(8, 2, 0))
            .build();

        let json = serde_json::to_value(&embed).expect("should serialize");
        assert_eq!(
            json.get("color").and_then(|c| c.as_u64()),
            Some(u64::from(colors::ERROR.0))
        );
    }

    #[test]
    fn test_error_embed() {
        let embed = ErrorEmbed::new("Failed to compile project")
            .error_code("E0425")
            .suggestion("Check your imports and ensure all dependencies are available")
            .build();

        let json = serde_json::to_value(&embed).expect("should serialize");
        let fields = json
            .get("fields")
            .and_then(|f| f.as_array())
            .expect("has fields");
        assert!(fields.len() >= 2, "Should have error code and suggestion");
    }

    #[test]
    fn test_error_embed_truncates_stack_trace() {
        let long_trace = "x".repeat(2000);
        let embed = ErrorEmbed::new("Error").stack_trace(&long_trace).build();

        let json = serde_json::to_value(&embed).expect("should serialize");
        let fields = json
            .get("fields")
            .and_then(|f| f.as_array())
            .expect("has fields");

        let trace_field = fields
            .iter()
            .find(|f| f.get("name").and_then(|n| n.as_str()) == Some("Stack Trace"));
        assert!(trace_field.is_some());

        let trace_value = trace_field
            .expect("found")
            .get("value")
            .and_then(|v| v.as_str())
            .expect("has value");
        assert!(trace_value.contains("truncated"));
    }

    #[test]
    fn test_question_embed() {
        let embed = QuestionEmbed::new("Which database should we use?")
            .option("PostgreSQL")
            .option("SQLite")
            .option("MongoDB")
            .timeout("5 minutes")
            .build();

        let json = serde_json::to_value(&embed).expect("should serialize");
        let fields = json
            .get("fields")
            .and_then(|f| f.as_array())
            .expect("has fields");
        assert!(fields.len() >= 2, "Should have options and timeout");
    }

    #[test]
    fn test_progress_embed() {
        let embed = ProgressEmbed::new("Deploying application")
            .step(ProgressStep::new("Build", StepStatus::Complete))
            .step(ProgressStep::new("Test", StepStatus::Complete))
            .step(ProgressStep::new("Deploy", StepStatus::InProgress))
            .step(ProgressStep::new("Verify", StepStatus::Pending))
            .eta("2 minutes")
            .build();

        let json = serde_json::to_value(&embed).expect("should serialize");
        assert!(json.get("description").is_some());
    }

    #[test]
    fn test_progress_bar() {
        assert_eq!(ProgressEmbed::build_progress_bar(0, 4), "░░░░░░░░░░ 0%");
        assert_eq!(ProgressEmbed::build_progress_bar(2, 4), "▓▓▓▓▓░░░░░ 50%");
        assert_eq!(ProgressEmbed::build_progress_bar(4, 4), "▓▓▓▓▓▓▓▓▓▓ 100%");
    }

    #[test]
    fn test_progress_embed_success() {
        let embed = ProgressEmbed::new("Task")
            .step(ProgressStep::new("Step 1", StepStatus::Complete))
            .step(ProgressStep::new("Step 2", StepStatus::Complete))
            .build_success();

        let json = serde_json::to_value(&embed).expect("should serialize");
        assert!(json
            .get("title")
            .and_then(|t| t.as_str())
            .unwrap_or("")
            .starts_with("✅"));
    }

    #[test]
    fn test_progress_embed_failure() {
        let embed = ProgressEmbed::new("Task")
            .step(ProgressStep::new("Step 1", StepStatus::Complete))
            .step(ProgressStep::new("Step 2", StepStatus::Failed))
            .build_failure("Connection refused");

        let json = serde_json::to_value(&embed).expect("should serialize");
        assert!(json
            .get("title")
            .and_then(|t| t.as_str())
            .unwrap_or("")
            .starts_with("❌"));
    }

    #[test]
    fn test_build_progress_embed() {
        let embed = BuildProgressEmbed::new("my-project")
            .output("Compiling src/main.rs")
            .output("Compiling src/lib.rs")
            .build();

        let json = serde_json::to_value(&embed).expect("should serialize");
        assert!(json
            .get("title")
            .and_then(|t| t.as_str())
            .unwrap_or("")
            .contains("Building"));
    }

    #[test]
    fn test_build_progress_embed_complete() {
        let embed = BuildProgressEmbed::new("my-project").complete(true).build();

        let json = serde_json::to_value(&embed).expect("should serialize");
        assert!(json
            .get("title")
            .and_then(|t| t.as_str())
            .unwrap_or("")
            .contains("Succeeded"));
    }

    #[test]
    fn test_build_progress_embed_failed() {
        let embed = BuildProgressEmbed::new("my-project")
            .complete(false)
            .error("Compilation error: cannot find type `User`")
            .build();

        let json = serde_json::to_value(&embed).expect("should serialize");
        assert!(json
            .get("title")
            .and_then(|t| t.as_str())
            .unwrap_or("")
            .contains("Failed"));
    }

    #[test]
    fn test_test_results() {
        let results = TestResults::new(10, 2, 3);
        assert_eq!(results.total(), 15);
        assert!(!results.all_passed());
        assert!(results.format().contains("10 passed"));
    }

    #[test]
    fn test_step_status_emoji() {
        assert_eq!(StepStatus::Complete.emoji(), "✅");
        assert_eq!(StepStatus::InProgress.emoji(), "⏳");
        assert_eq!(StepStatus::Pending.emoji(), "⬜");
        assert_eq!(StepStatus::Failed.emoji(), "❌");
    }

    #[test]
    fn test_colors_defined() {
        // Ensure all colors are defined and accessible
        let _ = colors::PRIMARY;
        let _ = colors::SUCCESS;
        let _ = colors::WORKING;
        let _ = colors::ERROR;
        let _ = colors::WAITING;
        let _ = colors::INFO;
    }
}
