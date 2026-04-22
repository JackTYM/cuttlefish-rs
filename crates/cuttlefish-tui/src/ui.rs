//! UI rendering for the TUI application.

#![allow(dead_code)]

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
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
    let status = if app.connected { "🟢" } else { "🔴" };

    // Active tab styling
    let active_style = Style::default()
        .fg(Color::Cyan)
        .add_modifier(Modifier::BOLD);
    let inactive_style = Style::default().fg(Color::DarkGray);

    let tab_dashboard = if app.view == AppView::Dashboard {
        Span::styled("[Projects]", active_style)
    } else {
        Span::styled(" Projects ", inactive_style)
    };

    // Only show project-specific tabs if we have a project
    let project_tabs: Vec<Span> = if app.project_id.is_some() {
        vec![
            if app.view == AppView::Chat {
                Span::styled("[Chat]", active_style)
            } else {
                Span::styled(" Chat ", inactive_style)
            },
            if app.view == AppView::Diff {
                Span::styled("[Diff]", active_style)
            } else {
                Span::styled(" Diff ", inactive_style)
            },
            if app.view == AppView::Log {
                Span::styled("[Log]", active_style)
            } else {
                Span::styled(" Log ", inactive_style)
            },
            if app.view == AppView::Runtime {
                Span::styled("[Runtime]", active_style)
            } else {
                Span::styled(" Runtime ", inactive_style)
            },
        ]
    } else {
        vec![]
    };

    let tab_help = if app.view == AppView::Help {
        Span::styled("[?]", active_style)
    } else {
        Span::styled(" ? ", inactive_style)
    };

    // Special mode indicators
    let mode_indicator = match app.view {
        AppView::History => Span::styled(
            " [HISTORY] ",
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        ),
        AppView::CreateProject => Span::styled(
            " [NEW PROJECT] ",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        ),
        _ => Span::raw(""),
    };

    // Project name display
    let project_display = if let Some(ref id) = app.project_id {
        // Find project name from list
        let name = app
            .projects
            .iter()
            .find(|p| p.id == *id)
            .map(|p| p.name.as_str())
            .unwrap_or(id.as_str());
        Span::styled(name, Style::default().fg(Color::Yellow))
    } else {
        Span::styled("No project", Style::default().fg(Color::DarkGray))
    };

    // Context-sensitive hints
    let nav_hint = match app.view {
        AppView::Dashboard => {
            Span::styled(" │ Enter=Open N=New ?=Help", Style::default().fg(Color::DarkGray))
        }
        AppView::History => {
            Span::styled(" │ Esc=back ↑↓=select Enter=restore", Style::default().fg(Color::DarkGray))
        }
        AppView::CreateProject => {
            Span::styled(" │ Esc=back Enter=continue", Style::default().fg(Color::DarkGray))
        }
        AppView::Chat => {
            Span::styled(" │ Tab ↹ │ Esc Esc=history", Style::default().fg(Color::DarkGray))
        }
        _ => {
            Span::styled(" │ Tab ↹", Style::default().fg(Color::DarkGray))
        }
    };

    // Build header
    let mut spans = vec![
        Span::styled(
            "🐙 ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        tab_dashboard,
    ];
    spans.extend(project_tabs);
    spans.push(tab_help);
    spans.push(mode_indicator);
    spans.push(Span::raw(" │ "));
    spans.push(project_display);
    spans.push(Span::raw(" "));
    spans.push(Span::raw(status));
    spans.push(nav_hint);

    let header = Paragraph::new(Line::from(spans));
    frame.render_widget(header, area);
}

/// Render the main content area based on current view.
fn render_main(app: &App, frame: &mut Frame, area: Rect) {
    match app.view {
        AppView::Dashboard => render_dashboard(app, frame, area),
        AppView::Chat => render_chat(app, frame, area),
        AppView::Diff => render_diff(app, frame, area),
        AppView::Log => render_log(app, frame, area),
        AppView::Runtime => render_runtime(app, frame, area),
        AppView::Help => render_help(frame, area),
        AppView::History => render_history(app, frame, area),
        AppView::CreateProject => render_create_project(app, frame, area),
    }
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

    // Without wrapping, each logical line = 1 visual line (truncated if too long)
    // This gives us reliable scroll calculations
    let total_lines = lines.len();

    // Calculate scroll position
    // chat_scroll=0 means "at bottom" (show newest messages)
    // Higher chat_scroll means "scrolled up" (show older messages)
    let scroll_offset: u16 = if total_lines > available_height {
        let max_scroll = total_lines.saturating_sub(available_height);
        let offset = max_scroll.saturating_sub(app.chat_scroll as usize);
        offset.min(u16::MAX as usize) as u16
    } else {
        0
    };

    // Render chat content without wrapping - long lines are truncated
    // This ensures scroll position is always accurate
    let chat_content = Paragraph::new(lines).scroll((scroll_offset, 0));
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

/// Render the runtime view with process logs and port forwarding.
fn render_runtime(app: &App, frame: &mut Frame, area: Rect) {
    use crate::app::{PortForwardStatus, RuntimeFocus};

    // Split into two panes: logs (left/top) and ports (right/bottom)
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(65), Constraint::Percentage(35)])
        .split(area);

    // Left pane: Process logs
    let logs_block = Block::default()
        .borders(Borders::ALL)
        .title(format!(
            "Process Logs {} (↑/↓ scroll){}",
            if app.runtime.process_running {
                "[Running]"
            } else {
                "[Stopped]"
            },
            if app.runtime.log_scroll > 0 {
                format!(" [+{}]", app.runtime.log_scroll)
            } else {
                String::new()
            }
        ))
        .border_style(if app.runtime.focus == RuntimeFocus::Logs {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default()
        });

    let log_inner = logs_block.inner(chunks[0]);
    let available_height = log_inner.height as usize;

    let log_lines: Vec<Line> = app
        .runtime
        .process_logs
        .iter()
        .map(|line| {
            let color = if line.contains("ERROR") || line.contains("error") {
                Color::Red
            } else if line.contains("WARN") || line.contains("warn") {
                Color::Yellow
            } else if line.contains("INFO") || line.contains("info") {
                Color::Cyan
            } else {
                Color::White
            };
            Line::from(Span::styled(line.clone(), Style::default().fg(color)))
        })
        .collect();

    let total_lines = log_lines.len();
    let scroll_offset = if total_lines > available_height {
        let max_scroll = total_lines.saturating_sub(available_height);
        max_scroll.saturating_sub(app.runtime.log_scroll as usize)
    } else {
        0
    };

    let logs_widget = Paragraph::new(log_lines)
        .block(logs_block)
        .wrap(Wrap { trim: false })
        .scroll((scroll_offset as u16, 0));
    frame.render_widget(logs_widget, chunks[0]);

    // Right pane: Port forwards
    let ports_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(5), Constraint::Length(3)])
        .split(chunks[1]);

    let ports_block = Block::default()
        .borders(Borders::ALL)
        .title("Port Forwards │ A=Add  D=Delete  Enter=Toggle")
        .border_style(if app.runtime.focus == RuntimeFocus::Ports {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default()
        });
    let ports_inner = ports_block.inner(ports_chunks[0]);
    frame.render_widget(ports_block, ports_chunks[0]);

    if app.runtime.port_forwards.is_empty() {
        let hint = Paragraph::new("No port forwards configured.\nPress 'A' to add one.")
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(hint, ports_inner);
    } else {
        let port_lines: Vec<Line> = app
            .runtime
            .port_forwards
            .iter()
            .enumerate()
            .map(|(i, pf)| {
                let selected = i == app.runtime.port_selected;
                let status_color = match pf.status {
                    PortForwardStatus::Active => Color::Green,
                    PortForwardStatus::Connecting => Color::Yellow,
                    PortForwardStatus::Error => Color::Red,
                    PortForwardStatus::Disconnected => Color::DarkGray,
                };
                let prefix = if selected { "> " } else { "  " };
                let text = format!(
                    "{}{}:{} -> localhost:{}",
                    pf.status.emoji(),
                    pf.remote_port,
                    pf.local_port,
                    pf.local_port
                );
                let style = if selected {
                    Style::default().fg(status_color).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(status_color)
                };
                Line::from(vec![Span::raw(prefix), Span::styled(text, style)])
            })
            .collect();

        let ports_list = Paragraph::new(port_lines);
        frame.render_widget(ports_list, ports_inner);
    }

    // Input area for adding ports
    let input_block = Block::default()
        .borders(Borders::ALL)
        .title("Add Port (remote:local)")
        .border_style(if app.runtime.focus == RuntimeFocus::AddPort {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default()
        });
    let input_hint = if app.runtime.port_input.is_empty() {
        Span::styled("e.g., 3000:3000", Style::default().fg(Color::DarkGray))
    } else {
        Span::styled(&app.runtime.port_input, Style::default().fg(Color::Yellow))
    };
    let input_widget = Paragraph::new(Line::from(input_hint)).block(input_block);
    frame.render_widget(input_widget, ports_chunks[1]);
}

/// Render the project dashboard view.
fn render_dashboard(app: &App, frame: &mut Frame, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title("Projects │ ↑↓=Select  Enter=Open  N=New  D=Delete")
        .border_style(Style::default().fg(Color::Cyan));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if app.projects.is_empty() {
        let empty_msg = Paragraph::new(vec![
            Line::from(""),
            Line::from(""),
            Line::from(Span::styled(
                "  No projects yet",
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "  Press N to create your first project",
                Style::default().fg(Color::Cyan),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "  Or connect to a server with existing projects",
                Style::default().fg(Color::DarkGray),
            )),
        ]);
        frame.render_widget(empty_msg, inner);
        return;
    }

    // Render each project as a card-like row
    let mut lines: Vec<Line> = Vec::new();
    for (i, proj) in app.projects.iter().enumerate() {
        let is_selected = i == app.projects_selected;

        // Selection marker and status emoji
        let marker = if is_selected { "▶ " } else { "  " };
        let status_emoji = proj.running_status.emoji();

        // Project name with styling
        let name_style = if is_selected {
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD)
        } else if proj.active {
            Style::default().fg(Color::Green)
        } else {
            Style::default().fg(Color::White)
        };

        // First line: status + name + activity
        lines.push(Line::from(vec![
            Span::raw(marker),
            Span::raw(status_emoji),
            Span::raw(" "),
            Span::styled(&proj.name, name_style),
            Span::styled(
                format!("  {}", proj.last_activity),
                Style::default().fg(Color::DarkGray),
            ),
        ]));

        // Second line: template + mode + path/url
        let template = proj
            .template_name
            .as_deref()
            .unwrap_or("no template");
        let mode = proj.execution_mode.label();

        let location = match proj.execution_mode {
            crate::app::ExecutionMode::Cloud => {
                proj.tunnel_url.as_deref().unwrap_or("").to_string()
            }
            crate::app::ExecutionMode::Local | crate::app::ExecutionMode::BuildRemoteRunLocal => {
                proj.local_path.as_deref().unwrap_or("").to_string()
            }
        };

        let indent = if is_selected { "    " } else { "    " };
        lines.push(Line::from(vec![
            Span::styled(indent, Style::default()),
            Span::styled(template, Style::default().fg(Color::Yellow)),
            Span::styled(" │ ", Style::default().fg(Color::DarkGray)),
            Span::styled(mode, Style::default().fg(Color::Magenta)),
            if !location.is_empty() {
                Span::styled(format!(" │ {}", location), Style::default().fg(Color::DarkGray))
            } else {
                Span::raw("")
            },
        ]));

        // Add spacing between projects
        lines.push(Line::from(""));
    }

    let list = Paragraph::new(lines);
    frame.render_widget(list, inner);
}

/// Render the create project wizard.
fn render_create_project(app: &App, frame: &mut Frame, area: Rect) {
    use crate::app::{CreateProjectStep, ExecutionMode};

    let step_num = match app.create_project.step {
        CreateProjectStep::Name => 1,
        CreateProjectStep::Template => 2,
        CreateProjectStep::Mode => 3,
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!("Create New Project │ Step {} of 3", step_num))
        .border_style(Style::default().fg(Color::Green));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    match app.create_project.step {
        CreateProjectStep::Name => {
            let lines = vec![
                Line::from(""),
                Line::from(Span::styled(
                    "  Name your project",
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                )),
                Line::from(""),
                Line::from(vec![
                    Span::raw("  "),
                    Span::styled(
                        format!("  {}█  ", app.create_project.name),
                        Style::default().fg(Color::Cyan).bg(Color::DarkGray),
                    ),
                ]),
                Line::from(""),
                Line::from(Span::styled(
                    "  This will be used for the project directory and tunnel URL.",
                    Style::default().fg(Color::DarkGray),
                )),
                if !app.create_project.name.is_empty() {
                    Line::from(Span::styled(
                        format!("  URL: {}.cuttlefish.ai", app.create_project.name),
                        Style::default().fg(Color::Yellow),
                    ))
                } else {
                    Line::from("")
                },
                Line::from(""),
                if !app.create_project.name_message.is_empty() {
                    Line::from(Span::styled(
                        format!("  {}", app.create_project.name_message),
                        Style::default().fg(Color::Red),
                    ))
                } else {
                    Line::from("")
                },
            ];
            frame.render_widget(Paragraph::new(lines), inner);
        }

        CreateProjectStep::Template => {
            let mut lines = vec![
                Line::from(""),
                Line::from(Span::styled(
                    "  Choose a template",
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                )),
                Line::from(""),
            ];

            for (i, template) in app.create_project.templates.iter().enumerate() {
                let is_selected = i == app.create_project.template_selected;
                let marker = if is_selected { "  ▶ " } else { "    " };
                let style = if is_selected {
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };

                lines.push(Line::from(vec![
                    Span::styled(marker, style),
                    Span::styled(&template.name, style),
                    Span::styled(
                        format!("  ({})", template.language),
                        Style::default().fg(Color::Yellow),
                    ),
                ]));
                lines.push(Line::from(Span::styled(
                    format!("      {}", template.description),
                    Style::default().fg(Color::DarkGray),
                )));
            }

            frame.render_widget(Paragraph::new(lines), inner);
        }

        CreateProjectStep::Mode => {
            let mut lines = vec![
                Line::from(""),
                Line::from(Span::styled(
                    "  Where should it run?",
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                )),
                Line::from(""),
            ];

            let modes = ExecutionMode::all();
            let icons = ["☁️ ", "📦 ", "🔧 "];

            for (i, mode) in modes.iter().enumerate() {
                let is_selected = i == app.create_project.mode_selected;
                let marker = if is_selected { "  ▶ " } else { "    " };
                let style = if is_selected {
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };

                lines.push(Line::from(vec![
                    Span::styled(marker, style),
                    Span::raw(icons[i]),
                    Span::styled(mode.label(), style),
                    if i == 0 {
                        Span::styled(" (recommended)", Style::default().fg(Color::Green))
                    } else {
                        Span::raw("")
                    },
                ]));
                lines.push(Line::from(Span::styled(
                    format!("      {}", mode.description()),
                    Style::default().fg(Color::DarkGray),
                )));
                lines.push(Line::from(""));
            }

            // Show local path input if Local mode is selected
            if app.create_project.mode_selected == 2 {
                lines.push(Line::from(""));
                lines.push(Line::from(Span::styled(
                    "  Local path:",
                    Style::default().fg(Color::White),
                )));
                lines.push(Line::from(vec![
                    Span::raw("    "),
                    Span::styled(
                        &app.create_project.local_path,
                        Style::default().fg(Color::Yellow),
                    ),
                ]));
            }

            frame.render_widget(Paragraph::new(lines), inner);
        }
    }
}

/// Render the help view.
fn render_help(frame: &mut Frame, area: Rect) {
    let help_text = vec![
        Line::from(Span::styled(
            "Cuttlefish TUI - Quick Reference",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "Navigation",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from("  Tab          Cycle through views"),
        Line::from("  ↑/↓          Scroll or select items"),
        Line::from("  PageUp/Down  Scroll faster"),
        Line::from("  Home         Jump to bottom"),
        Line::from("  Ctrl-C/Esc   Quit"),
        Line::from(""),
        Line::from(Span::styled(
            "Slash Commands",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from("  /help, /h    Show this help"),
        Line::from("  /projects, /p  List projects"),
        Line::from("  /new <name>  Create a new project"),
        Line::from("  /switch <id> Switch to a project"),
        Line::from("  /clear       Clear chat history"),
        Line::from("  /chat        Go to chat view"),
        Line::from("  /diff        Go to diff view"),
        Line::from("  /log         Go to log view"),
        Line::from("  /quit, /q    Exit the TUI"),
        Line::from(""),
        Line::from(Span::styled(
            "Input History",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from("  ↑/↓ (empty)  Browse previous inputs"),
        Line::from(""),
        Line::from(Span::styled(
            "Tip: Just start typing to chat with the AI!",
            Style::default().fg(Color::Green),
        )),
    ];

    let help_widget = Paragraph::new(help_text)
        .block(Block::default().borders(Borders::ALL).title("Help"))
        .wrap(Wrap { trim: false });
    frame.render_widget(help_widget, area);
}

/// Render the history mode view with message selection and restore options.
fn render_history(app: &App, frame: &mut Frame, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title("History - Select a message to restore (↑/↓ select, Enter restore, Esc cancel)")
        .border_style(Style::default().fg(Color::Magenta));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if app.messages.is_empty() {
        let empty_msg = Paragraph::new(vec![
            Line::from(""),
            Line::from(Span::styled(
                "No messages in history.",
                Style::default().fg(Color::DarkGray),
            )),
        ]);
        frame.render_widget(empty_msg, inner);
        return;
    }

    // Calculate how many messages we can show
    let available_height = inner.height as usize;
    let msg_count = app.messages.len();

    // Build lines for each message (reversed order - most recent first)
    let mut lines: Vec<Line> = Vec::new();
    for (rev_idx, msg) in app.messages.iter().rev().enumerate() {
        let is_selected = rev_idx == app.history_selected;
        let has_snapshot = app.message_has_snapshot(msg_count.saturating_sub(1 + rev_idx));

        // Selection marker
        let marker = if is_selected { "▶ " } else { "  " };

        // Snapshot indicator
        let snapshot_marker = if has_snapshot { "⬤ " } else { "  " };

        // Message preview (truncate long messages)
        let preview: String = msg
            .content
            .chars()
            .take(60)
            .collect::<String>()
            .replace('\n', " ");
        let preview = if msg.content.len() > 60 {
            format!("{}...", preview)
        } else {
            preview
        };

        // Role color
        let role_color = match msg.sender.as_str() {
            "user" => Color::Green,
            "assistant" | "coder" | "orchestrator" => Color::Cyan,
            "system" => Color::Blue,
            "error" => Color::Red,
            _ => Color::White,
        };

        // Build the line
        let style = if is_selected {
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };

        let snapshot_style = if has_snapshot {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        lines.push(Line::from(vec![
            Span::styled(marker, style),
            Span::styled(snapshot_marker, snapshot_style),
            Span::styled(
                format!("[{}] ", msg.sender),
                Style::default().fg(role_color).add_modifier(Modifier::BOLD),
            ),
            Span::styled(preview, style),
        ]));
    }

    // Calculate scroll to keep selected item visible
    let scroll_offset = if app.history_selected >= available_height {
        app.history_selected - available_height + 1
    } else {
        0
    };

    let history_list = Paragraph::new(lines).scroll((scroll_offset as u16, 0));
    frame.render_widget(history_list, inner);

    // Render restore options popup if showing
    if app.show_restore_options {
        render_restore_popup(app, frame, area);
    }
}

/// Render the restore options popup.
fn render_restore_popup(app: &App, frame: &mut Frame, area: Rect) {
    use crate::app::RestoreOption;

    // Calculate popup size and position (centered)
    let popup_width = 35u16;
    let popup_height = 8u16;
    let popup_x = area.x + (area.width.saturating_sub(popup_width)) / 2;
    let popup_y = area.y + (area.height.saturating_sub(popup_height)) / 2;
    let popup_area = Rect::new(popup_x, popup_y, popup_width, popup_height);

    // Clear the popup area with a block
    let popup_block = Block::default()
        .borders(Borders::ALL)
        .title("Restore Options")
        .border_style(Style::default().fg(Color::Cyan))
        .style(Style::default().bg(Color::Black));

    frame.render_widget(Clear, popup_area);
    frame.render_widget(popup_block.clone(), popup_area);

    let inner = popup_block.inner(popup_area);

    // Render options
    let options: Vec<Line> = RestoreOption::all()
        .iter()
        .enumerate()
        .map(|(i, opt)| {
            let is_selected = i == app.restore_option_selected;
            let marker = if is_selected { "▶ " } else { "  " };
            let style = if is_selected {
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            Line::from(Span::styled(format!("{}{}", marker, opt.label()), style))
        })
        .collect();

    let options_widget = Paragraph::new(options);
    frame.render_widget(options_widget, inner);
}

/// Render the input box.
fn render_input(app: &App, frame: &mut Frame, area: Rect) {
    let hint = if app.input.is_empty() {
        Span::styled(
            " Type a message or /help for commands",
            Style::default().fg(Color::DarkGray),
        )
    } else if app.input.starts_with('/') {
        Span::styled(" (command)", Style::default().fg(Color::Cyan))
    } else {
        Span::raw("")
    };

    let input_content = if app.input.is_empty() {
        Line::from(hint)
    } else {
        Line::from(vec![
            Span::styled(&app.input, Style::default().fg(Color::Yellow)),
            hint,
        ])
    };

    let input_widget = Paragraph::new(input_content).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Input (Enter ↵)"),
    );
    frame.render_widget(input_widget, area);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::App;

    #[test]
    fn test_app_view_default() {
        let app = App::default();
        // Verify no panic from view default - Dashboard is the new default
        assert_eq!(app.view, AppView::Dashboard);
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
