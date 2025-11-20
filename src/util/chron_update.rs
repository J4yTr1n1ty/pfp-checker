// ABOUTME: Scheduled update functions for monitoring user profile pictures and server icons
// ABOUTME: Checks for changes, calculates checksums, uploads new images to imgbb, and stores history
use std::future::Future;
use std::pin::Pin;
use std::time::SystemTime;

use chrono::{DateTime, Utc};
use serenity::all::{GuildId, Http, UserId};
use sha1::{Digest, Sha1};

use crate::util::{config::Config, external::imgbb};

/// Generic helper for monitoring entities (users or servers) and tracking image changes
async fn update_monitored_entity<'a, FetchIds, GetImageUrl, FormatId>(
    client: &'a Http,
    database: &'a sqlx::SqlitePool,
    config: &'a Config,
    fetch_entity_ids: FetchIds,
    get_image_url: GetImageUrl,
    format_entity_id: FormatId,
    table_name: &'static str,
    id_column_name: &'static str,
    filename_prefix: &'static str,
    entity_type_name: &'static str,
) where
    FetchIds: Fn(
        &'a sqlx::SqlitePool,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<i64>, sqlx::Error>> + Send + 'a>>,
    GetImageUrl: Fn(&'a Http, i64) -> Pin<Box<dyn Future<Output = Option<String>> + Send + 'a>>,
    FormatId: Fn(i64) -> String,
{
    let entity_ids = fetch_entity_ids(database).await;

    if let Ok(entries) = entity_ids {
        println!("Updating {} {}...", entries.len(), entity_type_name);

        for entity_id in entries {
            let entity_id_str = format_entity_id(entity_id);
            println!("Updating {} for {}...", entity_type_name, entity_id_str);

            // Get the image URL for this entity
            let image_url = match get_image_url(client, entity_id).await {
                Some(url) => url,
                None => {
                    println!(
                        "{} {} has no image, skipping...",
                        entity_type_name, entity_id_str
                    );
                    continue;
                }
            };

            // Download image
            let response = match reqwest::get(&image_url).await {
                Ok(r) => r,
                Err(e) => {
                    println!("Failed to download image for {}: {:?}", entity_id_str, e);
                    continue;
                }
            };

            let bytes = match response.bytes().await {
                Ok(b) => b,
                Err(e) => {
                    println!("Failed to read image bytes for {}: {:?}", entity_id_str, e);
                    continue;
                }
            };

            // Compute SHA1 checksum
            let mut hasher = Sha1::new();
            hasher.update(&bytes);
            let result = hasher.finalize();
            let checksum = format!("{:x}", result);

            // Check if this checksum already exists for this entity
            let check_query = format!(
                "SELECT checksum FROM {} WHERE checksum = ? AND {} = ?",
                table_name, id_column_name
            );

            let already_existing_record = sqlx::query_scalar::<_, String>(&check_query)
                .bind(&checksum)
                .bind(entity_id)
                .fetch_optional(database)
                .await
                .unwrap();

            match already_existing_record {
                Some(_) => {
                    // Check if same image as last change
                    let last_check_query = format!(
                        "SELECT CASE WHEN (SELECT checksum FROM {} WHERE {} = ? ORDER BY changedAt DESC LIMIT 1) = ? THEN 1 ELSE 0 END AS equals",
                        table_name, id_column_name
                    );

                    let last_change_equals_now = sqlx::query_scalar::<_, i32>(&last_check_query)
                        .bind(entity_id)
                        .bind(&checksum)
                        .fetch_one(database)
                        .await
                        .expect("Failed to check if last change equals now");

                    if last_change_equals_now == 1 {
                        continue;
                    } else {
                        println!(
                            "Updating {} for {} with checksum {} (used previously)",
                            entity_type_name, entity_id, checksum
                        );

                        // Get existing image link
                        let link_query = format!(
                            "SELECT link FROM {} WHERE {} = ? ORDER BY changedAt DESC LIMIT 1",
                            table_name, id_column_name
                        );

                        let existing_image = sqlx::query_scalar::<_, Option<String>>(&link_query)
                            .bind(entity_id)
                            .fetch_one(database)
                            .await
                            .unwrap();

                        let image_url = existing_image;

                        let now = SystemTime::now();
                        let dt: DateTime<Utc> = now.into();
                        let timestamp = dt.timestamp();

                        // Insert new record with existing link
                        let insert_query = format!(
                            "INSERT INTO {} (checksum, {}, changedAt, link) VALUES (?, ?, ?, ?)",
                            table_name, id_column_name
                        );

                        sqlx::query(&insert_query)
                            .bind(&checksum)
                            .bind(entity_id)
                            .bind(timestamp)
                            .bind(image_url)
                            .execute(database)
                            .await
                            .unwrap();
                    }
                }
                None => {
                    let now = SystemTime::now();
                    let dt: DateTime<Utc> = now.into();
                    let timestamp = dt.timestamp();

                    println!(
                        "Writing new {} for {} at {} with checksum {}",
                        entity_type_name,
                        entity_id,
                        dt.to_rfc2822(),
                        checksum
                    );

                    let filename = format!("{}{}_{}.png", filename_prefix, entity_id, timestamp);

                    let image_url =
                        imgbb::upload_image(bytes.to_vec(), filename, &config.imgbb_key)
                            .await
                            .unwrap();

                    // Insert new record
                    let insert_query = format!(
                        "INSERT INTO {} (checksum, {}, changedAt, link) VALUES (?, ?, ?, ?)",
                        table_name, id_column_name
                    );

                    sqlx::query(&insert_query)
                        .bind(&checksum)
                        .bind(entity_id)
                        .bind(timestamp)
                        .bind(image_url)
                        .execute(database)
                        .await
                        .unwrap();
                }
            }
        }
    }
}

pub async fn update_monitored_users(client: &Http, database: &sqlx::SqlitePool) {
    let config = Config::from_env().expect("Unable to load configuration.");

    // Update profile pictures using the generic helper
    update_monitored_entity(
        client,
        database,
        &config,
        |db| {
            Box::pin(async move {
                sqlx::query_scalar::<_, i64>("SELECT discordId FROM User")
                    .fetch_all(db)
                    .await
            })
        },
        |client, user_id| {
            Box::pin(async move {
                let user_id_obj = UserId::new(user_id.try_into().unwrap());
                match user_id_obj.to_user(client).await {
                    Ok(user) => Some(user.face()),
                    Err(_) => {
                        println!("Unable to retrieve User {}", user_id);
                        None
                    }
                }
            })
        },
        |id| format!("{}", id),
        "ProfilePicture",
        "userId",
        "pfp_",
        "profile picture",
    )
    .await;

    // Update usernames (this logic is unique to users, so keep it here)
    let users_to_check = sqlx::query!("SELECT discordId FROM User")
        .fetch_all(database)
        .await;

    if let Ok(entries) = users_to_check {
        for entry in entries {
            let user_id = UserId::new(entry.discordId.try_into().unwrap());

            let user = match user_id.to_user(client).await {
                Ok(u) => u,
                Err(_) => {
                    println!(
                        "Unable to retrieve User {} for username update",
                        entry.discordId
                    );
                    continue;
                }
            };

            let tag = &user.name;
            println!("Updating Usernames for {user_id} ({tag})...");

            let username = &user.global_name;

            match username {
                Some(username) => {
                    let already_existing_record = sqlx::query!(
                        "SELECT username FROM UsernameChange WHERE username = ? AND userId = ?",
                        username,
                        entry.discordId
                    )
                    .fetch_optional(database)
                    .await
                    .unwrap();

                    match already_existing_record {
                        Some(_) => {
                            // Still same username
                            continue;
                        }
                        None => {
                            let now = SystemTime::now();
                            let dt: DateTime<Utc> = now.into();
                            let timestamp = dt.timestamp();

                            sqlx::query!(
                                "INSERT INTO UsernameChange (changedAt, username, userId) VALUES (?, ?, ?)",
                                timestamp,
                                username,
                                entry.discordId
                            )
                            .execute(database)
                            .await
                            .unwrap();

                            println!("Updated username for {} to {}", entry.discordId, username);
                        }
                    }
                }
                None => {
                    continue;
                }
            }
        }
    }
}

pub async fn update_monitored_servers(client: &Http, database: &sqlx::SqlitePool) {
    let config = Config::from_env().expect("Unable to load configuration.");

    // Update server icons using the generic helper
    update_monitored_entity(
        client,
        database,
        &config,
        |db| {
            Box::pin(async move {
                sqlx::query_scalar::<_, i64>("SELECT serverId FROM Server")
                    .fetch_all(db)
                    .await
            })
        },
        |client, server_id| {
            Box::pin(async move {
                let guild_id = GuildId::new(server_id.try_into().unwrap());
                match guild_id.to_partial_guild(client).await {
                    Ok(guild) => guild.icon_url(),
                    Err(_) => {
                        println!("Unable to retrieve Server {}", server_id);
                        None
                    }
                }
            })
        },
        |id| format!("{}", id),
        "ServerPicture",
        "serverId",
        "server_icon_",
        "server icon",
    )
    .await;
}
