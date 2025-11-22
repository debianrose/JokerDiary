use actix_web::{web, App, HttpServer, HttpResponse, middleware::Logger};
use serde::{Deserialize, Serialize};
use sqlx::{SqlitePool, sqlite::SqlitePoolOptions};
use chrono::{Utc, DateTime};
use std::net::IpAddr;
use std::str::FromStr;

mod auth;
mod security;
mod database;

use auth::{register_user, login_user, UserCredentials};
use security::{is_ip_allowed, JwtToken};

#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct Stats {
    pub total_users: i64,
    pub server_uptime: i64,
    pub active_sessions: i64,
    pub server_status: String,
}

#[derive(Debug, Serialize)]
pub struct PingResponse {
    pub status: String,
    pub timestamp: DateTime<Utc>,
    pub version: String,
}

async fn ping() -> HttpResponse {
    HttpResponse::Ok().json(ApiResponse {
        success: true,
        data: Some(PingResponse {
            status: "OK".to_string(),
            timestamp: Utc::now(),
            version: "1.0.0".to_string(),
        }),
        message: "Сервер работает".to_string(),
    })
}

async fn register(
    pool: web::Data<SqlitePool>,
    credentials: web::Json<UserCredentials>,
) -> HttpResponse {
    match register_user(&pool, &credentials).await {
        Ok(user) => HttpResponse::Created().json(ApiResponse {
            success: true,
            data: Some(user),
            message: "Пользователь зарегистрирован".to_string(),
        }),
        Err(e) => HttpResponse::BadRequest().json(ApiResponse::<()> {
            success: false,
            data: None,
            message: e.to_string(),
        }),
    }
}

async fn login(
    pool: web::Data<SqlitePool>,
    credentials: web::Json<UserCredentials>,
) -> HttpResponse {
    match login_user(&pool, &credentials).await {
        Ok(token) => HttpResponse::Ok().json(ApiResponse {
            success: true,
            data: Some(token),
            message: "Успешный вход".to_string(),
        }),
        Err(e) => HttpResponse::Unauthorized().json(ApiResponse::<()> {
            success: false,
            data: None,
            message: e.to_string(),
        }),
    }
}

async fn stats(
    pool: web::Data<SqlitePool>,
    token: web::ReqData<JwtToken>,
    client_ip: web::ReqData<String>,
) -> HttpResponse {
    if !is_ip_allowed(&client_ip) {
        return HttpResponse::Forbidden().json(ApiResponse::<()> {
            success: false,
            data: None,
            message: "Доступ запрещен".to_string(),
        });
    }

    match database::get_stats(&pool).await {
        Ok(stats_data) => HttpResponse::Ok().json(ApiResponse {
            success: true,
            data: Some(stats_data),
            message: "Статистика получена".to_string(),
        }),
        Err(e) => HttpResponse::InternalServerError().json(ApiResponse::<()> {
            success: false,
            data: None,
            message: e.to_string(),
        }),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    
    println!("Запуск...");
    
    let database_url = "sqlite:diary.db";
    let pool = SqlitePoolOptions::new()
        .connect(database_url)
        .await
        .expect("Не удалось подключиться к базе данных");
    
    database::init_db(&pool).await
        .expect("Не удалось инициализировать базу данных");
    
    println!("Запущено.");
    
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .wrap(Logger::default())
            .service(
                web::scope("/diary")
                    .route("/", web::get().to(ping))
                    .route("/register", web::post().to(register))
                    .route("/login", web::post().to(login))
                    .route("/stats", web::get().to(stats))
            )
    })
    .bind("0.0.0.0:1161")?
    .run()
    .await
}
