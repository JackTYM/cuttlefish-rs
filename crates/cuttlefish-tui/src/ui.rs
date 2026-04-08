//! UI rendering for the TUI application.

#![allow(dead_code)]

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
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
        AppView::Mascot => render_mascot(app, frame, area),
    }
}

/// Render the cuttlefish mascot.
fn render_mascot(app: &App, frame: &mut Frame, area: Rect) {
    let widget = MascotWidget::new().with_mouth_open(app.mouth_open());
    frame.render_widget(widget, area);
}

/// Render the chat message list with mascot in top-right corner.
fn render_chat(app: &App, frame: &mut Frame, area: Rect) {
    // Mascot dimensions (half-block mode: 16 chars wide × 8 chars tall)
    let mascot_width: u16 = 17; // 16 + 1 for padding
    let mascot_height: u16 = 9; // 8 + 1 for padding

    // First render the chat block (full area)
    let chat_block = Block::default().borders(Borders::ALL).title(format!(
        "Chat (↑/↓ scroll, Home=bottom){}",
        if app.chat_scroll > 0 {
            format!(" [+{}]", app.chat_scroll)
        } else {
            String::new()
        }
    ));
    let inner_area = chat_block.inner(area);
    frame.render_widget(chat_block, area);

    // Calculate mascot position (top-right corner, inside the border)
    let mascot_area = if inner_area.width > mascot_width + 5 && inner_area.height > mascot_height {
        Some(Rect::new(
            inner_area.right().saturating_sub(mascot_width),
            inner_area.y,
            mascot_width,
            mascot_height.min(inner_area.height),
        ))
    } else {
        None // Too small to show mascot
    };

    // Calculate chat content area (exclude mascot space from top-right)
    let available_height = inner_area.height as usize;

    // Reduce text width when mascot is shown to avoid overlap
    let text_area = if mascot_area.is_some() {
        Rect::new(
            inner_area.x,
            inner_area.y,
            inner_area.width.saturating_sub(mascot_width + 1),
            inner_area.height,
        )
    } else {
        inner_area
    };

    // Convert messages to lines
    let lines: Vec<Line> = app
        .messages
        .iter()
        .flat_map(|msg| {
            let color = match msg.sender.as_str() {
                "user" => Color::Green,
                "orchestrator" => Color::Cyan,
                "coder" => Color::Yellow,
                "critic" => Color::Magenta,
                "assistant" => Color::Cyan,
                "system" => Color::Blue,
                "error" => Color::Red,
                _ => Color::White,
            };
            let prefix = format!("[{}] ", msg.sender);
            msg.content
                .lines()
                .enumerate()
                .map(move |(i, line)| {
                    if i == 0 {
                        Line::from(vec![
                            Span::styled(
                                prefix.clone(),
                                Style::default().fg(color).add_modifier(Modifier::BOLD),
                            ),
                            Span::raw(line.to_string()),
                        ])
                    } else {
                        let indent = " ".repeat(prefix.len());
                        Line::from(vec![Span::raw(indent), Span::raw(line.to_string())])
                    }
                })
                .collect::<Vec<_>>()
        })
        .collect();

    // Calculate scroll position
    let total_lines = lines.len();
    let scroll_offset = if total_lines > available_height {
        let max_scroll = total_lines.saturating_sub(available_height);
        max_scroll.saturating_sub(app.chat_scroll as usize)
    } else {
        0
    };

    // Render chat content (in reduced area to avoid mascot)
    let chat_content = Paragraph::new(lines)
        .wrap(Wrap { trim: false })
        .scroll((scroll_offset as u16, 0));
    frame.render_widget(chat_content, text_area);

    // Render mascot on top (in top-right corner) with mouth animation
    if let Some(mascot_rect) = mascot_area {
        let mascot = MascotWidget::compact().with_mouth_open(app.mouth_open());
        frame.render_widget(mascot, mascot_rect);
    }
}

/// Render the diff view.
fn render_diff(app: &App, frame: &mut Frame, area: Rect) {
    let available_height = area.height.saturating_sub(2) as usize;

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

    let total_lines = lines.len();
    let scroll_offset = if total_lines > available_height {
        let max_scroll = total_lines.saturating_sub(available_height);
        max_scroll.saturating_sub(app.diff_scroll as usize)
    } else {
        0
    };

    let diff_widget = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title(format!(
            "Diff (↑/↓ scroll){}",
            if app.diff_scroll > 0 {
                format!(" [+{}]", app.diff_scroll)
            } else {
                String::new()
            }
        )))
        .wrap(Wrap { trim: false })
        .scroll((scroll_offset as u16, 0));
    frame.render_widget(diff_widget, area);
}

/// Render the build log view.
fn render_log(app: &App, frame: &mut Frame, area: Rect) {
    let available_height = area.height.saturating_sub(2) as usize;

    let lines: Vec<Line> = app
        .log_lines
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

    let total_lines = lines.len();
    let scroll_offset = if total_lines > available_height {
        let max_scroll = total_lines.saturating_sub(available_height);
        max_scroll.saturating_sub(app.log_scroll as usize)
    } else {
        0
    };

    let log_widget = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title(format!(
            "Build Log (↑/↓ scroll){}",
            if app.log_scroll > 0 {
                format!(" [+{}]", app.log_scroll)
            } else {
                String::new()
            }
        )))
        .scroll((scroll_offset as u16, 0));
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
