use std::time::SystemTime;

use chrono::{DateTime, Utc};
use serenity::all::{Http, UserId};
use sha1::{Digest, Sha1};

use crate::util::{config::Config, external::imgbb};

pub async fn update_monitored_users(client: &Http, database: &sqlx::SqlitePool) {
    let config = Config::from_env().expect("Unable to load configuration.");

    let users_to_check = sqlx::query!("SELECT discordId FROM User")
        .fetch_all(database)
        .await;

    if let Ok(entries) = users_to_check {
        println!("Updating {} accounts...", entries.len());

        for entry in entries {
            let user_id = UserId::new(entry.discordId.try_into().unwrap());

            let user = user_id.to_user(client).await.unwrap_or_else(|_| {
                panic!(
                    "{}",
                    format!("Unable to retrieve User {}", entry.discordId).to_string()
                )
            });

            let tag = &user.name;

            println!("Updating Profile Picture for {user_id} ({tag})...");

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
                    // Check if same image as last change
                    let last_change_equals_now = sqlx::query!(
                        "SELECT CASE WHEN (SELECT checksum FROM ProfilePicture WHERE userId = ? ORDER BY changedAt DESC LIMIT 1) = ? THEN 1 ELSE 0 END AS equals",
                        entry.discordId,
                        checksum
                    )
                    .fetch_one(database)
                    .await
                    .expect("Failed to check if last change equals now");

                    if last_change_equals_now.equals == 1 {
                        continue;
                    } else {
                        println!(
                            "Updating pfp for {} with checksum {} (used previously)",
                            entry.discordId, checksum
                        );

                        let existing_image = sqlx::query!(
                            "SELECT link FROM ProfilePicture WHERE userId = ? ORDER BY changedAt DESC LIMIT 1",
                            entry.discordId
                        )
                        .fetch_one(database)
                        .await
                        .unwrap();

                        let image_url = existing_image.link;

                        let now = SystemTime::now();
                        let dt: DateTime<Utc> = now.into();
                        let timestamp = dt.timestamp();

                        sqlx::query!(
                            "INSERT INTO ProfilePicture (checksum, userId, changedAt, link) VALUES (?, ?, ?, ?)", 
                            checksum,
                            entry.discordId,
                            timestamp,
                            image_url).execute(database).await.unwrap();
                    }
                }
                None => {
                    let now = SystemTime::now();
                    let dt: DateTime<Utc> = now.into();
                    let timestamp = dt.timestamp();

                    println!(
                        "Writing new pfp for {} at {} with checksum {}",
                        entry.discordId,
                        dt.to_rfc2822(),
                        checksum
                    );

                    let filename = format!("pfp_{}_{}.png", user_id, timestamp);

                    let image_url =
                        imgbb::upload_image(bytes.to_vec(), filename, &config.imgbb_key)
                            .await
                            .unwrap();

                    sqlx::query!(
                        "INSERT INTO ProfilePicture (checksum, userId, changedAt, link) VALUES (?, ?, ?, ?)", 
                        checksum,
                        entry.discordId,
                        timestamp,
                        image_url).execute(database).await.unwrap();
                }
            }

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
