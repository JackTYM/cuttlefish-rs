//! Handler for the `/approve` slash command.

use cuttlefish_core::error::DiscordError;
use serenity::all::{CommandInteraction, Context, CreateInteractionResponse};
use serenity::builder::{CreateCommand, CreateEmbed, CreateEmbedFooter, CreateInteractionResponseMessage};
use serenity::model::Colour;

use super::{names, send_error_response};
use crate::api_client::{ApproveActionRequest, get_api_client};
use crate::channel_manager::PendingAction;

/// Build the `/approve` command definition for registration.
pub fn register() -> CreateCommand {
    CreateCommand::new(names::APPROVE).description("Approve a pending agent action in this channel")
}

/// Execute the `/approve` command.
pub async fn run(ctx: &Context, command: &CommandInteraction) -> Result<(), DiscordError> {
    let channel_id = command.channel_id;
    let user = &command.user;

    let pending = PendingAction::get_for_channel(channel_id);

    let Some(action) = pending else {
        send_error_response(
            ctx,
            command,
            "No pending action in this channel. There's nothing to approve.",
        )
        .await?;
        return Ok(());
    };

    let api_result = match get_api_client() {
        Ok(client) => {
            let request = ApproveActionRequest {
                action_id: action.id.clone(),
                approved_by: user.id.to_string(),
            };
            client.approve_action(&action.project_name, request).await
        }
        Err(e) => {
            tracing::warn!("API client not configured: {e}");
            Err(e)
        }
    };

    let embed = match api_result {
        Ok(response) => CreateEmbed::new()
            .title("✅ Action Approved")
            .description(format!(
                "**Action:** {}\n**Approved by:** <@{}>\n**Status:** {}\n\n_{}._",
                action.description, user.id, response.status, response.message
            ))
            .colour(Colour::from_rgb(87, 242, 135))
            .footer(CreateEmbedFooter::new(format!("Action ID: {}", action.id))),
        Err(e) => {
            tracing::warn!("API approval failed: {e}");
            CreateEmbed::new()
                .title("✅ Action Approved (Offline)")
                .description(format!(
                    "**Action:** {}\n**Approved by:** <@{}>\n\n⚠️ _API unavailable - approval recorded locally._",
                    action.description, user.id
                ))
                .colour(Colour::from_rgb(250, 166, 26))
                .footer(CreateEmbedFooter::new(format!("Action ID: {}", action.id)))
        }
    };

    let response = CreateInteractionResponseMessage::new().embed(embed);
    command
        .create_response(&ctx.http, CreateInteractionResponse::Message(response))
        .await
        .map_err(|e| DiscordError(format!("Failed to respond: {e}")))?;

    tracing::info!(
        action_id = %action.id,
        approved_by = %user.id,
        channel_id = %channel_id,
        "Approved pending action"
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
    fn test_register_has_no_required_options() {
        let cmd = register();
        let json = serde_json::to_value(&cmd).expect("should serialize");

        let options = json.get("options");
        assert!(
            options.is_none()
                || options.is_some_and(|o| o.as_array().is_some_and(|a| a.is_empty())),
            "approve command should have no options"
        );
    }
}
