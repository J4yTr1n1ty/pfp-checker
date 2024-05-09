mod commands;
mod util;

use chrono::{DateTime, Utc};
use dotenv::dotenv;
use serenity::all::{Http, UserId};
use sha1::{Digest, Sha1};
use std::env;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::task;

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
        if let Interaction::Command(command) = interaction {
            let content = match command.data.name.as_str() {
                "ping" => Some(commands::ping::run(&command.data.options())),
                "monitor" => {
                    commands::monitor::run(&ctx, &command, &self.database, &command.data.options())
                        .await
                        .unwrap();
                    update_monitored_users(&ctx.http, &self.database).await;
                    None
                }
                "removemonitor" => {
                    commands::removemonitor::run(&ctx, &command, &self.database, &command.data.options()).await.unwrap();
                    None
                }
                "history" => {
                    commands::history::run(&ctx, &command, &self.database, &command.data.options()).await.unwrap();
                    None
                }
                _ => Some("not implemented :(".to_string()),
            };

            if let Some(content) = content {
                let data = CreateInteractionResponseMessage::new().content(content);
                let builder = CreateInteractionResponse::Message(data);
                if let Err(why) = command.create_response(&ctx.http, builder).await {
                    println!("Cannot respond to slash command: {why}");
                }
            }
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);

        let _ = Command::set_global_commands(
            &ctx.http,
            vec![commands::ping::register(), commands::monitor::register(), commands::removemonitor::register(), commands::history::register()],
        )
        .await
        .unwrap();

        println!("Updated Global Application Commands");
        
        let database_clone = Arc::clone(&self.database);

        let update_scheduler = task::spawn(async move {
            let mut interval = interval(Duration::from_secs(30 * 60));

            loop {
                interval.tick().await;
                update_monitored_users(&ctx.http, &database_clone).await;
            }
        });

        let _ = update_scheduler.await;
    }
}

async fn update_monitored_users(client: &Http, database: &sqlx::SqlitePool) {
    let users_to_check = sqlx::query!("SELECT discordId FROM User")
        .fetch_all(database)
        .await;

    match users_to_check {
        Ok(entries) => {
            println!("Updating {} accounts...", entries.len());

            for entry in entries {
                println!("Trying to parse {}", entry.discordId);
                let user_id = UserId::new(entry.discordId.try_into().unwrap());

                println!("Updating {user_id}...");

                let user = user_id
                    .to_user(client)
                    .await
                    .expect(&format!("Unable to retrieve User {}", entry.discordId).to_string());

                let avatar_url = user.face();

                let response = reqwest::get(&avatar_url).await.unwrap();
                let bytes = response.bytes().await.unwrap();

                let mut hasher = Sha1::new();
                hasher.update(&bytes);

                let result = hasher.finalize();
                let checksum = format!("{:x}", result); // Format as hex

                let already_existing_record = sqlx::query!(
                    "SELECT checksum FROM ProfilePicture WHERE checksum = ? AND userId = ?",
                    checksum,
                    entry.discordId
                )
                .fetch_optional(database)
                .await
                .unwrap();

                match already_existing_record {
                    Some(_) => {
                        continue;
                    }
                    None => {
                        let now = SystemTime::now();
                        let dt: DateTime<Utc> = now.clone().into();
                        let timestamp = dt.timestamp();

                        println!("Writing new pfp for {} at {} with checksum {}", entry.discordId, dt.to_rfc2822(), checksum);

                        sqlx::query!(
                            "INSERT INTO ProfilePicture (checksum, userId, changedAt, link) VALUES (?, ?, ?, ?)", 
                            checksum, 
                            entry.discordId, 
                            timestamp, 
                            avatar_url).execute(database).await.unwrap();
                    }
                }
            }
        }
        Err(_) => {
            return;
        }
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok(); // load all environment variables from the .env file

    // Configure the client with your Discord bot token in the environment.
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    // Initiate a connection to the database file, creating the file if required.
    let database = Arc::new(sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(
            sqlx::sqlite::SqliteConnectOptions::new()
                .filename("database.sqlite")
                .create_if_missing(true),
        )
        .await
        .expect("Couldn't connect to database"));

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
