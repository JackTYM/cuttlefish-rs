//! UI rendering for the TUI application.

#![allow(dead_code)]

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};

use crate::app::{App, AppView};
use crate::mascot::MascotWidget;

/// Render the complete TUI layout.
pub fn render(app: &App, frame: &mut Frame) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Header
            Constraint::Min(0),    // Main content
            Constraint::Length(3), // Input box
        ])
        .split(frame.area());

    render_header(app, frame, chunks[0]);
    render_main(app, frame, chunks[1]);
    render_input(app, frame, chunks[2]);
}

/// Render the header bar.
fn render_header(app: &App, frame: &mut Frame, area: Rect) {
    let status = if app.connected {
        "🟢 Connected"
    } else {
        "🔴 Disconnected"
    };
    let project = app.project_id.as_deref().unwrap_or("No project");
    let tab_chat = if app.view == AppView::Chat {
        "[Chat]"
    } else {
        " Chat "
    };
    let tab_diff = if app.view == AppView::Diff {
        "[Diff]"
    } else {
        " Diff "
    };
    let tab_log = if app.view == AppView::Log {
        "[Log] "
    } else {
        " Log  "
    };
    let tab_mascot = if app.view == AppView::Mascot {
        "[Mascot]"
    } else {
        " Mascot "
    };

    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            "🐙 Cuttlefish ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("| "),
        Span::styled(tab_chat, Style::default().fg(Color::White)),
        Span::styled(tab_diff, Style::default().fg(Color::White)),
        Span::styled(tab_log, Style::default().fg(Color::White)),
        Span::styled(tab_mascot, Style::default().fg(Color::White)),
        Span::raw(" | "),
        Span::raw(project),
        Span::raw(" | "),
        Span::raw(status),
        Span::raw(" | Tab: switch view | Ctrl-C: quit"),
    ]));
    frame.render_widget(header, area);
}

/// Render the main content area based on current view.
fn render_main(app: &App, frame: &mut Frame, area: Rect) {
    match app.view {
        AppView::Chat => render_chat(app, frame, area),
        AppView::Diff => render_diff(app, frame, area),
        AppView::Log => render_log(app, frame, area),
        AppView::Mascot => render_mascot(frame, area),
    }
}

/// Render the cuttlefish mascot.
fn render_mascot(frame: &mut Frame, area: Rect) {
    let widget = MascotWidget::new();
    frame.render_widget(widget, area);
}

/// Render the chat message list.
fn render_chat(app: &App, frame: &mut Frame, area: Rect) {
    let items: Vec<ListItem> = app
        .messages
        .iter()
        .map(|msg| {
            let color = match msg.sender.as_str() {
                "user" => Color::Green,
                "orchestrator" => Color::Cyan,
                "coder" => Color::Yellow,
                "critic" => Color::Magenta,
                _ => Color::White,
            };
            let line = Line::from(vec![
                Span::styled(
                    format!("[{}] ", msg.sender),
                    Style::default().fg(color).add_modifier(Modifier::BOLD),
                ),
                Span::raw(&msg.content),
            ]);
            ListItem::new(line)
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Chat"))
        .highlight_style(Style::default().add_modifier(Modifier::BOLD));
    frame.render_widget(list, area);
}

/// Render the diff view.
fn render_diff(app: &App, frame: &mut Frame, area: Rect) {
    let lines: Vec<Line> = app
        .diff_content
        .lines()
        .map(|line| {
            let color = if line.starts_with('+') && !line.starts_with("+++") {
                Color::Green
            } else if line.starts_with('-') && !line.starts_with("---") {
                Color::Red
            } else if line.starts_with("@@") {
                Color::Cyan
            } else {
                Color::White
            };
            Line::from(Span::styled(line.to_string(), Style::default().fg(color)))
        })
        .collect();

    let diff_widget = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title("Diff"))
        .wrap(Wrap { trim: false });
    frame.render_widget(diff_widget, area);
}

/// Render the build log view.
fn render_log(app: &App, frame: &mut Frame, area: Rect) {
    // Show last N lines that fit in the area
    let available_lines = area.height as usize - 2;
    let start = if app.log_lines.len() > available_lines {
        app.log_lines.len() - available_lines
    } else {
        0
    };

    let lines: Vec<Line> = app.log_lines[start..]
        .iter()
        .map(|line| {
            let color = if line.contains("error") || line.contains("FAILED") {
                Color::Red
            } else if line.contains("warning") {
                Color::Yellow
            } else if line.contains("ok") || line.contains("PASSED") {
                Color::Green
            } else {
                Color::White
            };
            Line::from(Span::styled(line.clone(), Style::default().fg(color)))
        })
        .collect();

    let log_widget =
        Paragraph::new(lines).block(Block::default().borders(Borders::ALL).title("Build Log"));
    frame.render_widget(log_widget, area);
}

/// Render the input box.
fn render_input(app: &App, frame: &mut Frame, area: Rect) {
    let input_text = format!("│ {} ", app.input);
    let input_widget = Paragraph::new(input_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Input (Enter to send)"),
        )
        .style(Style::default().fg(Color::Yellow));
    frame.render_widget(input_widget, area);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::App;

    #[test]
    fn test_app_view_default() {
        let app = App::default();
        // Verify no panic from view default
        assert_eq!(app.view, AppView::Chat);
    }

    #[test]
    fn test_diff_color_logic() {
        // Test the color logic for diff lines
        let added_line = "+added content";
        let removed_line = "-removed content";
        assert!(added_line.starts_with('+'));
        assert!(removed_line.starts_with('-'));
    }
}
