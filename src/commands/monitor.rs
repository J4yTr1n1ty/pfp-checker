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
    options: &[ResolvedOption<'_>],
) -> Result<(), serenity::Error> {
    if let Some(ResolvedOption {
        value: ResolvedValue::User(user, _),
        ..
    }) = options.first()
    {
        let user_id = i64::from(user.id); // Need to cast until I figure out how to implement the
                                          // trait for sqlx.

        let entry = sqlx::query!(
            "SELECT trackedSince FROM User WHERE discordId = ? LIMIT 1",
            user_id,
        )
        .fetch_one(database)
        .await;

        match entry {
            Ok(record) => {
                let tracking_start_date = record.trackedSince.unwrap() as i64;
                let dt = DateTime::from_timestamp(tracking_start_date, 0).unwrap();
                interaction
                    .create_response(
                        &ctx.http,
                        CreateInteractionResponse::Message(
                            CreateInteractionResponseMessage::new().content(format!(
                                "{} is already being tracked since <t:{}:F>",
                                user.name,
                                dt.timestamp()
                            )),
                        ),
                    )
                    .await
                    .unwrap();
                return Ok(());
            }
            Err(_) => (),
        }

        let now = SystemTime::now();
        let dt: DateTime<Utc> = now.clone().into();
        let timestamp = dt.timestamp();

        // Add the user to the database
        sqlx::query_as!(
            objects::User,
            "INSERT INTO User (discordId, trackedSince) VALUES (?, ?)",
            user_id,
            timestamp
        )
        .execute(database)
        .await
        .unwrap();

        // Reply with confirmation
        interaction
            .create_response(
                &ctx.http,
                CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new().content(format!(
                        "Sucessfully added {} to the monitoring list.",
                        user.name
                    )),
                ),
            )
            .await
            .unwrap();
    } else {
        interaction
            .create_response(
                &ctx.http,
                CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new().content("Invalid User ID."),
                ),
            )
            .await
            .unwrap();

        return Ok(());
    }

    Ok(())
}

pub fn register() -> CreateCommand {
    CreateCommand::new("monitor")
        .description("Adds a user to the Monitor List.")
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::User,
                "memberid",
                "The User to add to the monitor list.",
            )
            .required(true),
        )
}
