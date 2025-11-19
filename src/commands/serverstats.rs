// ABOUTME: Command to display statistics about server icon changes
// ABOUTME: Shows average time between icon changes and total change count
use chrono::{DateTime, Duration, Utc};
use serenity::all::{
    CommandInteraction, Context, CreateEmbed, CreateEmbedAuthor, CreateEmbedFooter,
    CreateInteractionResponse, CreateInteractionResponseMessage,
};
use serenity::builder::CreateCommand;
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

    let guild_name = interaction
        .guild_id
        .and_then(|id| ctx.cache.guild(id))
        .map(|g| g.name.clone())
        .unwrap_or_else(|| "This server".to_string());

    let server_entry = sqlx::query!("SELECT * FROM Server WHERE serverId = ?", guild_id)
        .fetch_one(database)
        .await;

    match server_entry {
        Ok(record) => {
            let icons = sqlx::query!(
                "SELECT * FROM ServerPicture WHERE serverId = ? ORDER BY changedAt DESC",
                record.serverId
            )
            .fetch_all(database)
            .await;

            match icons {
                Ok(entries) => {
                    let mut total_duration = Duration::zero(); // Total duration of all changes
                    let mut count = 0; // Number of server icon changes

                    for entry in &entries {
                        // Convert the i64 timestamp to DateTime<Utc>
                        let dt = DateTime::<Utc>::from_timestamp(entry.changedAt.unwrap(), 0)
                            .expect("Invalid timestamp");

                        if count > 0 {
                            // Calculate the duration between the current and previous entry
                            let prev_entry = &entries[count - 1];
                            // Use expect to handle the Option, providing a reason for potential panics
                            let prev_dt =
                                DateTime::<Utc>::from_timestamp(prev_entry.changedAt.unwrap(), 0)
                                    .expect("Invalid timestamp");
                            let duration = dt.signed_duration_since(prev_dt);
                            total_duration += duration;
                        }
                        count += 1;
                    }

                    if count > 1 {
                        // Ensure there's more than one entry to calculate an average
                        let average_duration = total_duration.num_seconds() as f64 / count as f64;
                        // Convert the average duration to seconds for display
                        let average_duration_in_seconds = average_duration.round() as i32; // Round to the nearest whole number

                        let average_duration_in_hours = average_duration_in_seconds / 3600;
                        let average_duration_in_days = average_duration_in_hours / 24;

                        let dt = DateTime::from_timestamp(record.trackedSince.unwrap(), 0).unwrap();
                        let embed_author = CreateEmbedAuthor::new(guild_name.clone());
                        let embed_footer =
                            CreateEmbedFooter::new(format!("Monitored since {}", dt.to_rfc2822()));

                        let embed = CreateEmbed::new()
                            .title("Average times between server icon changes:")
                            .author(embed_author)
                            .footer(embed_footer)
                            .fields(vec![
                                ("Hours", format!("{average_duration_in_hours}"), true),
                                ("Days", format!("{average_duration_in_days}"), true),
                                ("", "".to_string(), false), // empty row for spacing
                                (
                                    "Changes since beginning of Monitoring",
                                    format!("{count}"),
                                    false,
                                ),
                            ]);

                        // Respond with the average time
                        interaction
                            .create_response(
                                &ctx,
                                CreateInteractionResponse::Message(
                                    CreateInteractionResponseMessage::new().embed(embed),
                                ),
                            )
                            .await
                            .unwrap();
                    } else {
                        // Respond if there's not enough data to calculate an average
                        interaction
                            .create_response(
                                &ctx,
                                CreateInteractionResponse::Message(
                                    CreateInteractionResponseMessage::new().content(
                                        "Not enough data to calculate average server icon changes.",
                                    ),
                                ),
                            )
                            .await
                            .unwrap();
                    }
                }
                Err(_) => {
                    interaction
                        .create_response(
                            &ctx,
                            CreateInteractionResponse::Message(
                                CreateInteractionResponseMessage::new().content(
                                    "No server icons have been recorded. Please wait at least 30 minutes and check again.",
                                ),
                            ),
                        )
                        .await
                        .unwrap();
                }
            }
        }
        Err(_) => {
            interaction
                .create_response(
                    &ctx,
                    CreateInteractionResponse::Message(
                        CreateInteractionResponseMessage::new().content(format!(
                            "{} is not currently being tracked. You can add it using /monitorserver",
                            guild_name
                        )),
                    ),
                )
                .await
                .unwrap();
        }
    }
    Ok(())
}

pub fn register() -> CreateCommand {
    CreateCommand::new("serverstats")
        .description("Shows statistics for server icon changes in this server.")
}
