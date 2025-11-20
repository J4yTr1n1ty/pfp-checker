use serenity::all::{
    CommandInteraction, Context, CreateInteractionResponse, CreateInteractionResponseMessage,
};
use serenity::builder::CreateCommand;

pub async fn run(ctx: &Context, interaction: &CommandInteraction) -> Result<(), serenity::Error> {
    // Calculate API latency (time from command creation to now)
    let now = chrono::Utc::now();
    let command_timestamp = interaction.id.created_at();
    let api_latency = now.timestamp_millis() - command_timestamp.unix_timestamp() * 1000;

    let response_content = format!("🏓 Pong!\n**Latency:** {}ms", api_latency);

    interaction
        .create_response(
            ctx,
            CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new().content(response_content),
            ),
        )
        .await?;

    Ok(())
}

pub fn register() -> CreateCommand {
    CreateCommand::new("ping").description("Check bot latency and responsiveness")
}
