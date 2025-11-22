use serde::{Deserialize, Serialize};
use sqlx::{SqlitePool, Row};
use bcrypt::{hash, verify, DEFAULT_COST};
use uuid::Uuid;
use crate::security::create_jwt;

#[derive(Debug, Deserialize)]
pub struct UserCredentials {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct User {
    pub id: String,
    pub username: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: User,
}

pub async fn register_user(
    pool: &SqlitePool,
    credentials: &UserCredentials,
) -> Result<AuthResponse, Box<dyn std::error::Error>> {
    let existing_user = sqlx::query("SELECT id FROM users WHERE username = ?")
        .bind(&credentials.username)
        .fetch_optional(pool)
        .await?;
    
    if existing_user.is_some() {
        return Err("Пользователь уже существует".into());
    }
    
    let hashed_password = hash(&credentials.password, DEFAULT_COST)?;
    let user_id = Uuid::new_v4().to_string();
    
    sqlx::query(
        "INSERT INTO users (id, username, password_hash, created_at) VALUES (?, ?, ?, ?)"
    )
    .bind(&user_id)
    .bind(&credentials.username)
    .bind(&hashed_password)
    .bind(chrono::Utc::now())
    .execute(pool)
    .await?;
    
    let token = create_jwt(&user_id)?;
    
    let user = User {
        id: user_id,
        username: credentials.username.clone(),
        created_at: chrono::Utc::now(),
    };
    
    Ok(AuthResponse { token, user })
}

pub async fn login_user(
    pool: &SqlitePool,
    credentials: &UserCredentials,
) -> Result<AuthResponse, Box<dyn std::error::Error>> {
    let row = sqlx::query("SELECT id, username, password_hash FROM users WHERE username = ?")
        .bind(&credentials.username)
        .fetch_optional(pool)
        .await?;
    
    let row = row.ok_or("Неверное имя пользователя или пароль")?;
    
    let user_id: String = row.get("id");
    let username: String = row.get("username");
    let password_hash: String = row.get("password_hash");
    
    if verify(&credentials.password, &password_hash)? {
        let token = create_jwt(&user_id)?;
        
        let user = User {
            id: user_id,
            username,
            created_at: chrono::Utc::now(),
        };
        
        Ok(AuthResponse { token, user })
    } else {
        Err("Неверное имя пользователя или пароль".into())
    }
}
