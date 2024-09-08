mod commands;
mod util;

use dotenv::dotenv;
use serenity::all::UserId;
use std::env;
use std::sync::Arc;
use tokio::task;
use util::objects;

use serenity::async_trait;
use serenity::builder::{CreateInteractionResponse, CreateInteractionResponseMessage};
use serenity::model::application::{Command, Interaction};
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use tokio::time::{interval, Duration};

struct Handler {
    database: Arc<sqlx::SqlitePool>,
}

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        match interaction {
            Interaction::Command(command) => {
                let content = match command.data.name.as_str() {
                    "ping" => Some(commands::ping::run(&command.data.options())),
                    "monitor" => {
                        commands::monitor::run(
                            &ctx,
                            &command,
                            &self.database,
                            &command.data.options(),
                        )
                        .await
                        .unwrap();
                        util::chron_update::update_monitored_users(&ctx.http, &self.database).await;
                        None
                    }
                    "removemonitor" => {
                        commands::removemonitor::run(
                            &ctx,
                            &command,
                            &self.database,
                            &command.data.options(),
                        )
                        .await
                        .unwrap();
                        None
                    }
                    "pfphistory" => {
                        commands::pfphistory::run(
                            &ctx,
                            &command,
                            &self.database,
                            &command.data.options(),
                        )
                        .await
                        .unwrap();
                        None
                    }
                    "usernamehistory" => {
                        commands::usernamehistory::run(
                            &ctx,
                            &command,
                            &self.database,
                            &command.data.options(),
                        )
                        .await
                        .unwrap();
                        None
                    }
                    "stats" => {
                        commands::stats::run(
                            &ctx,
                            &command,
                            &self.database,
                            &command.data.options(),
                        )
                        .await
                        .unwrap();
                        None
                    }
                    _ => Some(format!("{} is not implemented :(", command.data.name)),
                };

                if let Some(content) = content {
                    let data = CreateInteractionResponseMessage::new().content(content);
                    let builder = CreateInteractionResponse::Message(data);
                    if let Err(why) = command.create_response(&ctx.http, builder).await {
                        println!("Cannot respond to slash command: {why}");
                    }
                }
            }
            Interaction::Component(component) => {
                let custom_id = &component.data.custom_id;
                let mut sender_message = component.message.to_owned();
                if custom_id.starts_with("pfphistory_") {
                    let parts: Vec<&str> = custom_id.split('_').collect();
                    if parts.len() == 4 {
                        let direction = parts[1];
                        let current_page: usize = parts[2].parse().unwrap_or(0);
                        let new_page = match direction {
                            "back" => current_page.saturating_sub(1),
                            "next" => current_page + 1,
                            _ => current_page,
                        };

                        // Fetch the user and pfps data again
                        let user_id = parts[3].parse::<UserId>().unwrap();
                        let user = user_id.to_user(&ctx.http).await.unwrap();
                        let pfps = fetch_profile_pictures(&self.database, i64::from(user_id))
                            .await
                            .unwrap();

                        let response = commands::pfphistory::get_paginated_embed_edit_response(
                            &user, &pfps, new_page,
                        )
                        .await
                        .unwrap();

                        if let Err(why) = component
                            .create_response(&ctx.http, CreateInteractionResponse::Acknowledge)
                            .await
                        {
                            println!("Cannot respond to slash command: {why}")
                        }

                        if let Err(why) = sender_message.edit(&ctx.http, response).await {
                            println!("Cannot respond to slash command: {why}");
                        }
                    }
                }

                if custom_id.starts_with("usernamehistory_") {
                    let parts: Vec<&str> = custom_id.split('_').collect();
                    if parts.len() == 4 {
                        let direction = parts[1];
                        let current_page: usize = parts[2].parse().unwrap_or(0);
                        let new_page = match direction {
                            "back" => current_page.saturating_sub(1),
                            "next" => current_page + 1,
                            _ => current_page,
                        };

                        // Fetch the user and pfps data again
                        let user_id = parts[3].parse::<UserId>().unwrap();
                        let user = user_id.to_user(&ctx.http).await.unwrap();
                        let pfps = fetch_usernames(&self.database, i64::from(user_id))
                            .await
                            .unwrap();

                        let response =
                            commands::usernamehistory::get_paginated_embed_edit_response(
                                &user, &pfps, new_page,
                            )
                            .await
                            .unwrap();

                        if let Err(why) = component
                            .create_response(&ctx.http, CreateInteractionResponse::Acknowledge)
                            .await
                        {
                            println!("Cannot respond to slash command: {why}")
                        }

                        if let Err(why) = sender_message.edit(&ctx.http, response).await {
                            println!("Cannot respond to slash command: {why}");
                        }
                    }
                }
            }
            _ => {}
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);

        let _ = Command::set_global_commands(
            &ctx.http,
            vec![
                commands::ping::register(),
                commands::monitor::register(),
                commands::removemonitor::register(),
                commands::pfphistory::register(),
                commands::usernamehistory::register(),
                commands::stats::register(),
            ],
        )
        .await
        .unwrap();

        println!("Updated Global Application Commands");

        let database_clone = Arc::clone(&self.database);

        let update_scheduler = task::spawn(async move {
            let mut interval = interval(Duration::from_secs(30 * 60));

            loop {
                interval.tick().await;
                util::chron_update::update_monitored_users(&ctx.http, &database_clone).await;
            }
        });

        let _ = update_scheduler.await;
    }
}

async fn fetch_usernames(
    database: &sqlx::SqlitePool,
    user_id: i64,
) -> Result<Vec<objects::EmbedEntry>, sqlx::Error> {
    let entries = sqlx::query!("SELECT * FROM UsernameChange WHERE userId = ?", user_id)
        .fetch_all(database)
        .await?;

    Ok(entries
        .into_iter()
        .map(|entry| {
            let tracking_start_date = entry.changedAt.unwrap() as i64;
            let dt = chrono::DateTime::from_timestamp(tracking_start_date, 0).unwrap();
            objects::EmbedEntry {
                title: format!("Username first recorded <t:{}:R>", dt.timestamp()),
                content: format!("{}", entry.username.unwrap()),
                inline: false,
            }
        })
        .collect())
}

async fn fetch_profile_pictures(
    database: &sqlx::SqlitePool,
    user_id: i64,
) -> Result<Vec<objects::EmbedEntry>, sqlx::Error> {
    let entries = sqlx::query!("SELECT * FROM ProfilePicture WHERE userId = ?", user_id)
        .fetch_all(database)
        .await?;

    Ok(entries
        .into_iter()
        .map(|entry| {
            let tracking_start_date = entry.changedAt.unwrap() as i64;
            let dt = chrono::DateTime::from_timestamp(tracking_start_date, 0).unwrap();
            objects::EmbedEntry {
                title: format!("Profile Picture first recorded <t:{}:R>", dt.timestamp()),
                content: format!(
                    "Link: [Look at the previous picture]({})\nChecksum: {}",
                    entry.link.unwrap(),
                    entry.checksum.unwrap()
                ),
                inline: false,
            }
        })
        .collect())
}

#[tokio::main]
async fn main() {
    dotenv().ok(); // load all environment variables from the .env file

    // Configure the client with your Discord bot token in the environment.
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    // Initiate a connection to the database file, creating the file if required.
    let database = Arc::new(
        sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(
                sqlx::sqlite::SqliteConnectOptions::new()
                    .filename("database.sqlite")
                    .create_if_missing(true),
            )
            .await
            .expect("Couldn't connect to database"),
    );

    // Run migrations, which updates the database's schema to the latest version.
    sqlx::migrate!("./migrations")
        .run(&*database)
        .await
        .expect("Couldn't run database migrations");

    let handler = Handler { database };

    // Build our client.
    let mut client = Client::builder(token, GatewayIntents::empty())
        .event_handler(handler)
        .await
        .expect("Error creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
}
