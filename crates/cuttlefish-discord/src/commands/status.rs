//! Handler for the `/status` slash command.

use cuttlefish_core::error::DiscordError;
use serenity::all::{CommandInteraction, Context, CreateInteractionResponse};
use serenity::builder::{
    CreateCommand, CreateCommandOption, CreateEmbed, CreateEmbedFooter,
    CreateInteractionResponseMessage,
};
use serenity::model::application::CommandOptionType;
use serenity::model::Colour;

use super::{get_string_option, names};
use crate::api_client::{AgentInfo, ProjectStatus, get_api_client};

/// Build the `/status` command definition for registration.
pub fn register() -> CreateCommand {
    CreateCommand::new(names::STATUS)
        .description("Show project or agent status")
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::String,
                "project",
                "Project name (defaults to current channel's project)",
            )
            .required(false),
        )
}

/// Execute the `/status` command.
pub async fn run(ctx: &Context, command: &CommandInteraction) -> Result<(), DiscordError> {
    let project_name = get_string_option(command, "project");

    let name = match project_name {
        Some(name) => name,
        None => {
            let embed = CreateEmbed::new()
                .title("📊 Status")
                .description(
                    "No project specified and channel context lookup not yet implemented.\n\n\
                    Use `/status project:<name>` to check a specific project.",
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

    let embed = match get_api_client() {
        Ok(client) => match client.get_project_by_name(&name).await {
            Ok(status) => build_status_embed_from_api(&status),
            Err(e) => {
                tracing::warn!("Failed to fetch project status: {e}");
                build_error_embed(&name, &e.to_string())
            }
        },
        Err(e) => {
            tracing::warn!("API client not configured: {e}");
            build_fallback_embed(&name)
        }
    };

    let response = CreateInteractionResponseMessage::new().embed(embed);
    command
        .create_response(&ctx.http, CreateInteractionResponse::Message(response))
        .await
        .map_err(|e| DiscordError(format!("Failed to respond: {e}")))?;

    tracing::info!(project_name = %name, "Displayed project status");

    Ok(())
}

fn build_status_embed_from_api(status: &ProjectStatus) -> CreateEmbed {
    let status_emoji = match status.status.as_str() {
        "active" => "🟢",
        "paused" => "🟡",
        "completed" => "✅",
        "error" => "🔴",
        _ => "⚪",
    };

    let agent_count = status.active_agents.len();
    let agent_text = if agent_count == 0 {
        "None".to_string()
    } else {
        format_agents(&status.active_agents)
    };

    let last_activity = status
        .last_activity
        .as_deref()
        .unwrap_or("N/A");

    let current_task = status
        .current_task
        .as_deref()
        .unwrap_or("_No active task_");

    CreateEmbed::new()
        .title(format!("📊 Project: {}", status.name))
        .colour(status_colour(&status.status))
        .field("Status", format!("{status_emoji} {}", status.status), true)
        .field("Active Agents", agent_count.to_string(), true)
        .field("Last Activity", last_activity, true)
        .field("Current Task", current_task, false)
        .field("Agents", agent_text, false)
        .footer(CreateEmbedFooter::new(format!("Project ID: {}", status.id)))
}

fn build_fallback_embed(project_name: &str) -> CreateEmbed {
    CreateEmbed::new()
        .title(format!("📊 Project: {project_name}"))
        .colour(Colour::from_rgb(128, 128, 128))
        .field("Status", "⚪ Unknown", true)
        .field("Active Agents", "N/A", true)
        .field("Last Activity", "N/A", true)
        .field(
            "Note",
            "⚠️ API unavailable - cannot fetch real status.",
            false,
        )
        .footer(CreateEmbedFooter::new("Use /logs to see recent activity"))
}

fn build_error_embed(project_name: &str, error: &str) -> CreateEmbed {
    CreateEmbed::new()
        .title(format!("📊 Project: {project_name}"))
        .colour(Colour::from_rgb(237, 66, 69))
        .description(format!("❌ Failed to fetch status: {error}"))
        .footer(CreateEmbedFooter::new("Check project name and try again"))
}

fn format_agents(agents: &[AgentInfo]) -> String {
    if agents.is_empty() {
        return "None".to_string();
    }

    agents
        .iter()
        .map(|a| {
            let action = a.current_action.as_deref().unwrap_or("idle");
            format!("• **{}** ({}): {}", a.name, a.status, action)
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn status_colour(status: &str) -> Colour {
    match status {
        "active" => Colour::from_rgb(87, 242, 135),
        "paused" => Colour::from_rgb(250, 166, 26),
        "completed" => Colour::from_rgb(88, 101, 242),
        "error" => Colour::from_rgb(237, 66, 69),
        _ => Colour::from_rgb(128, 128, 128),
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

        let project_opt = project_opt.expect("project option exists");
        assert_eq!(
            project_opt.get("required").and_then(|r| r.as_bool()),
            Some(false),
            "'project' option should be optional"
        );
    }

    #[test]
    fn test_status_colour_active() {
        let colour = status_colour("active");
        assert_eq!(colour, Colour::from_rgb(87, 242, 135));
    }

    #[test]
    fn test_status_colour_error() {
        let colour = status_colour("error");
        assert_eq!(colour, Colour::from_rgb(237, 66, 69));
    }

    #[test]
    fn test_status_colour_unknown() {
        let colour = status_colour("unknown");
        assert_eq!(colour, Colour::from_rgb(128, 128, 128));
    }

    #[test]
    fn test_format_agents_empty() {
        let agents: Vec<AgentInfo> = vec![];
        assert_eq!(format_agents(&agents), "None");
    }

    #[test]
    fn test_format_agents_with_entries() {
        let agents = vec![
            AgentInfo {
                name: "Coder".to_string(),
                status: "working".to_string(),
                current_action: Some("Writing code".to_string()),
            },
            AgentInfo {
                name: "Critic".to_string(),
                status: "idle".to_string(),
                current_action: None,
            },
        ];
        let formatted = format_agents(&agents);
        assert!(formatted.contains("Coder"));
        assert!(formatted.contains("Writing code"));
        assert!(formatted.contains("Critic"));
        assert!(formatted.contains("idle"));
    }

    #[test]
    fn test_build_fallback_embed_contains_project_name() {
        let embed = build_fallback_embed("test-project");
        let json = serde_json::to_value(&embed).expect("should serialize");

        let title = json.get("title").and_then(|t| t.as_str());
        assert!(
            title.is_some_and(|t| t.contains("test-project")),
            "Embed title should contain project name"
        );
    }

    #[test]
    fn test_build_error_embed_contains_error() {
        let embed = build_error_embed("my-project", "Connection refused");
        let json = serde_json::to_value(&embed).expect("should serialize");

        let desc = json.get("description").and_then(|d| d.as_str());
        assert!(
            desc.is_some_and(|d| d.contains("Connection refused")),
            "Embed should contain error message"
        );
    }
}
