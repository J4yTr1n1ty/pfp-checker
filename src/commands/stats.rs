use serenity::all::{
    CommandInteraction, Context, CreateCommandOption, CreateInteractionResponse,
    CreateInteractionResponseMessage, ResolvedValue,
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

        let entry = sqlx::query!("SELECT discordId FROM User WHERE discordId = ?", user_id)
            .fetch_one(database)
            .await;

        match entry {
            Ok(record) => {
                let pfps = sqlx::query!("SELECT * FROM ProfilePicture WHERE userId = ?", record.discordId).fetch_all(database).await;

                match pfps {
                    Ok(entries) => {
                        
                    },
                    Err(_) => {
                        interaction
                            .create_response(
                                &ctx, 
                                CreateInteractionResponse::Message(
                                    CreateInteractionResponseMessage::new()
                                        .content("No Profile Pictures have been recorded for this User. Please wait at least 30 minutes and check again.")))
                            .await
                            .unwrap();
                    },
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
        .add_option(CreateCommandOption::new(
            serenity::all::CommandOptionType::User,
            "member",
            "The member whose Statistics to show.",
        ))
}
