use chrono::DateTime;
use serenity::builder::*;
use serenity::model::prelude::*;
use serenity::prelude::*;

use sqlx::SqlitePool;

use crate::util::objects::ProfilePictureEntry;

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

        let user = sqlx::query!("SELECT discordId FROM User WHERE discordId = ?", user_id)
            .fetch_one(database)
            .await;

        match user {
            Ok(_) => {
                let profile_pictures =
                    sqlx::query!("SELECT * FROM ProfilePicture WHERE userId = ?", user_id)
                        .fetch_all(database)
                        .await;

                match profile_pictures {
                    Ok(entries) => {
                        let user = UserId::new(user_id.try_into().expect("Invalid User ID"));
                        let user = user.to_user(&ctx.http).await.unwrap();
                        let mut pfps: Vec<ProfilePictureEntry> = Vec::new();

                        for entry in entries {
                            let tracking_start_date = entry.changedAt.unwrap() as i64;
                            let dt = DateTime::from_timestamp(tracking_start_date, 0).unwrap();

                            pfps.push(ProfilePictureEntry {
                                title: format!(
                                    "Profile Picture first recorded <t:{}:R>",
                                    dt.timestamp()
                                ),
                                content: format!(
                                    "Link: {}\nChecksum: {}",
                                    entry.link.unwrap(),
                                    entry.checksum.unwrap()
                                ),
                                inline: false,
                            })
                        }

                        if pfps.is_empty() {
                            interaction.create_response(&ctx.http, CreateInteractionResponse::Message(CreateInteractionResponseMessage::new().content("No Profile picture entries found. Please check back in about 30 minutes."))).await.unwrap();
                            return Ok(());
                        }

                        let embed = CreateEmbed::new()
                            .title(format!("Profile Picture History of {}", user.tag()))
                            .fields(
                                pfps.into_iter()
                                    .map(|entry| (entry.title, entry.content, entry.inline)),
                            );

                        interaction
                            .create_response(
                                &ctx.http,
                                CreateInteractionResponse::Message(
                                    CreateInteractionResponseMessage::new().embed(embed),
                                ),
                            )
                            .await
                            .unwrap();
                    }
                    Err(_) => {
                        let embed = CreateEmbed::new()
                            .title("No History Found")
                            .description(
                                "The requested User has not been recorded yet. However they are queued for future monitoring. Please wait at least 30 minutes.",
                            )
                            .footer(CreateEmbedFooter::new(
                                "To add the user to tracking use /monitor @User",
                            ))
                            .colour(colours::branding::RED);

                        interaction
                            .create_response(
                                &ctx.http,
                                CreateInteractionResponse::Message(
                                    CreateInteractionResponseMessage::new().embed(embed),
                                ),
                            )
                            .await
                            .unwrap();
                    }
                }

                return Ok(());
            }
            Err(_) => {
                let embed = CreateEmbed::new()
                    .title("User not found")
                    .description(
                        "The User you requested the history of could not be found in our Database.",
                    )
                    .footer(CreateEmbedFooter::new(
                        "To add the user to tracking use /monitor @User",
                    ))
                    .colour(colours::branding::RED);

                interaction
                    .create_response(
                        &ctx.http,
                        CreateInteractionResponse::Message(
                            CreateInteractionResponseMessage::new().embed(embed),
                        ),
                    )
                    .await
                    .unwrap();
            }
        }
    }

    return Ok(());
}

pub fn register() -> CreateCommand {
    CreateCommand::new("history")
        .description("Shows the history for a specified user.")
        .add_option(
            CreateCommandOption::new(
                serenity::all::CommandOptionType::User,
                "memberid",
                "Member to show history for.",
            )
            .required(true),
        )
}
