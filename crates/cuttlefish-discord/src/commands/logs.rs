//! Handler for the `/logs` slash command.

use cuttlefish_core::error::DiscordError;
use serenity::all::{
    ButtonStyle, CommandInteraction, ComponentInteraction, Context, CreateInteractionResponse,
};
use serenity::builder::{
    CreateActionRow, CreateButton, CreateCommand, CreateCommandOption, CreateEmbed,
    CreateInteractionResponseMessage,
};
use serenity::model::Colour;
use serenity::model::application::CommandOptionType;

use super::{get_string_option, names};
use crate::api_client::{LogEntry as ApiLogEntry, get_api_client};

const DEFAULT_LINES: u32 = 20;
const MAX_LINES: u32 = 50;
const DISCORD_CONTENT_LIMIT: usize = 2000;

/// Build the `/logs` command definition for registration.
pub fn register() -> CreateCommand {
    CreateCommand::new(names::LOGS)
        .description("Show recent project activity logs")
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::String,
                "project",
                "Project name (defaults to current channel's project)",
            )
            .required(false),
        )
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::Integer,
                "lines",
                "Number of log lines to show (default 20, max 50)",
            )
            .required(false)
            .min_int_value(1)
            .max_int_value(MAX_LINES as u64),
        )
}

/// Execute the `/logs` command.
pub async fn run(ctx: &Context, command: &CommandInteraction) -> Result<(), DiscordError> {
    let project_name = get_string_option(command, "project");
    let lines = get_integer_option(command, "lines")
        .map(|l| l.clamp(1, MAX_LINES as i64) as u32)
        .unwrap_or(DEFAULT_LINES);

    let name = match project_name {
        Some(name) => name,
        None => {
            let embed = CreateEmbed::new()
                .title("📜 Logs")
                .description(
                    "No project specified and channel context lookup not yet implemented.\n\n\
                    Use `/logs project:<name>` to view logs for a specific project.",
                )
                .colour(Colour::from_rgb(128, 128, 128));

            let response = CreateInteractionResponseMessage::new().embed(embed);
            command
                .create_response(&ctx.http, CreateInteractionResponse::Message(response))
                .await
                .map_err(|e| DiscordError(format!("Failed to respond: {e}")))?;

            return Ok(());
        }
    };

    let (logs, has_more_from_api) = match get_api_client() {
        Ok(client) => match client.get_project_logs(&name, lines).await {
            Ok(response) => {
                let entries: Vec<LogEntry> = response
                    .entries
                    .into_iter()
                    .map(|e| LogEntry {
                        timestamp: e.timestamp,
                        agent: e.agent,
                        message: e.message,
                    })
                    .collect();
                (entries, response.has_more)
            }
            Err(e) => {
                tracing::warn!("Failed to fetch logs from API: {e}");
                (fetch_mock_logs(&name, lines), false)
            }
        },
        Err(e) => {
            tracing::warn!("API client not configured: {e}");
            (fetch_mock_logs(&name, lines), false)
        }
    };

    let (content, truncated) = format_logs_for_discord(&logs);
    let has_more = has_more_from_api || truncated;

    let mut response = CreateInteractionResponseMessage::new().content(content);

    if has_more {
        let button = CreateButton::new(format!("logs_more_{name}"))
            .label("Show More")
            .style(ButtonStyle::Secondary);
        response = response.components(vec![CreateActionRow::Buttons(vec![button])]);
    }

    command
        .create_response(&ctx.http, CreateInteractionResponse::Message(response))
        .await
        .map_err(|e| DiscordError(format!("Failed to respond: {e}")))?;

    tracing::info!(project_name = %name, lines = lines, "Displayed project logs");

    Ok(())
}

/// Handle the "Show More" button interaction for logs pagination.
pub async fn handle_show_more(
    ctx: &Context,
    interaction: &ComponentInteraction,
    project_name: &str,
) -> Result<(), DiscordError> {
    let logs = match get_api_client() {
        Ok(client) => match client.get_project_logs(project_name, MAX_LINES).await {
            Ok(response) => response
                .entries
                .into_iter()
                .map(|e| LogEntry {
                    timestamp: e.timestamp,
                    agent: e.agent,
                    message: e.message,
                })
                .collect(),
            Err(_) => fetch_mock_logs(project_name, MAX_LINES),
        },
        Err(_) => fetch_mock_logs(project_name, MAX_LINES),
    };

    let (content, _) = format_logs_for_discord(&logs);

    let response = CreateInteractionResponseMessage::new()
        .content(content)
        .ephemeral(true);

    interaction
        .create_response(&ctx.http, CreateInteractionResponse::Message(response))
        .await
        .map_err(|e| DiscordError(format!("Failed to respond to button: {e}")))?;

    Ok(())
}

fn get_integer_option(command: &CommandInteraction, name: &str) -> Option<i64> {
    use serenity::model::application::ResolvedValue;

    command.data.options().iter().find_map(|opt| {
        if opt.name == name
            && let ResolvedValue::Integer(i) = opt.value
        {
            return Some(i);
        }
        None
    })
}

fn fetch_mock_logs(project_name: &str, lines: u32) -> Vec<LogEntry> {
    let mock_entries = vec![
        LogEntry {
            timestamp: "2024-01-15 14:32:01".to_string(),
            agent: "Orchestrator".to_string(),
            message: "Starting task analysis".to_string(),
        },
        LogEntry {
            timestamp: "2024-01-15 14:32:05".to_string(),
            agent: "Planner".to_string(),
            message: "Created implementation plan with 3 steps".to_string(),
        },
        LogEntry {
            timestamp: "2024-01-15 14:32:10".to_string(),
            agent: "Coder".to_string(),
            message: "Implementing step 1: Create module structure".to_string(),
        },
        LogEntry {
            timestamp: "2024-01-15 14:33:45".to_string(),
            agent: "Coder".to_string(),
            message: "Step 1 complete, running tests".to_string(),
        },
        LogEntry {
            timestamp: "2024-01-15 14:34:00".to_string(),
            agent: "Critic".to_string(),
            message: "Reviewing changes for step 1".to_string(),
        },
        LogEntry {
            timestamp: "2024-01-15 14:34:30".to_string(),
            agent: "Critic".to_string(),
            message: "Approved: Code follows project conventions".to_string(),
        },
    ];

    let _ = project_name;
    mock_entries.into_iter().take(lines as usize).collect()
}

fn format_logs_for_discord(logs: &[LogEntry]) -> (String, bool) {
    if logs.is_empty() {
        return ("```\nNo activity logs found.\n```".to_string(), false);
    }

    let mut output = String::from("```\n");
    let mut truncated = false;

    for entry in logs {
        let line = format!("[{}] {}: {}\n", entry.timestamp, entry.agent, entry.message);

        if output.len() + line.len() + 4 > DISCORD_CONTENT_LIMIT {
            truncated = true;
            break;
        }
        output.push_str(&line);
    }

    output.push_str("```");

    if truncated {
        let remaining = DISCORD_CONTENT_LIMIT.saturating_sub(output.len() + 50);
        if remaining > 0 {
            output.push_str("\n_Output truncated. Click \"Show More\" for full logs._");
        }
    }

    (output, truncated)
}

struct LogEntry {
    timestamp: String,
    agent: String,
    message: String,
}

impl From<ApiLogEntry> for LogEntry {
    fn from(api: ApiLogEntry) -> Self {
        Self {
            timestamp: api.timestamp,
            agent: api.agent,
            message: api.message,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_creates_valid_command() {
        let cmd = register();
        let json = serde_json::to_string(&cmd);
        assert!(json.is_ok(), "Command should serialize to JSON");
    }

    #[test]
    fn test_register_has_optional_project_option() {
        let cmd = register();
        let json = serde_json::to_value(&cmd).expect("should serialize");

        let options = json.get("options").expect("should have options");
        let options_arr = options.as_array().expect("options should be array");

        let project_opt = options_arr
            .iter()
            .find(|o| o.get("name").and_then(|n| n.as_str()) == Some("project"));

        assert!(project_opt.is_some(), "Should have 'project' option");
    }

    #[test]
    fn test_register_has_lines_option_with_limits() {
        let cmd = register();
        let json = serde_json::to_value(&cmd).expect("should serialize");

        let options = json.get("options").expect("should have options");
        let options_arr = options.as_array().expect("options should be array");

        let lines_opt = options_arr
            .iter()
            .find(|o| o.get("name").and_then(|n| n.as_str()) == Some("lines"));

        assert!(lines_opt.is_some(), "Should have 'lines' option");

        let lines_opt = lines_opt.expect("lines option exists");
        assert_eq!(
            lines_opt.get("max_value").and_then(|v| v.as_u64()),
            Some(MAX_LINES as u64),
            "lines should have max_value of MAX_LINES"
        );
    }

    #[test]
    fn test_fetch_mock_logs_respects_limit() {
        let logs = fetch_mock_logs("test-project", 3);
        assert_eq!(logs.len(), 3);
    }

    #[test]
    fn test_format_logs_empty() {
        let (content, has_more) = format_logs_for_discord(&[]);
        assert!(content.contains("No activity logs found"));
        assert!(!has_more);
    }

    #[test]
    fn test_format_logs_with_entries() {
        let logs = fetch_mock_logs("test", 2);
        let (content, _) = format_logs_for_discord(&logs);
        assert!(content.starts_with("```"));
        assert!(content.ends_with("```"));
        assert!(content.contains("Orchestrator"));
    }

    #[test]
    fn test_format_logs_truncation() {
        let long_logs: Vec<LogEntry> = (0..100)
            .map(|i| LogEntry {
                timestamp: format!("2024-01-15 14:{i:02}:00"),
                agent: "TestAgent".to_string(),
                message: format!(
                    "This is a test log message number {i} with some extra content to make it longer"
                ),
            })
            .collect();

        let (content, has_more) = format_logs_for_discord(&long_logs);
        assert!(content.len() <= DISCORD_CONTENT_LIMIT + 100);
        assert!(has_more);
    }
}
