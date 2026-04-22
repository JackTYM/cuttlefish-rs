//! TUI application state management.

use std::collections::VecDeque;

/// Slash commands that can be executed from the input.
#[derive(Debug, Clone)]
pub enum SlashCommand {
    /// Show help.
    Help,
    /// List projects.
    ListProjects,
    /// Create a new project.
    NewProject { name: Option<String> },
    /// Switch to a project by ID.
    SwitchProject { id: String },
    /// Clear chat history.
    ClearChat,
    /// Quit the application.
    Quit,
}

/// A message displayed in the chat view.
#[derive(Debug, Clone)]
pub struct ChatMessage {
    /// Message sender (e.g., "user", "coder", "orchestrator").
    pub sender: String,
    /// Message content.
    pub content: String,
}

/// The current view tab.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AppView {
    /// Project dashboard - the startup view.
    #[default]
    Dashboard,
    /// Chat with agents (requires active project).
    Chat,
    /// File diff viewer.
    Diff,
    /// Build log viewer.
    Log,
    /// Help screen showing available commands.
    Help,
    /// History mode - browse past messages and restore.
    History,
    /// Create project wizard.
    CreateProject,
}

/// Restore options shown when a message is selected in history mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RestoreOption {
    /// Restore only the conversation (truncate to this message).
    Conversation,
    /// Restore only the code/sandbox state.
    Code,
    /// Restore both conversation and code.
    Both,
    /// Cancel and return to history view.
    Cancel,
}

impl RestoreOption {
    /// Get the display label.
    pub fn label(&self) -> &'static str {
        match self {
            RestoreOption::Conversation => "Restore Conversation",
            RestoreOption::Code => "Restore Code",
            RestoreOption::Both => "Restore Code & Conversation",
            RestoreOption::Cancel => "Cancel",
        }
    }

    /// Get all options in order.
    pub fn all() -> &'static [RestoreOption] {
        &[
            RestoreOption::Conversation,
            RestoreOption::Code,
            RestoreOption::Both,
            RestoreOption::Cancel,
        ]
    }
}

/// Execution mode for a project.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ExecutionMode {
    /// Everything on server.
    #[default]
    Cloud,
    /// Build on server, run binary locally.
    BuildRemoteRunLocal,
    /// Sync code, build & run locally.
    Local,
}

impl ExecutionMode {
    /// Get a short label.
    pub fn label(&self) -> &'static str {
        match self {
            ExecutionMode::Cloud => "Cloud",
            ExecutionMode::BuildRemoteRunLocal => "Build Remote",
            ExecutionMode::Local => "Local",
        }
    }

    /// Get a description.
    pub fn description(&self) -> &'static str {
        match self {
            ExecutionMode::Cloud => "Build & run on server. Web apps get tunnel URL, TUI apps stream interactively.",
            ExecutionMode::BuildRemoteRunLocal => "Build on server, download binary to run locally. No toolchain needed.",
            ExecutionMode::Local => "Sync code to your machine, build & run locally. Requires toolchain.",
        }
    }

    /// Parse from string.
    pub fn from_str(s: &str) -> Self {
        match s {
            "cloud" => ExecutionMode::Cloud,
            "build_remote_run_local" => ExecutionMode::BuildRemoteRunLocal,
            "local" => ExecutionMode::Local,
            _ => ExecutionMode::Cloud,
        }
    }

    /// Convert to string for API.
    pub fn as_str(&self) -> &'static str {
        match self {
            ExecutionMode::Cloud => "cloud",
            ExecutionMode::BuildRemoteRunLocal => "build_remote_run_local",
            ExecutionMode::Local => "local",
        }
    }

    /// Get all modes for selection.
    pub fn all() -> &'static [ExecutionMode] {
        &[
            ExecutionMode::Cloud,
            ExecutionMode::BuildRemoteRunLocal,
            ExecutionMode::Local,
        ]
    }
}

/// Running status of a project.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RunningStatus {
    #[default]
    Idle,
    Building,
    Running,
    Error,
    Stopped,
}

impl RunningStatus {
    /// Get emoji indicator.
    pub fn emoji(&self) -> &'static str {
        match self {
            RunningStatus::Idle => "⚪",
            RunningStatus::Building => "🟡",
            RunningStatus::Running => "🟢",
            RunningStatus::Error => "🔴",
            RunningStatus::Stopped => "⚫",
        }
    }

    /// Parse from string.
    pub fn from_str(s: &str) -> Self {
        match s {
            "idle" => RunningStatus::Idle,
            "building" => RunningStatus::Building,
            "running" => RunningStatus::Running,
            "error" => RunningStatus::Error,
            "stopped" => RunningStatus::Stopped,
            _ => RunningStatus::Idle,
        }
    }
}

/// Represents a project in the dashboard.
#[derive(Debug, Clone)]
pub struct ProjectInfo {
    /// Project ID.
    pub id: String,
    /// Project name.
    pub name: String,
    /// Template name (if any).
    pub template_name: Option<String>,
    /// Execution mode.
    pub execution_mode: ExecutionMode,
    /// Running status.
    pub running_status: RunningStatus,
    /// Local path (for local mode).
    pub local_path: Option<String>,
    /// Tunnel URL (for cloud mode).
    pub tunnel_url: Option<String>,
    /// Last activity relative time (e.g., "2m", "3d").
    pub last_activity: String,
    /// Whether this is the currently active project.
    pub active: bool,
}

/// Step in the create project wizard.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CreateProjectStep {
    /// Step 1: Enter project name.
    #[default]
    Name,
    /// Step 2: Choose template.
    Template,
    /// Step 3: Choose execution mode.
    Mode,
}

/// Template info for selection.
#[derive(Debug, Clone)]
pub struct TemplateInfo {
    /// Template name/ID.
    pub name: String,
    /// Template description.
    pub description: String,
    /// Category (builtin, user, community).
    pub category: String,
    /// Language/framework.
    pub language: String,
}

/// State for the create project wizard.
#[derive(Debug, Clone, Default)]
pub struct CreateProjectState {
    /// Current step.
    pub step: CreateProjectStep,
    /// Project name input.
    pub name: String,
    /// Validated name (available and valid).
    pub name_valid: bool,
    /// Name validation message.
    pub name_message: String,
    /// Available templates.
    pub templates: Vec<TemplateInfo>,
    /// Selected template index.
    pub template_selected: usize,
    /// Selected execution mode.
    pub mode_selected: usize,
    /// Local path input (for local mode).
    pub local_path: String,
}

/// Application state for the TUI.
pub struct App {
    /// Whether the app should exit.
    pub should_exit: bool,
    /// Current active view tab.
    pub view: AppView,
    /// Current input buffer.
    pub input: String,
    /// Chat messages (most recent last).
    pub messages: VecDeque<ChatMessage>,
    /// Build log lines.
    pub log_lines: Vec<String>,
    /// Current diff content.
    pub diff_content: String,
    /// Active project ID.
    pub project_id: Option<String>,
    /// Connection status.
    pub connected: bool,
    /// Whether we're currently receiving a streaming response.
    pub streaming: bool,
    /// Counter for mouth animation (increments with each token chunk).
    pub token_count: u32,
    /// Scroll offset for chat view.
    pub chat_scroll: u16,
    /// Scroll offset for log view.
    pub log_scroll: u16,
    /// Scroll offset for diff view.
    pub diff_scroll: u16,
    /// List of available projects.
    pub projects: Vec<ProjectInfo>,
    /// Selected index in projects list.
    pub projects_selected: usize,
    /// Input history for command recall.
    pub input_history: VecDeque<String>,
    /// Current position in input history.
    pub history_pos: Option<usize>,

    // Create project wizard state
    /// State for the create project wizard.
    pub create_project: CreateProjectState,

    // History mode state
    /// Last Esc key press timestamp (for double-tap detection).
    pub last_esc_time: Option<std::time::Instant>,
    /// Selected message index in history view (0 = most recent).
    pub history_selected: usize,
    /// Whether restore options popup is showing.
    pub show_restore_options: bool,
    /// Currently selected restore option.
    pub restore_option_selected: usize,
    /// Message sequence numbers with snapshots (for indicator display).
    pub snapshot_message_indices: Vec<usize>,
}

impl App {
    /// Create a new application state.
    pub fn new() -> Self {
        Self {
            should_exit: false,
            view: AppView::Dashboard,
            input: String::new(),
            messages: VecDeque::new(),
            log_lines: Vec::new(),
            diff_content: String::new(),
            project_id: None,
            connected: false,
            streaming: false,
            token_count: 0,
            chat_scroll: 0,
            log_scroll: 0,
            diff_scroll: 0,
            projects: Vec::new(),
            projects_selected: 0,
            input_history: VecDeque::new(),
            history_pos: None,
            // Create project wizard
            create_project: CreateProjectState::default(),
            // History mode
            last_esc_time: None,
            history_selected: 0,
            show_restore_options: false,
            restore_option_selected: 0,
            snapshot_message_indices: Vec::new(),
        }
    }

    /// Cycle to the next view tab (only when in a project).
    pub fn next_view(&mut self) {
        // Only cycle views when we have an active project
        if self.project_id.is_some() {
            self.view = match self.view {
                AppView::Chat => AppView::Diff,
                AppView::Diff => AppView::Log,
                AppView::Log => AppView::Help,
                AppView::Help => AppView::Dashboard,
                AppView::Dashboard => AppView::Chat,
                AppView::History => AppView::Chat,
                AppView::CreateProject => AppView::Dashboard,
            };
        } else {
            // Without a project, only Dashboard and Help make sense
            self.view = match self.view {
                AppView::Dashboard => AppView::Help,
                AppView::Help => AppView::Dashboard,
                _ => AppView::Dashboard,
            };
        }
    }

    /// Handle Esc key press. Returns true if we should exit the app.
    pub fn handle_esc(&mut self) -> bool {
        let now = std::time::Instant::now();

        // If in create project wizard, go back or cancel
        if self.view == AppView::CreateProject {
            match self.create_project.step {
                CreateProjectStep::Name => {
                    // Cancel wizard, go back to dashboard
                    self.view = AppView::Dashboard;
                }
                CreateProjectStep::Template => {
                    self.create_project.step = CreateProjectStep::Name;
                }
                CreateProjectStep::Mode => {
                    self.create_project.step = CreateProjectStep::Template;
                }
            }
            return false;
        }

        // If in history mode
        if self.view == AppView::History {
            if self.show_restore_options {
                // Close restore popup
                self.show_restore_options = false;
                return false;
            }
            // Exit history mode
            self.view = AppView::Chat;
            return false;
        }

        // Check for double-tap Esc to enter history mode (only in Chat view)
        if self.view == AppView::Chat {
            if let Some(last) = self.last_esc_time {
                if now.duration_since(last).as_millis() < 400 {
                    // Double Esc - enter history mode
                    self.enter_history_mode();
                    self.last_esc_time = None;
                    return false;
                }
            }
            self.last_esc_time = Some(now);
        }

        // Single Esc in Dashboard - exit app
        // Single Esc in Chat - record timestamp for double-tap detection
        self.view == AppView::Dashboard
    }

    /// Enter history mode.
    pub fn enter_history_mode(&mut self) {
        self.view = AppView::History;
        self.history_selected = 0;
        self.show_restore_options = false;
        self.restore_option_selected = 0;
    }

    /// Exit history mode and return to chat.
    pub fn exit_history_mode(&mut self) {
        self.view = AppView::Chat;
        self.show_restore_options = false;
    }

    /// Select next message in history view.
    pub fn history_next(&mut self) {
        if self.show_restore_options {
            // Navigate restore options
            let opts = RestoreOption::all();
            self.restore_option_selected = (self.restore_option_selected + 1) % opts.len();
        } else if !self.messages.is_empty() {
            self.history_selected = (self.history_selected + 1).min(self.messages.len() - 1);
        }
    }

    /// Select previous message in history view.
    pub fn history_prev(&mut self) {
        if self.show_restore_options {
            // Navigate restore options
            let opts = RestoreOption::all();
            self.restore_option_selected = self
                .restore_option_selected
                .checked_sub(1)
                .unwrap_or(opts.len() - 1);
        } else {
            self.history_selected = self.history_selected.saturating_sub(1);
        }
    }

    /// Get the currently selected message in history mode.
    pub fn selected_history_message(&self) -> Option<&ChatMessage> {
        if self.messages.is_empty() {
            return None;
        }
        // Messages are stored oldest-first, history_selected 0 = most recent
        let index = self.messages.len().saturating_sub(1 + self.history_selected);
        self.messages.get(index)
    }

    /// Show restore options popup for the selected message.
    pub fn show_restore_popup(&mut self) {
        self.show_restore_options = true;
        self.restore_option_selected = 0;
    }

    /// Get the currently selected restore option.
    pub fn selected_restore_option(&self) -> RestoreOption {
        RestoreOption::all()
            .get(self.restore_option_selected)
            .copied()
            .unwrap_or(RestoreOption::Cancel)
    }

    /// Confirm the selected restore option.
    /// Returns the option if one was selected, or None if cancelled.
    pub fn confirm_restore(&mut self) -> Option<(RestoreOption, usize)> {
        if !self.show_restore_options {
            return None;
        }

        let option = self.selected_restore_option();
        self.show_restore_options = false;

        if option == RestoreOption::Cancel {
            None
        } else {
            Some((option, self.history_selected))
        }
    }

    /// Check if a message index has a snapshot.
    pub fn message_has_snapshot(&self, index: usize) -> bool {
        self.snapshot_message_indices.contains(&index)
    }

    /// Mark a message as having a snapshot.
    pub fn add_snapshot_marker(&mut self, index: usize) {
        if !self.snapshot_message_indices.contains(&index) {
            self.snapshot_message_indices.push(index);
        }
    }

    /// Go directly to a specific view.
    pub fn go_to_view(&mut self, view: AppView) {
        self.view = view;
    }

    /// Handle a slash command. Returns true if it was a command.
    pub fn handle_command(&mut self, input: &str) -> Option<SlashCommand> {
        if !input.starts_with('/') {
            return None;
        }

        let parts: Vec<&str> = input.split_whitespace().collect();
        let cmd = parts.first().copied().unwrap_or("");
        let args: Vec<&str> = parts.iter().skip(1).copied().collect();

        match cmd {
            "/help" | "/h" | "/?" => Some(SlashCommand::Help),
            "/projects" | "/p" => Some(SlashCommand::ListProjects),
            "/new" | "/create" => {
                let name = if args.is_empty() {
                    None
                } else {
                    Some(args.join(" "))
                };
                Some(SlashCommand::NewProject { name })
            }
            "/switch" | "/s" => {
                if let Some(id) = args.first() {
                    Some(SlashCommand::SwitchProject {
                        id: (*id).to_string(),
                    })
                } else {
                    self.add_message("system", "Usage: /switch <project_id>");
                    None
                }
            }
            "/clear" => Some(SlashCommand::ClearChat),
            "/diff" => {
                self.view = AppView::Diff;
                None
            }
            "/log" => {
                self.view = AppView::Log;
                None
            }
            "/chat" => {
                self.view = AppView::Chat;
                None
            }
            "/quit" | "/q" | "/exit" => Some(SlashCommand::Quit),
            _ => {
                self.add_message(
                    "system",
                    format!("Unknown command: {cmd}. Type /help for available commands."),
                );
                None
            }
        }
    }

    /// Add input to history.
    pub fn add_to_history(&mut self, input: &str) {
        if input.is_empty() {
            return;
        }
        // Don't add duplicates of the last entry
        if self.input_history.back().is_some_and(|last| last == input) {
            return;
        }
        self.input_history.push_back(input.to_string());
        // Keep last 100 entries
        while self.input_history.len() > 100 {
            self.input_history.pop_front();
        }
        self.history_pos = None;
    }

    /// Navigate up in input history.
    pub fn history_up(&mut self) {
        if self.input_history.is_empty() {
            return;
        }
        match self.history_pos {
            None => {
                // Start from the end
                self.history_pos = Some(self.input_history.len().saturating_sub(1));
            }
            Some(pos) if pos > 0 => {
                self.history_pos = Some(pos - 1);
            }
            _ => return,
        }
        if let Some(pos) = self.history_pos
            && let Some(entry) = self.input_history.get(pos)
        {
            self.input = entry.clone();
        }
    }

    /// Navigate down in input history.
    pub fn history_down(&mut self) {
        let Some(pos) = self.history_pos else {
            return;
        };

        if pos + 1 < self.input_history.len() {
            self.history_pos = Some(pos + 1);
            if let Some(entry) = self.input_history.get(pos + 1) {
                self.input = entry.clone();
            }
        } else {
            // Past the end - clear input
            self.history_pos = None;
            self.input.clear();
        }
    }

    /// Select the next project in the list.
    pub fn select_next_project(&mut self) {
        if !self.projects.is_empty() {
            self.projects_selected = (self.projects_selected + 1) % self.projects.len();
        }
    }

    /// Select the previous project in the list.
    pub fn select_prev_project(&mut self) {
        if !self.projects.is_empty() {
            self.projects_selected = if self.projects_selected == 0 {
                self.projects.len() - 1
            } else {
                self.projects_selected - 1
            };
        }
    }

    /// Get the currently selected project.
    pub fn selected_project(&self) -> Option<&ProjectInfo> {
        self.projects.get(self.projects_selected)
    }

    /// Add a chat message.
    pub fn add_message(&mut self, sender: impl Into<String>, content: impl Into<String>) {
        self.messages.push_back(ChatMessage {
            sender: sender.into(),
            content: content.into(),
        });
        // Keep last 500 messages
        while self.messages.len() > 500 {
            self.messages.pop_front();
        }
    }

    /// Add a log line.
    pub fn add_log_line(&mut self, line: impl Into<String>) {
        self.log_lines.push(line.into());
        // Keep last 2000 lines
        if self.log_lines.len() > 2000 {
            self.log_lines.drain(..100);
        }
    }

    /// Append content to the last message (for streaming).
    /// If there's no message or the last message is from a different sender,
    /// creates a new message instead.
    pub fn append_to_message(&mut self, sender: impl Into<String>, content: &str) {
        let sender = sender.into();
        // Increment token counter for mouth animation
        self.token_count = self.token_count.wrapping_add(1);

        if let Some(last) = self.messages.back_mut()
            && last.sender == sender
        {
            last.content.push_str(content);
            return;
        }
        // No existing message to append to, create new one
        self.add_message(sender, content);
    }

    /// Returns whether the mascot's mouth should be open (for animation).
    /// Alternates based on token count during streaming.
    pub fn mouth_open(&self) -> bool {
        self.streaming && (self.token_count % 4 < 2)
    }

    /// Scroll up in the current view.
    pub fn scroll_up(&mut self, amount: u16) {
        match self.view {
            AppView::Chat => {
                self.chat_scroll = self.chat_scroll.saturating_add(amount);
            }
            AppView::Diff => {
                self.diff_scroll = self.diff_scroll.saturating_add(amount);
            }
            AppView::Log => {
                self.log_scroll = self.log_scroll.saturating_add(amount);
            }
            AppView::Dashboard => {
                self.select_prev_project();
            }
            AppView::History => {
                self.history_prev();
            }
            AppView::CreateProject => {
                self.create_project_prev();
            }
            AppView::Help => {}
        }
    }

    /// Scroll down in the current view.
    pub fn scroll_down(&mut self, amount: u16) {
        match self.view {
            AppView::Chat => {
                self.chat_scroll = self.chat_scroll.saturating_sub(amount);
            }
            AppView::Diff => {
                self.diff_scroll = self.diff_scroll.saturating_sub(amount);
            }
            AppView::Log => {
                self.log_scroll = self.log_scroll.saturating_sub(amount);
            }
            AppView::Dashboard => {
                self.select_next_project();
            }
            AppView::History => {
                self.history_next();
            }
            AppView::CreateProject => {
                self.create_project_next();
            }
            AppView::Help => {}
        }
    }

    /// Reset scroll to bottom (most recent).
    pub fn scroll_to_bottom(&mut self) {
        match self.view {
            AppView::Chat => self.chat_scroll = 0,
            AppView::Diff => self.diff_scroll = 0,
            AppView::Log => self.log_scroll = 0,
            AppView::History => self.history_selected = 0,
            AppView::Dashboard | AppView::Help | AppView::CreateProject => {}
        }
    }

    // ========== Create Project Wizard ==========

    /// Start the create project wizard.
    ///
    /// Templates are initially empty (or use previously loaded templates)
    /// and will be populated when the server responds to ListTemplates.
    pub fn start_create_project(&mut self) {
        // Keep existing templates if we have them (from a previous load)
        let templates = if self.create_project.templates.is_empty() {
            // Start with a minimal blank template while loading
            vec![TemplateInfo {
                name: "blank".to_string(),
                description: "Empty project, you define everything".to_string(),
                category: "builtin".to_string(),
                language: "Any".to_string(),
            }]
        } else {
            std::mem::take(&mut self.create_project.templates)
        };

        self.create_project = CreateProjectState {
            step: CreateProjectStep::Name,
            name: String::new(),
            name_valid: false,
            name_message: String::new(),
            templates,
            template_selected: 0,
            mode_selected: 0,
            local_path: String::new(),
        };
        self.view = AppView::CreateProject;
    }

    /// Move to previous item in create project step.
    pub fn create_project_prev(&mut self) {
        match self.create_project.step {
            CreateProjectStep::Name => {}
            CreateProjectStep::Template => {
                if self.create_project.template_selected > 0 {
                    self.create_project.template_selected -= 1;
                } else {
                    self.create_project.template_selected =
                        self.create_project.templates.len().saturating_sub(1);
                }
            }
            CreateProjectStep::Mode => {
                let modes = ExecutionMode::all();
                if self.create_project.mode_selected > 0 {
                    self.create_project.mode_selected -= 1;
                } else {
                    self.create_project.mode_selected = modes.len().saturating_sub(1);
                }
            }
        }
    }

    /// Move to next item in create project step.
    pub fn create_project_next(&mut self) {
        match self.create_project.step {
            CreateProjectStep::Name => {}
            CreateProjectStep::Template => {
                self.create_project.template_selected =
                    (self.create_project.template_selected + 1) % self.create_project.templates.len();
            }
            CreateProjectStep::Mode => {
                let modes = ExecutionMode::all();
                self.create_project.mode_selected =
                    (self.create_project.mode_selected + 1) % modes.len();
            }
        }
    }

    /// Advance to next step in create project wizard.
    /// Returns true if wizard is complete.
    pub fn create_project_advance(&mut self) -> bool {
        match self.create_project.step {
            CreateProjectStep::Name => {
                // Validate name before proceeding
                if self.create_project.name.is_empty() {
                    self.create_project.name_message = "Name cannot be empty".to_string();
                    self.create_project.name_valid = false;
                    return false;
                }
                // Basic validation - alphanumeric, dashes, underscores
                let valid = self.create_project.name.chars().all(|c| {
                    c.is_ascii_alphanumeric() || c == '-' || c == '_'
                });
                if !valid {
                    self.create_project.name_message =
                        "Name can only contain letters, numbers, dashes, and underscores".to_string();
                    self.create_project.name_valid = false;
                    return false;
                }
                self.create_project.name_valid = true;
                self.create_project.step = CreateProjectStep::Template;
                false
            }
            CreateProjectStep::Template => {
                self.create_project.step = CreateProjectStep::Mode;
                // Set default local path based on name
                if self.create_project.local_path.is_empty() {
                    self.create_project.local_path =
                        format!("~/projects/{}", self.create_project.name);
                }
                false
            }
            CreateProjectStep::Mode => {
                // Wizard complete
                true
            }
        }
    }

    /// Get the selected template info.
    pub fn selected_template(&self) -> Option<&TemplateInfo> {
        self.create_project.templates.get(self.create_project.template_selected)
    }

    /// Get the selected execution mode.
    pub fn selected_execution_mode(&self) -> ExecutionMode {
        ExecutionMode::all()
            .get(self.create_project.mode_selected)
            .copied()
            .unwrap_or_default()
    }

    /// Open a project (switch to chat view with this project).
    pub fn open_project(&mut self, project_id: &str) {
        self.project_id = Some(project_id.to_string());
        self.view = AppView::Chat;
        self.messages.clear();
        self.chat_scroll = 0;
    }

    /// Delete the currently selected project from the local list.
    /// Returns the project ID if one was deleted.
    pub fn delete_selected_project(&mut self) -> Option<String> {
        if self.projects.is_empty() {
            return None;
        }

        let project_id = self.projects[self.projects_selected].id.clone();

        // If we're deleting the active project, clear it
        if self.project_id.as_ref().is_some_and(|id| *id == project_id) {
            self.project_id = None;
        }

        // Remove from list
        self.projects.remove(self.projects_selected);

        // Adjust selection
        if self.projects_selected >= self.projects.len() && self.projects_selected > 0 {
            self.projects_selected -= 1;
        }

        Some(project_id)
    }

    /// Set up for local mode (single implicit project, start in Chat).
    pub fn setup_local_mode(&mut self, workdir: &std::path::Path) {
        let project_id = uuid::Uuid::new_v4().to_string();
        let name = workdir
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("local-project")
            .to_string();

        self.projects.push(ProjectInfo {
            id: project_id.clone(),
            name: name.clone(),
            template_name: None,
            execution_mode: ExecutionMode::Local,
            running_status: RunningStatus::Idle,
            local_path: Some(workdir.display().to_string()),
            tunnel_url: None,
            last_activity: "now".to_string(),
            active: true,
        });

        self.project_id = Some(project_id);
        self.view = AppView::Chat;
    }

    /// Clear the chat messages.
    pub fn clear_chat(&mut self) {
        self.messages.clear();
        self.chat_scroll = 0;
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_app_defaults() {
        let app = App::new();
        assert!(!app.should_exit);
        assert_eq!(app.view, AppView::Dashboard);
        assert!(app.input.is_empty());
        assert!(!app.connected);
    }

    #[test]
    fn test_next_view_cycles_without_project() {
        let mut app = App::new();
        assert_eq!(app.view, AppView::Dashboard);
        // Without a project, only Dashboard and Help cycle
        app.next_view();
        assert_eq!(app.view, AppView::Help);
        app.next_view();
        assert_eq!(app.view, AppView::Dashboard);
    }

    #[test]
    fn test_next_view_cycles_with_project() {
        let mut app = App::new();
        app.project_id = Some("test-project".to_string());
        app.view = AppView::Chat;
        app.next_view();
        assert_eq!(app.view, AppView::Diff);
        app.next_view();
        assert_eq!(app.view, AppView::Log);
        app.next_view();
        assert_eq!(app.view, AppView::Help);
        app.next_view();
        assert_eq!(app.view, AppView::Dashboard);
        app.next_view();
        assert_eq!(app.view, AppView::Chat);
    }

    #[test]
    fn test_add_message() {
        let mut app = App::new();
        app.add_message("user", "Hello");
        assert_eq!(app.messages.len(), 1);
        assert_eq!(app.messages[0].sender, "user");
    }

    #[test]
    fn test_message_cap() {
        let mut app = App::new();
        for i in 0..600 {
            app.add_message("user", format!("msg {i}"));
        }
        assert!(app.messages.len() <= 500);
    }

    #[test]
    fn test_handle_command_help() {
        let mut app = App::new();
        let cmd = app.handle_command("/help");
        assert!(matches!(cmd, Some(SlashCommand::Help)));
    }

    #[test]
    fn test_handle_command_unknown() {
        let mut app = App::new();
        let cmd = app.handle_command("/foobar");
        assert!(cmd.is_none());
        assert_eq!(app.messages.len(), 1);
        assert!(app.messages[0].content.contains("Unknown command"));
    }

    #[test]
    fn test_handle_command_not_command() {
        let mut app = App::new();
        let cmd = app.handle_command("hello world");
        assert!(cmd.is_none());
        assert!(app.messages.is_empty());
    }

    #[test]
    fn test_input_history() {
        let mut app = App::new();
        app.add_to_history("first");
        app.add_to_history("second");
        app.add_to_history("third");

        app.history_up();
        assert_eq!(app.input, "third");
        app.history_up();
        assert_eq!(app.input, "second");
        app.history_down();
        assert_eq!(app.input, "third");
        app.history_down();
        assert!(app.input.is_empty());
    }

    fn make_test_project(id: &str, name: &str, active: bool) -> ProjectInfo {
        ProjectInfo {
            id: id.to_string(),
            name: name.to_string(),
            template_name: None,
            execution_mode: ExecutionMode::Cloud,
            running_status: RunningStatus::Idle,
            local_path: None,
            tunnel_url: None,
            last_activity: "now".to_string(),
            active,
        }
    }

    #[test]
    fn test_project_selection() {
        let mut app = App::new();
        app.projects = vec![
            make_test_project("1", "First", true),
            make_test_project("2", "Second", false),
            make_test_project("3", "Third", false),
        ];

        assert_eq!(app.projects_selected, 0);
        app.select_next_project();
        assert_eq!(app.projects_selected, 1);
        app.select_next_project();
        assert_eq!(app.projects_selected, 2);
        app.select_next_project();
        assert_eq!(app.projects_selected, 0); // Wraps around

        app.select_prev_project();
        assert_eq!(app.projects_selected, 2); // Wraps around
    }

    #[test]
    fn test_history_mode_entry() {
        let mut app = App::new();
        app.view = AppView::Chat; // Start in Chat for this test

        app.enter_history_mode();
        assert_eq!(app.view, AppView::History);
        assert_eq!(app.history_selected, 0);
        assert!(!app.show_restore_options);
    }

    #[test]
    fn test_history_navigation() {
        let mut app = App::new();
        app.add_message("user", "First");
        app.add_message("assistant", "Response 1");
        app.add_message("user", "Second");
        app.add_message("assistant", "Response 2");

        app.enter_history_mode();

        // history_selected 0 = most recent message
        assert_eq!(app.history_selected, 0);
        let msg = app.selected_history_message().expect("has message");
        assert_eq!(msg.content, "Response 2");

        app.history_next();
        assert_eq!(app.history_selected, 1);
        let msg = app.selected_history_message().expect("has message");
        assert_eq!(msg.content, "Second");

        app.history_prev();
        assert_eq!(app.history_selected, 0);
    }

    #[test]
    fn test_restore_options() {
        let mut app = App::new();
        app.add_message("user", "Test");

        app.enter_history_mode();
        app.show_restore_popup();
        assert!(app.show_restore_options);

        assert_eq!(app.selected_restore_option(), RestoreOption::Conversation);

        app.history_next(); // Navigate options
        assert_eq!(app.selected_restore_option(), RestoreOption::Code);

        app.history_next();
        assert_eq!(app.selected_restore_option(), RestoreOption::Both);

        app.history_next();
        assert_eq!(app.selected_restore_option(), RestoreOption::Cancel);
    }

    #[test]
    fn test_restore_option_labels() {
        assert_eq!(RestoreOption::Conversation.label(), "Restore Conversation");
        assert_eq!(RestoreOption::Code.label(), "Restore Code");
        assert_eq!(RestoreOption::Both.label(), "Restore Code & Conversation");
        assert_eq!(RestoreOption::Cancel.label(), "Cancel");
    }

    #[test]
    fn test_snapshot_markers() {
        let mut app = App::new();
        assert!(!app.message_has_snapshot(0));

        app.add_snapshot_marker(0);
        assert!(app.message_has_snapshot(0));
        assert!(!app.message_has_snapshot(1));

        // Adding same marker again shouldn't duplicate
        app.add_snapshot_marker(0);
        assert_eq!(app.snapshot_message_indices.len(), 1);
    }
}
