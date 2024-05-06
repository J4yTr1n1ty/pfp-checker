use serenity::builder::*;
use serenity::model::prelude::*;
use serenity::{prelude::*, utils::CreateQuickModal};
use sqlx::{query, SqlitePool};

pub async fn run(
    ctx: &Context,
    interaction: &CommandInteraction,
    database: &SqlitePool,
) -> Result<(), serenity::Error> {
    let modal = CreateQuickModal::new("Monitoring request").short_field("User ID to Monitor");
    let response = interaction.quick_modal(ctx, modal).await?.unwrap();

    let id = &response.inputs[0];

    // TODO: figure out how to add a cron job
    // query!("INSERT INTO monitorlist (userid) VALUES (?)", id).execute(&database);

    response
        .interaction
        .create_response(
            &ctx.http,
            CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new()
                    .content(format!("{id} will be added to the monitoring list.")),
            ),
        )
        .await?;

    Ok(())
}

pub fn register() -> CreateCommand {
    CreateCommand::new("monitor").description("Promts you for a User ID to monitor.")
}
