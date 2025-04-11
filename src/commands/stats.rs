use chrono::{DateTime, Duration, Utc};
use serenity::all::{
    CommandInteraction, Context, CreateCommandOption, CreateEmbed, CreateEmbedAuthor,
    CreateEmbedFooter, CreateInteractionResponse, CreateInteractionResponseMessage, ResolvedValue,
};
use serenity::builder::CreateCommand;
use serenity::model::application::ResolvedOption;
use sqlx::SqlitePool;

pub async fn run(
    ctx: &Context,
    interaction: &CommandInteraction,
    database: &SqlitePool,
    options: &[ResolvedOption<'_>],
) -> Result<(), serenity::Error> {
    if let Some(ResolvedOption {
        value: ResolvedValue::User(user, _),
        ..
    }) = options.first()
    {
        let user_id = i64::from(user.id); // Need to cast until I figure out how to implement the
                                          // trait for sqlx.

        let user_entry = sqlx::query!("SELECT * FROM User WHERE discordId = ?", user_id)
            .fetch_one(database)
            .await;

        match user_entry {
            Ok(record) => {
                let pfps = sqlx::query!(
                    "SELECT * FROM ProfilePicture WHERE userId = ?",
                    record.discordId
                )
                .fetch_all(database)
                .await;

                match pfps {
                    Ok(entries) => {
                        let mut total_duration = Duration::zero(); // Total duration of all changes
                        let mut count = 0; // Number of profile picture changes

                        for entry in &entries {
                            // Convert the i64 timestamp to DateTime<Utc>
                            let dt = DateTime::<Utc>::from_timestamp(entry.changedAt.unwrap(), 0)
                                .expect("Invalid timestamp");

                            if count > 0 {
                                // Calculate the duration between the current and previous entry
                                let prev_entry = &entries[count - 1];
                                // Use expect to handle the Option, providing a reason for potential panics
                                let prev_dt = DateTime::<Utc>::from_timestamp(
                                    prev_entry.changedAt.unwrap(),
                                    0,
                                )
                                .expect("Invalid timestamp");
                                let duration = dt.signed_duration_since(prev_dt);
                                total_duration += duration;
                            }
                            count += 1;
                        }

                        if count > 1 {
                            // Ensure there's more than one entry to calculate an average
                            let average_duration =
                                total_duration.num_seconds() as f64 / count as f64;
                            // Convert the average duration to seconds for display
                            let average_duration_in_seconds = average_duration.round() as i32; // Round to the nearest whole number

                            let average_duration_in_hours = average_duration_in_seconds / 3600;
                            let average_duration_in_days = average_duration_in_hours / 24;

                            let dt =
                                DateTime::from_timestamp(record.trackedSince.unwrap(), 0).unwrap();
                            let embed_author = CreateEmbedAuthor::new(user.tag().to_string());
                            let embed_footer = CreateEmbedFooter::new(format!(
                                "Monitored since {}",
                                dt.to_rfc2822()
                            ));

                            let embed = CreateEmbed::new()
                                .title("Average times between profile picture changes:")
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
                            interaction.create_response(
                                &ctx,
                                CreateInteractionResponse::Message(
                                    CreateInteractionResponseMessage::new()
                                   .content("Not enough data to calculate an average time between profile picture changes.")
                                )
                            ).await.unwrap();
                        }
                    }
                    Err(_) => {
                        interaction
                            .create_response(
                                &ctx,
                                CreateInteractionResponse::Message(
                                    CreateInteractionResponseMessage::new()
                                        .content("No Profile Pictures have been recorded for this User. Please wait at least 30 minutes and check again.")))
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
                            CreateInteractionResponseMessage::new()
                                .content("User is not currently being tracked. You an add the user to the monitor list by using /monitor @user")))
                    .await
                    .unwrap();
            }
        }
    }
    Ok(())
}

pub fn register() -> CreateCommand {
    CreateCommand::new("stats")
        .description("Shows Statistics of a User.")
        .add_option(
            CreateCommandOption::new(
                serenity::all::CommandOptionType::User,
                "member",
                "The member whose Statistics to show.",
            )
            .required(true),
        )
}
