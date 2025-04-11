use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::{Pool, Sqlite};
use std::sync::Arc;

pub async fn establish_connection(database_url: &str) -> Result<Arc<Pool<Sqlite>>, sqlx::Error> {
    // Create a connection pool
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(
            SqliteConnectOptions::new()
                .filename(database_url)
                .create_if_missing(true),
        )
        .await?;

    // Run migrations
    sqlx::migrate!("./migrations").run(&pool).await?;

    Ok(Arc::new(pool))
}
