//! TUI application state management.

use std::collections::VecDeque;

/// A message displayed in the chat view.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ChatMessage {
    /// Message sender (e.g., "user", "coder", "orchestrator").
    pub sender: String,
    /// Message content.
    pub content: String,
}

/// The current view tab.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AppView {
    /// Chat with agents.
    #[default]
    Chat,
    /// File diff viewer.
    Diff,
    /// Build log viewer.
    Log,
}

/// Application state for the TUI.
#[allow(dead_code)]
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
}

#[allow(dead_code)]
impl App {
    /// Create a new application state.
    pub fn new() -> Self {
        Self {
            should_exit: false,
            view: AppView::Chat,
            input: String::new(),
            messages: VecDeque::new(),
            log_lines: Vec::new(),
            diff_content: String::new(),
            project_id: None,
            connected: false,
        }
    }

    /// Cycle to the next view tab.
    pub fn next_view(&mut self) {
        self.view = match self.view {
            AppView::Chat => AppView::Diff,
            AppView::Diff => AppView::Log,
            AppView::Log => AppView::Chat,
        };
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
        assert_eq!(app.view, AppView::Chat);
        assert!(app.input.is_empty());
        assert!(!app.connected);
    }

    #[test]
    fn test_next_view_cycles() {
        let mut app = App::new();
        assert_eq!(app.view, AppView::Chat);
        app.next_view();
        assert_eq!(app.view, AppView::Diff);
        app.next_view();
        assert_eq!(app.view, AppView::Log);
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
}
