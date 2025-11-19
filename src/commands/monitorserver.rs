// ABOUTME: Command to add a Discord server to the monitoring list for tracking server icon changes
// ABOUTME: Stores server ID and tracking start timestamp in the Server table
use std::time::SystemTime;

use chrono::{DateTime, Utc};
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

    // Check if server is already being tracked
    let entry = sqlx::query!(
        "SELECT trackedSince FROM Server WHERE serverId = ? LIMIT 1",
        guild_id,
    )
    .fetch_one(database)
    .await;

    if let Ok(record) = entry {
        let tracking_start_date = record.trackedSince.unwrap();
        let dt = DateTime::from_timestamp(tracking_start_date, 0).unwrap();

        let guild_name = interaction
            .guild_id
            .and_then(|id| ctx.cache.guild(id))
            .map(|g| g.name.clone())
            .unwrap_or_else(|| "This server".to_string());

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

    let now = SystemTime::now();
    let dt: DateTime<Utc> = now.into();
    let timestamp = dt.timestamp();

    // Add the server to the database
    sqlx::query!(
        "INSERT INTO Server (serverId, trackedSince) VALUES (?, ?)",
        guild_id,
        timestamp
    )
    .execute(database)
    .await
    .unwrap();

    let guild_name = interaction
        .guild_id
        .and_then(|id| ctx.cache.guild(id))
        .map(|g| g.name.clone())
        .unwrap_or_else(|| "This server".to_string());

    // Reply with confirmation
    interaction
        .create_response(
            &ctx.http,
            CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new().content(format!(
                    "Successfully added {} to the server monitoring list.",
                    guild_name
                )),
            ),
        )
        .await?;

    Ok(())
}

pub fn register() -> CreateCommand {
    CreateCommand::new("monitorserver")
        .description("Adds this server to the monitoring list to track server icon changes.")
}
