use actix_cors::Cors;
use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use dotenv::dotenv;
use redis::aio::Connection;
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use std::env;
use tarnish::connectors::postgres_connector::{DbConnector, RealDbConnector};
use tarnish::controllers::auth_handler::get_user_role;
use tarnish::controllers::blog_controller::{
    create_post, delete_all_posts, delete_post, get_all_posts, get_by_post_id, get_post, update_post,
};
use tarnish::controllers::login_controller::{login, logout};
use tarnish::controllers::register_user_controller::create_user;
use tarnish::controllers::skills_controller::{
    create_skill, delete_skill, get_all_skills, get_by_skill_id, get_skill, update_skill,
};
use tarnish::controllers::worklog_controller::{
    create_worklog, delete_worklog, get_all_worklog, get_by_worklog_id, get_worklog, update_worklog,
};

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
) -> Result<(), redis::RedisError> {
    let mut conn: Connection = redis_client.get_async_connection().await?; // Use get_async_connection
    let session_key = format!("session:{}", session_id);
    let session_json = serde_json::to_string(session_data).unwrap();

    // Set session data in Redis with an expiration time
    conn.set_ex(session_key, session_json, 86400).await?; // 86400 seconds = 1 day

    Ok(())
}

// Function to get session from Redis asynchronously
async fn get_session_from_redis(
    redis_client: &redis::Client,
    session_id: &str,
) -> Result<Option<SessionData>, redis::RedisError> {
    let mut conn: Connection = redis_client.get_async_connection().await?; // Use get_async_connection
    let session_key = format!("session:{}", session_id);

    let session_json: Option<String> = conn.get(session_key).await?;
    if let Some(session_str) = session_json {
        let session_data: SessionData = serde_json::from_str(&session_str).unwrap();
        Ok(Some(session_data))
    } else {
        Ok(None)
    }
}

// Define a simple health check endpoint
#[get("/health")]
async fn health_check() -> impl Responder {
    HttpResponse::Ok().body("OK")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());

    // Initialize Redis client
    let redis_url = env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
    let redis_client = redis::Client::open(redis_url.clone()).expect("Invalid Redis URL");

    // Establish connection to the PostgreSQL database
    let pool = match RealDbConnector.establish_connection() {
        Ok(pool) => web::Data::new(pool),
        Err(e) => {
            eprintln!("Failed to connect to the database: {}", e);
            return Err(std::io::Error::new(std::io::ErrorKind::Other, "Failed to connect to the database"));
        }
    };

    let redis_client_data = web::Data::new(redis_client);

    HttpServer::new(move || {
        App::new()
            .wrap(
                Cors::default()
                    .allow_any_origin()
                    .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
                    .allow_any_header()
                    .supports_credentials()
                    .max_age(3600),
            )
            .app_data(pool.clone()) // Passing the PostgreSQL connection pool to handlers
            .app_data(redis_client_data.clone()) // Pass the Redis client to handlers
            .service(health_check)

            // Blog Post Endpoints
            .service(create_post)
            .service(get_post)
            .service(get_by_post_id)
            .service(get_all_posts)
            .service(update_post)
            .service(delete_post)
            .service(delete_all_posts)

            // Worklog Endpoints
            .service(create_worklog)
            .service(get_worklog)
            .service(get_by_worklog_id)
            .service(get_all_worklog)
            .service(update_worklog)
            .service(delete_worklog)

            // Skills Endpoints
            .service(create_skill)
            .service(get_skill)
            .service(get_by_skill_id)
            .service(update_skill)
            .service(delete_skill)
            .service(get_all_skills)

            // User Registration Endpoints
            .service(create_user)

            // User Login Endpoints
            .service(login)
            .service(logout)
            .service(get_user_role)
    })
        .bind(format!("0.0.0.0:{}", port))?
        .run()
        .await
}
