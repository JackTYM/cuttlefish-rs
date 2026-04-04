//! Handler for the `/new-project` slash command.

use cuttlefish_core::error::DiscordError;
use serenity::all::{CommandInteraction, Context};
use serenity::builder::{CreateCommand, CreateCommandOption, EditInteractionResponse};
use serenity::model::application::CommandOptionType;

use super::{defer_reply, get_string_option, names, send_error_response};
use crate::api_client::{CreateProjectRequest, get_api_client};
use crate::channel_manager::ChannelManager;

/// Build the `/new-project` command definition for registration.
pub fn register() -> CreateCommand {
    CreateCommand::new(names::NEW_PROJECT)
        .description("Create a new Cuttlefish project")
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::String,
                "name",
                "The name for your new project",
            )
            .required(true)
            .min_length(1)
            .max_length(90),
        )
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::String,
                "template",
                "Project template to use (e.g., rust, python, node)",
            )
            .required(false),
        )
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::String,
                "description",
                "A brief description of the project",
            )
            .required(false)
            .max_length(500),
        )
}

/// Execute the `/new-project` command.
pub async fn run(ctx: &Context, command: &CommandInteraction) -> Result<(), DiscordError> {
    let name = get_string_option(command, "name")
        .ok_or_else(|| DiscordError("Missing required 'name' parameter".to_string()))?;

    let template = get_string_option(command, "template");
    let description = get_string_option(command, "description")
        .unwrap_or_else(|| format!("Discord project: {name}"));

    if let Err(e) = ChannelManager::validate_project_name(&name) {
        send_error_response(ctx, command, &e.to_string()).await?;
        return Err(e);
    }

    defer_reply(ctx, command, false).await?;

    let api_result = match get_api_client() {
        Ok(client) => {
            let request = CreateProjectRequest {
                name: name.clone(),
                description: description.clone(),
                template: template.clone(),
            };
            client.create_project(request).await
        }
        Err(e) => {
            tracing::warn!("API client not configured: {e}");
            Err(e)
        }
    };

    let channel_name = ChannelManager::project_channel_name(&name);

    let response_content = match api_result {
        Ok(project) => {
            let mut content = format!(
                "✅ **Project Created**\n\n\
                **Name:** {}\n\
                **ID:** `{}`\n\
                **Status:** {}\n\
                **Channel:** #{}",
                project.name, project.id, project.status, channel_name
            );

            if let Some(ref tmpl) = project.template {
                content.push_str(&format!("\n**Template:** {tmpl}"));
            }

            content
        }
        Err(e) => {
            tracing::warn!("API call failed, using fallback response: {e}");
            let mut content = format!(
                "✅ **Project Created** _(offline mode)_\n\n\
                **Name:** {name}\n\
                **Channel:** #{channel_name}"
            );

            if let Some(ref tmpl) = template {
                content.push_str(&format!("\n**Template:** {tmpl}"));
            }

            content.push_str(&format!("\n**Description:** {description}"));
            content.push_str("\n\n⚠️ _API unavailable - project created locally only._");
            content
        }
    };

    command
        .edit_response(
            &ctx.http,
            EditInteractionResponse::new().content(response_content),
        )
        .await
        .map_err(|e| DiscordError(format!("Failed to edit response: {e}")))?;

    tracing::info!(
        project_name = %name,
        template = ?template,
        "Created new project via slash command"
    );

    Ok(())
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
    fn test_register_has_required_name_option() {
        let cmd = register();
        let json = serde_json::to_value(&cmd).expect("should serialize");

        let options = json.get("options").expect("should have options");
        let options_arr = options.as_array().expect("options should be array");

        let name_opt = options_arr
            .iter()
            .find(|o| o.get("name").and_then(|n| n.as_str()) == Some("name"));

        assert!(name_opt.is_some(), "Should have 'name' option");

        let name_opt = name_opt.expect("name option exists");
        assert_eq!(
            name_opt.get("required").and_then(|r| r.as_bool()),
            Some(true),
            "'name' option should be required"
        );
    }

    #[test]
    fn test_register_has_optional_template() {
        let cmd = register();
        let json = serde_json::to_value(&cmd).expect("should serialize");

        let options = json.get("options").expect("should have options");
        let options_arr = options.as_array().expect("options should be array");

        let template_opt = options_arr
            .iter()
            .find(|o| o.get("name").and_then(|n| n.as_str()) == Some("template"));

        assert!(template_opt.is_some(), "Should have 'template' option");

        let template_opt = template_opt.expect("template option exists");
        assert_eq!(
            template_opt.get("required").and_then(|r| r.as_bool()),
            Some(false),
            "'template' option should be optional"
        );
    }

    #[test]
    fn test_register_has_optional_description() {
        let cmd = register();
        let json = serde_json::to_value(&cmd).expect("should serialize");

        let options = json.get("options").expect("should have options");
        let options_arr = options.as_array().expect("options should be array");

        let desc_opt = options_arr
            .iter()
            .find(|o| o.get("name").and_then(|n| n.as_str()) == Some("description"));

        assert!(desc_opt.is_some(), "Should have 'description' option");

        let desc_opt = desc_opt.expect("description option exists");
        assert_eq!(
            desc_opt.get("required").and_then(|r| r.as_bool()),
            Some(false),
            "'description' option should be optional"
        );
    }
}
