use dotenvy::dotenv;
use std::env;

pub struct Config {
    pub discord_token: String,
    pub database_url: String,
    pub imgbb_key: String,
}

impl Config {
    pub fn from_env() -> Result<Self, env::VarError> {
        dotenv().ok();

        Ok(Config {
            discord_token: env::var("DISCORD_TOKEN")?,
            database_url: env::var("DATABASE_URL")?,
            imgbb_key: env::var("IMGBB_KEY")?,
        })
    }
}
