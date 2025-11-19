// ABOUTME: Command to display paginated history of server icon changes
// ABOUTME: Shows all recorded server icons with timestamps and navigation buttons
use chrono::DateTime;
use serenity::all::{
    ButtonStyle, CommandInteraction, Context, CreateActionRow, CreateButton, CreateEmbed,
    CreateEmbedFooter, CreateInteractionResponse, CreateInteractionResponseMessage, GuildId,
};
use serenity::builder::{CreateCommand, EditMessage};

use sqlx::SqlitePool;

use crate::util::objects::EmbedEntry;

const ENTRIES_PER_PAGE: usize = 10;

pub async fn run(
    ctx: &Context,
    interaction: &CommandInteraction,
    database: &SqlitePool,
) -> Result<(), serenity::Error> {
    // Get the guild (server) from the interaction
    let guild_id = match interaction.guild_id {
        Some(id) => id,
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

    // Fetch server picture history from database
    let guild_id_i64 = i64::from(guild_id);
    let entries = sqlx::query!(
        "SELECT checksum, changedAt, link FROM ServerPicture WHERE serverId = ? ORDER BY changedAt DESC",
        guild_id_i64
    )
    .fetch_all(database)
    .await;

    match entries {
        Ok(records) => {
            if records.is_empty() {
                interaction
                    .create_response(
                        &ctx.http,
                        CreateInteractionResponse::Message(
                            CreateInteractionResponseMessage::new().content(format!(
                                "{} has no recorded server icon history.",
                                guild_name
                            )),
                        ),
                    )
                    .await?;
                return Ok(());
            }

            let mut embed_entries: Vec<EmbedEntry> = Vec::new();

            for record in records {
                let timestamp = record.changedAt.unwrap();
                let dt = DateTime::from_timestamp(timestamp, 0).unwrap();
                let checksum = record.checksum.unwrap();
                let link = record.link.unwrap();

                embed_entries.push(EmbedEntry {
                    title: format!("<t:{}:F>", dt.timestamp()),
                    content: format!("[Link]({})\nChecksum: {}", link, checksum),
                    inline: false,
                });
            }

            send_paginated_response(ctx, interaction, &guild_name, guild_id, &embed_entries, 0)
                .await?;
        }
        Err(_) => {
            interaction
                .create_response(
                    &ctx.http,
                    CreateInteractionResponse::Message(
                        CreateInteractionResponseMessage::new()
                            .content("Failed to fetch server icon history."),
                    ),
                )
                .await?;
        }
    }

    Ok(())
}

pub async fn get_paginated_embed_response(
    guild_name: &str,
    guild_id: GuildId,
    icons: &[EmbedEntry],
    page: usize,
) -> Result<CreateInteractionResponse, serenity::Error> {
    let total_pages = (icons.len() as f32 / ENTRIES_PER_PAGE as f32).ceil() as usize;
    let start = page * ENTRIES_PER_PAGE;
    let end = (start + ENTRIES_PER_PAGE).min(icons.len());

    let embed = CreateEmbed::new()
        .title(format!("{} Server Icon History", guild_name))
        .fields(
            icons[start..end]
                .iter()
                .map(|entry| (entry.title.clone(), entry.content.clone(), entry.inline)),
        )
        .footer(CreateEmbedFooter::new(format!(
            "Page {} of {}",
            page + 1,
            total_pages
        )));

    let components = CreateActionRow::Buttons(vec![
        CreateButton::new(format!("serverpfphistory_back_{}_{}", page, guild_id))
            .label("Back")
            .style(ButtonStyle::Primary)
            .disabled(page == 0),
        CreateButton::new(format!("serverpfphistory_next_{}_{}", page, guild_id))
            .label("Next")
            .style(ButtonStyle::Primary)
            .disabled(end == icons.len()),
    ]);

    Ok(CreateInteractionResponse::Message(
        CreateInteractionResponseMessage::new()
            .embed(embed)
            .components(vec![components]),
    ))
}

pub async fn get_paginated_embed_edit_response(
    guild_name: &str,
    guild_id: GuildId,
    icons: &[EmbedEntry],
    page: usize,
) -> Result<EditMessage, serenity::Error> {
    let total_pages = (icons.len() as f32 / ENTRIES_PER_PAGE as f32).ceil() as usize;
    let start = page * ENTRIES_PER_PAGE;
    let end = (start + ENTRIES_PER_PAGE).min(icons.len());

    let embed = CreateEmbed::new()
        .title(format!("{} Server Icon History", guild_name))
        .fields(
            icons[start..end]
                .iter()
                .map(|entry| (entry.title.clone(), entry.content.clone(), entry.inline)),
        )
        .footer(CreateEmbedFooter::new(format!(
            "Page {} of {}",
            page + 1,
            total_pages
        )));

    let components = CreateActionRow::Buttons(vec![
        CreateButton::new(format!("serverpfphistory_back_{}_{}", page, guild_id))
            .label("Back")
            .style(ButtonStyle::Primary)
            .disabled(page == 0),
        CreateButton::new(format!("serverpfphistory_next_{}_{}", page, guild_id))
            .label("Next")
            .style(ButtonStyle::Primary)
            .disabled(end == icons.len()),
    ]);

    Ok(EditMessage::new().embed(embed).components(vec![components]))
}

pub async fn send_paginated_response(
    ctx: &Context,
    interaction: &CommandInteraction,
    guild_name: &str,
    guild_id: GuildId,
    icons: &[EmbedEntry],
    page: usize,
) -> Result<(), serenity::Error> {
    let response = get_paginated_embed_response(guild_name, guild_id, icons, page)
        .await
        .unwrap();

    interaction.create_response(&ctx.http, response).await?;

    Ok(())
}

pub fn register() -> CreateCommand {
    CreateCommand::new("serverpfphistory")
        .description("Displays the server icon history for this server.")
}
