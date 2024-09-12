use std::time::SystemTime;

use chrono::{DateTime, Utc};
use serenity::all::{Http, UserId};
use sha1::{Sha1, Digest};

use crate::util::uploader::upload_image_to_img_bb;

pub async fn update_monitored_users(client: &Http, database: &sqlx::SqlitePool) {
    let users_to_check = sqlx::query!("SELECT discordId FROM User")
        .fetch_all(database)
        .await;

    match users_to_check {
        Ok(entries) => {
            println!("Updating {} accounts...", entries.len());

            for entry in entries {
                let user_id = UserId::new(entry.discordId.try_into().unwrap());

                let user = user_id
                    .to_user(client)
                    .await
                    .expect(&format!("Unable to retrieve User {}", entry.discordId).to_string());

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
                        // Do nothing, still same pfp
                    }
                    None => {
                        let now = SystemTime::now();
                        let dt: DateTime<Utc> = now.clone().into();
                        let timestamp = dt.timestamp();

                        println!("Writing new pfp for {} at {} with checksum {}", entry.discordId, dt.to_rfc2822(), checksum);

                        let image_url = upload_image_to_img_bb(bytes.to_vec(), entry.discordId).await.unwrap();

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
                                let dt: DateTime<Utc> = now.clone().into();
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
        Err(_) => {
            return;
        }
    }
}

