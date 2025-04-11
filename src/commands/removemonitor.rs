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

        let delete_result = sqlx::query!("DELETE FROM User WHERE discordId = ?", user_id)
            .execute(database)
            .await;

        match delete_result {
            Ok(res) => {
                if res.rows_affected() > 0 {
                    interaction
                        .create_response(
                            &ctx.http,
                            CreateInteractionResponse::Message(
                                CreateInteractionResponseMessage::new()
                                    .content("Sucessfully deleted user."),
                            ),
                        )
                        .await
                        .unwrap();
                } else {
                    interaction
                        .create_response(
                            &ctx.http,
                            CreateInteractionResponse::Message(
                                CreateInteractionResponseMessage::new()
                                    .content("Unable to find user. User may not be tracked."),
                            ),
                        )
                        .await
                        .unwrap();
                }
            }
            Err(_) => {
                interaction
                    .create_response(
                        &ctx.http,
                        CreateInteractionResponse::Message(
                            CreateInteractionResponseMessage::new()
                                .content("Unable to delete User. User may not be tracked."),
                        ),
                    )
                    .await
                    .unwrap();
            }
        }

        return Ok(());
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
    }

    Ok(())
}

pub fn register() -> CreateCommand {
    CreateCommand::new("removemonitor")
        .description("Removes a user from the Monitor List.")
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::User,
                "memberid",
                "The User to remove from the monitor list.",
            )
            .required(true),
        )
}
