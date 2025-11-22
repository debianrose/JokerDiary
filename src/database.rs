use sqlx::SqlitePool;
use chrono::Utc;
use crate::Stats;

pub async fn init_db(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS users (
            id TEXT PRIMARY KEY,
            username TEXT UNIQUE NOT NULL,
            password_hash TEXT NOT NULL,
            created_at DATETIME NOT NULL
        )
        "#
    )
    .execute(pool)
    .await?;
    
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS sessions (
            id TEXT PRIMARY KEY,
            user_id TEXT NOT NULL,
            created_at DATETIME NOT NULL,
            expires_at DATETIME NOT NULL,
            FOREIGN KEY (user_id) REFERENCES users (id)
        )
        "#
    )
    .execute(pool)
    .await?;
    
    Ok(())
}

pub async fn get_stats(pool: &SqlitePool) -> Result<Stats, sqlx::Error> {
    let total_users: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
        .fetch_one(pool)
        .await?;
    
    let active_sessions: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM sessions WHERE expires_at > ?"
    )
    .bind(Utc::now())
    .fetch_one(pool)
    .await?;
    
    Ok(Stats {
        total_users,
        server_uptime: 0,
        active_sessions,
        server_status: "OK".to_string(),
    })
}
