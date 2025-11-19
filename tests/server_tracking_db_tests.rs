// ABOUTME: Integration tests for server tracking database functionality
// ABOUTME: Tests migrations, CRUD operations, and data integrity for Server and ServerPicture tables
use sqlx::{migrate::MigrateDatabase, Sqlite, SqlitePool};
use std::time::SystemTime;
use chrono::{DateTime, Utc};
use tempfile::TempDir;

/// Helper function to create a test database
async fn create_test_db() -> (SqlitePool, TempDir) {
    let temp_dir = tempfile::tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let db_url = format!("sqlite:{}", db_path.display());

    // Create database
    Sqlite::create_database(&db_url).await.unwrap();

    // Connect to database
    let pool = SqlitePool::connect(&db_url).await.unwrap();

    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .unwrap();

    (pool, temp_dir)
}

#[tokio::test]
async fn test_server_table_creation() {
    let (pool, _temp_dir) = create_test_db().await;

    // Verify Server table exists
    let result = sqlx::query!("SELECT name FROM sqlite_master WHERE type='table' AND name='Server'")
        .fetch_one(&pool)
        .await;

    assert!(result.is_ok(), "Server table should exist");
    pool.close().await;
}

#[tokio::test]
async fn test_server_picture_table_creation() {
    let (pool, _temp_dir) = create_test_db().await;

    // Verify ServerPicture table exists
    let result = sqlx::query!("SELECT name FROM sqlite_master WHERE type='table' AND name='ServerPicture'")
        .fetch_one(&pool)
        .await;

    assert!(result.is_ok(), "ServerPicture table should exist");
    pool.close().await;
}

#[tokio::test]
async fn test_insert_server() {
    let (pool, _temp_dir) = create_test_db().await;

    let server_id: i64 = 123456789;
    let now = SystemTime::now();
    let dt: DateTime<Utc> = now.into();
    let timestamp = dt.timestamp();

    // Insert a server
    let result = sqlx::query!(
        "INSERT INTO Server (serverId, trackedSince) VALUES (?, ?)",
        server_id,
        timestamp
    )
    .execute(&pool)
    .await;

    assert!(result.is_ok(), "Should be able to insert a server");
    assert_eq!(result.unwrap().rows_affected(), 1, "Should insert exactly one row");

    pool.close().await;
}

#[tokio::test]
async fn test_retrieve_server() {
    let (pool, _temp_dir) = create_test_db().await;

    let server_id: i64 = 987654321;
    let now = SystemTime::now();
    let dt: DateTime<Utc> = now.into();
    let timestamp = dt.timestamp();

    // Insert a server
    sqlx::query!(
        "INSERT INTO Server (serverId, trackedSince) VALUES (?, ?)",
        server_id,
        timestamp
    )
    .execute(&pool)
    .await
    .unwrap();

    // Retrieve the server
    let result = sqlx::query!(
        "SELECT serverId, trackedSince FROM Server WHERE serverId = ?",
        server_id
    )
    .fetch_one(&pool)
    .await;

    assert!(result.is_ok(), "Should be able to retrieve the server");
    let record = result.unwrap();
    assert_eq!(record.serverId, server_id, "Server ID should match");
    assert_eq!(record.trackedSince, Some(timestamp), "Timestamp should match");

    pool.close().await;
}

#[tokio::test]
async fn test_delete_server() {
    let (pool, _temp_dir) = create_test_db().await;

    let server_id: i64 = 111222333;
    let now = SystemTime::now();
    let dt: DateTime<Utc> = now.into();
    let timestamp = dt.timestamp();

    // Insert a server
    sqlx::query!(
        "INSERT INTO Server (serverId, trackedSince) VALUES (?, ?)",
        server_id,
        timestamp
    )
    .execute(&pool)
    .await
    .unwrap();

    // Delete the server
    let delete_result = sqlx::query!("DELETE FROM Server WHERE serverId = ?", server_id)
        .execute(&pool)
        .await;

    assert!(delete_result.is_ok(), "Should be able to delete the server");
    assert_eq!(delete_result.unwrap().rows_affected(), 1, "Should delete exactly one row");

    // Verify deletion
    let fetch_result = sqlx::query!(
        "SELECT serverId FROM Server WHERE serverId = ?",
        server_id
    )
    .fetch_optional(&pool)
    .await;

    assert!(fetch_result.is_ok());
    assert!(fetch_result.unwrap().is_none(), "Server should be deleted");

    pool.close().await;
}

#[tokio::test]
async fn test_insert_server_picture() {
    let (pool, _temp_dir) = create_test_db().await;

    let server_id: i64 = 444555666;
    let now = SystemTime::now();
    let dt: DateTime<Utc> = now.into();
    let timestamp = dt.timestamp();

    // Insert a server first
    sqlx::query!(
        "INSERT INTO Server (serverId, trackedSince) VALUES (?, ?)",
        server_id,
        timestamp
    )
    .execute(&pool)
    .await
    .unwrap();

    // Insert a server picture
    let checksum = "abcdef1234567890";
    let link = "https://example.com/icon.png";

    let result = sqlx::query!(
        "INSERT INTO ServerPicture (checksum, serverId, changedAt, link) VALUES (?, ?, ?, ?)",
        checksum,
        server_id,
        timestamp,
        link
    )
    .execute(&pool)
    .await;

    assert!(result.is_ok(), "Should be able to insert a server picture");
    assert_eq!(result.unwrap().rows_affected(), 1, "Should insert exactly one row");

    pool.close().await;
}

#[tokio::test]
async fn test_retrieve_server_pictures() {
    let (pool, _temp_dir) = create_test_db().await;

    let server_id: i64 = 777888999;
    let now = SystemTime::now();
    let dt: DateTime<Utc> = now.into();
    let timestamp = dt.timestamp();

    // Insert a server
    sqlx::query!(
        "INSERT INTO Server (serverId, trackedSince) VALUES (?, ?)",
        server_id,
        timestamp
    )
    .execute(&pool)
    .await
    .unwrap();

    // Insert multiple server pictures
    let pictures = vec![
        ("checksum1", "https://example.com/icon1.png", timestamp),
        ("checksum2", "https://example.com/icon2.png", timestamp + 100),
        ("checksum3", "https://example.com/icon3.png", timestamp + 200),
    ];

    for (checksum, link, time) in &pictures {
        sqlx::query!(
            "INSERT INTO ServerPicture (checksum, serverId, changedAt, link) VALUES (?, ?, ?, ?)",
            checksum,
            server_id,
            time,
            link
        )
        .execute(&pool)
        .await
        .unwrap();
    }

    // Retrieve all pictures for the server
    let results = sqlx::query!(
        "SELECT checksum, changedAt, link FROM ServerPicture WHERE serverId = ? ORDER BY changedAt DESC",
        server_id
    )
    .fetch_all(&pool)
    .await;

    assert!(results.is_ok(), "Should be able to retrieve server pictures");
    let records = results.unwrap();
    assert_eq!(records.len(), 3, "Should retrieve all three pictures");

    // Verify ordering (DESC by changedAt)
    assert_eq!(records[0].checksum.as_ref().unwrap(), "checksum3");
    assert_eq!(records[1].checksum.as_ref().unwrap(), "checksum2");
    assert_eq!(records[2].checksum.as_ref().unwrap(), "checksum1");

    pool.close().await;
}

#[tokio::test]
async fn test_cascade_delete_server_pictures() {
    let (pool, _temp_dir) = create_test_db().await;

    let server_id: i64 = 123123123;
    let now = SystemTime::now();
    let dt: DateTime<Utc> = now.into();
    let timestamp = dt.timestamp();

    // Insert a server
    sqlx::query!(
        "INSERT INTO Server (serverId, trackedSince) VALUES (?, ?)",
        server_id,
        timestamp
    )
    .execute(&pool)
    .await
    .unwrap();

    // Insert server pictures
    sqlx::query!(
        "INSERT INTO ServerPicture (checksum, serverId, changedAt, link) VALUES (?, ?, ?, ?)",
        "checksum_test",
        server_id,
        timestamp,
        "https://example.com/icon.png"
    )
    .execute(&pool)
    .await
    .unwrap();

    // Delete the server
    sqlx::query!("DELETE FROM Server WHERE serverId = ?", server_id)
        .execute(&pool)
        .await
        .unwrap();

    // Verify server pictures are also deleted (CASCADE)
    let pictures = sqlx::query!(
        "SELECT checksum FROM ServerPicture WHERE serverId = ?",
        server_id
    )
    .fetch_all(&pool)
    .await;

    assert!(pictures.is_ok());
    assert_eq!(pictures.unwrap().len(), 0, "Server pictures should be cascade deleted");

    pool.close().await;
}

#[tokio::test]
async fn test_unique_server_id_constraint() {
    let (pool, _temp_dir) = create_test_db().await;

    let server_id: i64 = 999888777;
    let now = SystemTime::now();
    let dt: DateTime<Utc> = now.into();
    let timestamp = dt.timestamp();

    // Insert a server
    sqlx::query!(
        "INSERT INTO Server (serverId, trackedSince) VALUES (?, ?)",
        server_id,
        timestamp
    )
    .execute(&pool)
    .await
    .unwrap();

    // Try to insert the same server again (should fail due to PRIMARY KEY constraint)
    let new_timestamp = timestamp + 1000;
    let duplicate_result = sqlx::query!(
        "INSERT INTO Server (serverId, trackedSince) VALUES (?, ?)",
        server_id,
        new_timestamp
    )
    .execute(&pool)
    .await;

    assert!(duplicate_result.is_err(), "Should not allow duplicate server IDs");

    pool.close().await;
}

#[tokio::test]
async fn test_server_picture_composite_key() {
    let (pool, _temp_dir) = create_test_db().await;

    let server_id: i64 = 555444333;
    let now = SystemTime::now();
    let dt: DateTime<Utc> = now.into();
    let timestamp = dt.timestamp();

    // Insert a server
    sqlx::query!(
        "INSERT INTO Server (serverId, trackedSince) VALUES (?, ?)",
        server_id,
        timestamp
    )
    .execute(&pool)
    .await
    .unwrap();

    let checksum = "test_checksum";
    let link = "https://example.com/icon.png";

    // Insert a server picture
    sqlx::query!(
        "INSERT INTO ServerPicture (checksum, serverId, changedAt, link) VALUES (?, ?, ?, ?)",
        checksum,
        server_id,
        timestamp,
        link
    )
    .execute(&pool)
    .await
    .unwrap();

    // Try to insert the same picture with same composite key (should fail)
    let duplicate_result = sqlx::query!(
        "INSERT INTO ServerPicture (checksum, serverId, changedAt, link) VALUES (?, ?, ?, ?)",
        checksum,
        server_id,
        timestamp,
        link
    )
    .execute(&pool)
    .await;

    assert!(duplicate_result.is_err(), "Should not allow duplicate composite keys");

    // But same checksum with different timestamp should work
    let different_timestamp = timestamp + 100;
    let different_time_result = sqlx::query!(
        "INSERT INTO ServerPicture (checksum, serverId, changedAt, link) VALUES (?, ?, ?, ?)",
        checksum,
        server_id,
        different_timestamp,
        link
    )
    .execute(&pool)
    .await;

    assert!(different_time_result.is_ok(), "Should allow same checksum with different timestamp");

    pool.close().await;
}

#[tokio::test]
async fn test_query_latest_server_picture() {
    let (pool, _temp_dir) = create_test_db().await;

    let server_id: i64 = 333222111;
    let now = SystemTime::now();
    let dt: DateTime<Utc> = now.into();
    let timestamp = dt.timestamp();

    // Insert a server
    sqlx::query!(
        "INSERT INTO Server (serverId, trackedSince) VALUES (?, ?)",
        server_id,
        timestamp
    )
    .execute(&pool)
    .await
    .unwrap();

    // Insert pictures at different times
    sqlx::query!(
        "INSERT INTO ServerPicture (checksum, serverId, changedAt, link) VALUES (?, ?, ?, ?)",
        "old_checksum",
        server_id,
        timestamp,
        "https://example.com/old.png"
    )
    .execute(&pool)
    .await
    .unwrap();

    let later_timestamp = timestamp + 1000;
    sqlx::query!(
        "INSERT INTO ServerPicture (checksum, serverId, changedAt, link) VALUES (?, ?, ?, ?)",
        "latest_checksum",
        server_id,
        later_timestamp,
        "https://example.com/latest.png"
    )
    .execute(&pool)
    .await
    .unwrap();

    // Query for the latest picture
    let latest = sqlx::query!(
        "SELECT checksum, link FROM ServerPicture WHERE serverId = ? ORDER BY changedAt DESC LIMIT 1",
        server_id
    )
    .fetch_one(&pool)
    .await;

    assert!(latest.is_ok(), "Should retrieve the latest picture");
    let record = latest.unwrap();
    assert_eq!(record.checksum.unwrap(), "latest_checksum", "Should get the latest checksum");
    assert_eq!(record.link.unwrap(), "https://example.com/latest.png", "Should get the latest link");

    pool.close().await;
}
