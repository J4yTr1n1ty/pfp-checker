use chrono::DateTime;
use serenity::builder::*;
use serenity::model::prelude::*;
use serenity::prelude::*;

use sqlx::SqlitePool;

use crate::util::objects::ProfilePictureEntry;

const ENTRIES_PER_PAGE: usize = 10;

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
        let user_id = i64::from(user.id);

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
                        let user = user.to_user(&ctx.http).await?;
                        let pfps: Vec<ProfilePictureEntry> = entries
                            .into_iter()
                            .map(|entry| {
                                let tracking_start_date = entry.changedAt.unwrap() as i64;
                                let dt = DateTime::from_timestamp(tracking_start_date, 0).unwrap();
                                ProfilePictureEntry {
                                    title: format!(
                                        "Profile Picture first recorded <t:{}:R>",
                                        dt.timestamp()
                                    ),
                                    content: format!(
                                        "Link: [Look at the previous picture]({})\nChecksum: {}",
                                        entry.link.unwrap(),
                                        entry.checksum.unwrap()
                                    ),
                                    inline: false,
                                }
                            })
                            .collect();

                        if pfps.is_empty() {
                            interaction.create_response(&ctx.http, CreateInteractionResponse::Message(CreateInteractionResponseMessage::new().content("No Profile picture entries found. Please check back in about 30 minutes."))).await?;
                            return Ok(());
                        }

                        send_paginated_response(ctx, interaction, &user, &pfps, 0).await?;
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
                            .await?;
                    }
                }
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
                    .await?;
            }
        }
    }

    Ok(())
}

pub async fn get_paginated_embed_response(
    user: &User,
    pfps: &[ProfilePictureEntry],
    page: usize,
) -> Result<CreateInteractionResponse, serenity::Error> {
    let total_pages = (pfps.len() as f32 / ENTRIES_PER_PAGE as f32).ceil() as usize;
    let start = page * ENTRIES_PER_PAGE;
    let end = (start + ENTRIES_PER_PAGE).min(pfps.len());

    let embed = CreateEmbed::new()
        .title(format!("Profile Picture History of {}", user.tag()))
        .fields(
            pfps[start..end]
                .iter()
                .map(|entry| (entry.title.clone(), entry.content.clone(), entry.inline)),
        )
        .footer(CreateEmbedFooter::new(format!(
            "Page {} of {}",
            page + 1,
            total_pages
        )));

    let components = CreateActionRow::Buttons(vec![
        CreateButton::new(format!("history_back_{}_{}", page, user.id))
            .label("Back")
            .style(ButtonStyle::Primary)
            .disabled(page == 0),
        CreateButton::new(format!("history_next_{}_{}", page, user.id))
            .label("Next")
            .style(ButtonStyle::Primary)
            .disabled(end == pfps.len()),
    ]);

    return Ok(CreateInteractionResponse::Message(
        CreateInteractionResponseMessage::new()
            .embed(embed)
            .components(vec![components]),
    ));
}

pub async fn send_paginated_response(
    ctx: &Context,
    interaction: &CommandInteraction,
    user: &User,
    pfps: &[ProfilePictureEntry],
    page: usize,
) -> Result<(), serenity::Error> {
    let response = get_paginated_embed_response(user, pfps, page)
        .await
        .unwrap();

    interaction.create_response(&ctx.http, response).await?;

    Ok(())
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
