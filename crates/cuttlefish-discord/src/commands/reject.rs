//! Handler for the `/reject` slash command.

use cuttlefish_core::error::DiscordError;
use serenity::all::{CommandInteraction, Context, CreateInteractionResponse};
use serenity::builder::{
    CreateCommand, CreateCommandOption, CreateEmbed, CreateEmbedFooter,
    CreateInteractionResponseMessage,
};
use serenity::model::Colour;
use serenity::model::application::CommandOptionType;

use super::{get_string_option, names, send_error_response};
use crate::api_client::{RejectActionRequest, get_api_client};
use crate::channel_manager::PendingAction;

/// Build the `/reject` command definition for registration.
pub fn register() -> CreateCommand {
    CreateCommand::new(names::REJECT)
        .description("Reject a pending agent action with optional feedback")
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::String,
                "reason",
                "Reason for rejection (feedback for the agent)",
            )
            .required(false)
            .max_length(500),
        )
}

/// Execute the `/reject` command.
pub async fn run(ctx: &Context, command: &CommandInteraction) -> Result<(), DiscordError> {
    let channel_id = command.channel_id;
    let user = &command.user;
    let reason = get_string_option(command, "reason");

    let pending = PendingAction::get_for_channel(channel_id);

    let Some(action) = pending else {
        send_error_response(
            ctx,
            command,
            "No pending action in this channel. There's nothing to reject.",
        )
        .await?;
        return Ok(());
    };

    let reason_text = reason.as_deref().unwrap_or("No reason provided");

    let api_result = match get_api_client() {
        Ok(client) => {
            let request = RejectActionRequest {
                action_id: action.id.clone(),
                rejected_by: user.id.to_string(),
                reason: reason.clone(),
            };
            client.reject_action(&action.project_name, request).await
        }
        Err(e) => {
            tracing::warn!("API client not configured: {e}");
            Err(e)
        }
    };

    let embed = match api_result {
        Ok(response) => CreateEmbed::new()
            .title("❌ Action Rejected")
            .description(format!(
                "**Action:** {}\n**Rejected by:** <@{}>\n**Reason:** {}\n**Status:** {}\n\n_{}._",
                action.description, user.id, reason_text, response.status, response.message
            ))
            .colour(Colour::from_rgb(237, 66, 69))
            .footer(CreateEmbedFooter::new(format!("Action ID: {}", action.id))),
        Err(e) => {
            tracing::warn!("API rejection failed: {e}");
            CreateEmbed::new()
                .title("❌ Action Rejected (Offline)")
                .description(format!(
                    "**Action:** {}\n**Rejected by:** <@{}>\n**Reason:** {}\n\n⚠️ _API unavailable - rejection recorded locally._",
                    action.description, user.id, reason_text
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
        rejected_by = %user.id,
        channel_id = %channel_id,
        reason = ?reason,
        "Rejected pending action"
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
    fn test_register_has_optional_reason() {
        let cmd = register();
        let json = serde_json::to_value(&cmd).expect("should serialize");

        let options = json.get("options").expect("should have options");
        let options_arr = options.as_array().expect("options should be array");

        let reason_opt = options_arr
            .iter()
            .find(|o| o.get("name").and_then(|n| n.as_str()) == Some("reason"));

        assert!(reason_opt.is_some(), "Should have 'reason' option");

        let reason_opt = reason_opt.expect("reason option exists");
        assert_eq!(
            reason_opt.get("required").and_then(|r| r.as_bool()),
            Some(false),
            "'reason' option should be optional"
        );
    }
}
