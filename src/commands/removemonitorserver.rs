// ABOUTME: Command to remove a Discord server from the monitoring list
// ABOUTME: Deletes the server entry and all associated server icon history from the database
use serenity::builder::*;
use serenity::model::prelude::*;
use serenity::prelude::*;

use sqlx::SqlitePool;

pub async fn run(
    ctx: &Context,
    interaction: &CommandInteraction,
    database: &SqlitePool,
) -> Result<(), serenity::Error> {
    // Get the guild (server) from the interaction
    let guild_id = match interaction.guild_id {
        Some(id) => i64::from(id),
        None => {
            interaction
                .create_response(
                    &ctx.http,
                    CreateInteractionResponse::Message(
                        CreateInteractionResponseMessage::new()
                            .content("This command can only be used in a server."),
                    ),
                )
                .await?;
            return Ok(());
        }
    };

    let delete_result = sqlx::query!("DELETE FROM Server WHERE serverId = ?", guild_id)
        .execute(database)
        .await;

    match delete_result {
        Ok(res) => {
            if res.rows_affected() > 0 {
                let guild_name = interaction
                    .guild_id
                    .and_then(|id| ctx.cache.guild(id))
                    .map(|g| g.name.clone())
                    .unwrap_or_else(|| "Server".to_string());

                interaction
                    .create_response(
                        &ctx.http,
                        CreateInteractionResponse::Message(
                            CreateInteractionResponseMessage::new()
                                .content(format!("Successfully removed {} from monitoring.", guild_name)),
                        ),
                    )
                    .await?;
            } else {
                interaction
                    .create_response(
                        &ctx.http,
                        CreateInteractionResponse::Message(
                            CreateInteractionResponseMessage::new()
                                .content("Unable to find server. Server may not be tracked."),
                        ),
                    )
                    .await?;
            }
        }
        Err(_) => {
            interaction
                .create_response(
                    &ctx.http,
                    CreateInteractionResponse::Message(
                        CreateInteractionResponseMessage::new()
                            .content("Unable to delete server. Server may not be tracked."),
                    ),
                )
                .await?;
        }
    }

    Ok(())
}

pub fn register() -> CreateCommand {
    CreateCommand::new("removemonitorserver")
        .description("Removes this server from the monitoring list.")
}
