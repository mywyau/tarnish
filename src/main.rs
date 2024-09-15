pub mod connectors;
pub mod controllers;
pub mod models;
pub mod middleware;
pub mod schemas;
pub mod table_models;

use crate::connectors::postgres_connector::{DbConnector, RealDbConnector};
use crate::controllers::auth_handler::*;
use crate::controllers::blog_controller::*;
use crate::controllers::login_controller::*;
use crate::controllers::register_user_controller::*;
use crate::controllers::skills_controller::*;
use crate::controllers::validate_user_controller::*;
use crate::controllers::worklog_controller::create_worklog;
use crate::controllers::worklog_controller::*;
use actix_cors::Cors;
use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use dotenv::dotenv;
use redis::aio::MultiplexedConnection;
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use std::env;
use std::sync::Arc;

// Redis session struct
#[derive(Debug, Deserialize, Serialize)]
struct SessionData {
    user_id: String,
    role: String,
}

// Function to set session in Redis asynchronously
async fn set_session_in_redis(
    redis_client: &redis::Client,
    session_id: &str,
    session_data: &SessionData,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut conn: MultiplexedConnection = redis_client.get_multiplexed_async_connection().await?;
    let session_key = format!("session:{}", session_id);

    let session_json = serde_json::to_string(session_data)?;
    conn.set_ex(session_key, session_json, 86400).await?;

    Ok(())
}

// Function to get session from Redis asynchronously
async fn get_session_from_redis(
    redis_client: &redis::Client,
    session_id: &str,
) -> Result<Option<SessionData>, Box<dyn std::error::Error>> {
    let mut conn = redis_client.get_multiplexed_async_connection().await?;
    let session_key = format!("session:{}", session_id);

    let session_json: Option<String> = conn.get(session_key).await?;
    if let Some(session_str) = session_json {
        let session_data: SessionData = serde_json::from_str(&session_str)?;
        Ok(Some(session_data))
    } else {
        Ok(None)
    }
}

// Health check endpoint
#[get("/health")]
async fn health_check() -> impl Responder {
    HttpResponse::Ok().body("OK")
}

// Test handler
#[get("/test")]
async fn test_handler() -> impl Responder {
    HttpResponse::Ok().body("This is a test")
}

use crate::middleware::rate_limiter::RateLimiter;
// Import rate limiter
use actix_web::middleware::Logger;
use actix_web::web::Data;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();

    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());

    // Initialize Redis client
    let redis_url = env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
    let redis_client = redis::Client::open(redis_url.clone()).expect("Invalid Redis URL");

    // Establish connection to the PostgreSQL database
    let pool = match RealDbConnector.establish_connection() {
        Ok(pool) => web::Data::new(pool),
        Err(e) => {
            log::error!("Failed to connect to the database: {}", e);
            return Err(std::io::Error::new(std::io::ErrorKind::Other, "Failed to connect to the database"));
        }
    };

    // Store Redis client in a web::Data container
    let redis_client_data = Data::new(redis_client.clone());

    // Set up the Actix server
    HttpServer::new(
        move || {
            let cors =
                Cors::default()
                    // .allow_any_origin()
                    .allowed_origin("http://localhost:3000") // Adjust frontend URL
                    .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
                    .allow_any_header()
                    .supports_credentials() // for now keep this commented out until I add more auth
                    .max_age(3600);


            App::new()
                .wrap(Logger::default()) // Add logging middleware
                .wrap(cors) // No need for Arc, create a new instance of Cors
                .app_data(pool.clone()) // Pass the PostgreSQL connection pool to handlers
                .app_data(redis_client_data.clone()) // Pass the Redis client to handlers
                .wrap(RateLimiter::new(redis_client_data.clone(), 300, 60)) // Rate limiter
                .service(health_check)
                .service(test_handler)
                .service(create_post)
                .service(get_post)
                .service(get_by_post_id)
                .service(get_all_posts)
                .service(update_post)
                .service(delete_post)
                .service(delete_all_posts)
                .service(create_worklog)
                .service(get_worklog)
                .service(get_by_worklog_id)
                .service(get_all_worklog)
                .service(update_worklog)
                .service(delete_worklog)
                .service(create_skill)
                .service(get_skill)
                .service(get_by_skill_id)
                .service(update_skill)
                .service(delete_skill)
                .service(get_all_skills)
                .service(create_user)
                .service(login)
                .service(logout)
                .service(get_user_role)
                .service(check_username)
                .service(check_email)
        })
        .bind(format!("0.0.0.0:{}", port))?
        .run()
        .await
}
