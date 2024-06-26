mod commands;
mod util;

use dotenv::dotenv;
use std::env;
use std::sync::Arc;
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
                "history" => {
                    commands::history::run(&ctx, &command, &self.database, &command.data.options())
                        .await
                        .unwrap();
                    None
                }
                "stats" => {
                    commands::stats::run(&ctx, &command, &self.database, &command.data.options())
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
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);

        let _ = Command::set_global_commands(
            &ctx.http,
            vec![
                commands::ping::register(),
                commands::monitor::register(),
                commands::removemonitor::register(),
                commands::history::register(),
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
