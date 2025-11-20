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
                    let change_count = entries.len();

                    if change_count > 1 {
                        // Total duration between consecutive icon changes (newest → oldest)
                        let mut total_duration = Duration::zero();

                        for window in entries.windows(2) {
                            let newer = &window[0];
                            let older = &window[1];

                            let newer_dt =
                                DateTime::<Utc>::from_timestamp(newer.changedAt.unwrap(), 0)
                                    .expect("Invalid timestamp");
                            let older_dt =
                                DateTime::<Utc>::from_timestamp(older.changedAt.unwrap(), 0)
                                    .expect("Invalid timestamp");

                            let duration = newer_dt.signed_duration_since(older_dt);
                            total_duration += duration;
                        }

                        let intervals = (change_count - 1) as f64;
                        let average_duration = total_duration.num_seconds() as f64 / intervals;
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
                                    format!("{change_count}"),
                                    false,
                                ),
                            ]);

                        // Respond with the average time
                        interaction
                            .create_response(
                                &ctx.http,
                                CreateInteractionResponse::Message(
                                    CreateInteractionResponseMessage::new().embed(embed),
                                ),
                            )
                            .await?;
                    } else {
                        // Respond if there's not enough data to calculate an average
                        interaction
                            .create_response(
                                &ctx.http,
                                CreateInteractionResponse::Message(
                                    CreateInteractionResponseMessage::new().content(
                                        "Not enough data to calculate average server icon changes.",
                                    ),
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
                                CreateInteractionResponseMessage::new().content(
                                    "No server icons have been recorded. Please wait at least 30 minutes and check again.",
                                ),
                            ),
                        )
                        .await?;
                }
            }
        }
        Err(_) => {
            interaction
                .create_response(
                    &ctx.http,
                    CreateInteractionResponse::Message(
                        CreateInteractionResponseMessage::new().content(format!(
                            "{} is not currently being tracked. You can add it using /monitorserver",
                            guild_name
                        )),
                    ),
                )
                .await?;
        }
    }
    Ok(())
}

pub fn register() -> CreateCommand {
    CreateCommand::new("serverstats")
        .description("Shows statistics for server icon changes in this server.")
}
