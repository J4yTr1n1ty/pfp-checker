// ABOUTME: Command to add a Discord server to the monitoring list for tracking server icon changes
// ABOUTME: Stores server ID and tracking start timestamp in the Server table
use std::time::SystemTime;

use chrono::{DateTime, Utc};
use serenity::builder::*;
use serenity::model::prelude::*;
use serenity::prelude::*;
use sqlx::SqlitePool;

/// Handles the /monitorserver command to add a server to the monitoring list.
///
/// Requires MANAGE_GUILD permission. Checks if the server is already tracked
/// and adds it to the database with a timestamp if not.
///
/// # Arguments
/// * `ctx` - The Serenity context
/// * `interaction` - The command interaction
/// * `database` - SQLite connection pool
///
/// # Returns
/// * `Result<(), serenity::Error>` - Ok if successful, error otherwise
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

    // Check if user has permission to manage server
    if let Some(member) = &interaction.member {
        if let Some(permissions) = member.permissions {
            if !permissions.manage_guild() {
                interaction
                    .create_response(
                        &ctx.http,
                        CreateInteractionResponse::Message(
                            CreateInteractionResponseMessage::new().content(
                                "You need 'Manage Server' permission to use this command.",
                            ),
                        ),
                    )
                    .await?;
                return Ok(());
            }
        }
    }

    // Check if server is already being tracked
    let entry = sqlx::query!(
        "SELECT trackedSince FROM Server WHERE serverId = ? LIMIT 1",
        guild_id,
    )
    .fetch_one(database)
    .await;

    if let Ok(record) = entry {
        let guild_name = interaction
            .guild_id
            .and_then(|id| ctx.cache.guild(id))
            .map(|g| g.name.clone())
            .unwrap_or_else(|| "This server".to_string());

        if let Some(tracking_start_date) = record.trackedSince {
            if let Some(dt) = DateTime::from_timestamp(tracking_start_date, 0) {
                interaction
                    .create_response(
                        &ctx.http,
                        CreateInteractionResponse::Message(
                            CreateInteractionResponseMessage::new().content(format!(
                                "{} is already being tracked since <t:{}:F>",
                                guild_name,
                                dt.timestamp()
                            )),
                        ),
                    )
                    .await?;
                return Ok(());
            }
        }

        // If trackedSince is NULL or invalid, respond with generic message
        interaction
            .create_response(
                &ctx.http,
                CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new()
                        .content(format!("{} is already being tracked.", guild_name)),
                ),
            )
            .await?;
        return Ok(());
    }

    let now = SystemTime::now();
    let dt: DateTime<Utc> = now.into();
    let timestamp = dt.timestamp();

    let guild_name = interaction
        .guild_id
        .and_then(|id| ctx.cache.guild(id))
        .map(|g| g.name.clone())
        .unwrap_or_else(|| "This server".to_string());

    // Add the server to the database
    let insert_result = sqlx::query!(
        "INSERT INTO Server (serverId, trackedSince) VALUES (?, ?)",
        guild_id,
        timestamp
    )
    .execute(database)
    .await;

    if let Err(e) = insert_result {
        eprintln!("Failed to insert server {}: {:?}", guild_id, e);
        interaction
            .create_response(
                &ctx.http,
                CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new()
                        .content("Failed to add server to monitoring list. Please try again."),
                ),
            )
            .await?;
        return Ok(());
    }

    // Reply with confirmation
    interaction
        .create_response(
            &ctx.http,
            CreateInteractionResponse::Message(CreateInteractionResponseMessage::new().content(
                format!(
                    "Successfully added {} to the server monitoring list.",
                    guild_name
                ),
            )),
        )
        .await?;

    Ok(())
}

/// Registers the /monitorserver command with Discord.
///
/// # Returns
/// * `CreateCommand` - The command builder for registration
pub fn register() -> CreateCommand {
    CreateCommand::new("monitorserver")
        .description("Adds this server to the monitoring list to track server icon changes.")
}
