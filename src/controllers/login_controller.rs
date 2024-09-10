use actix_web::cookie::Cookie;
use actix_web::{post, web, Error, HttpRequest, HttpResponse};
use bcrypt::verify;
use diesel::prelude::*;
use diesel::{PgConnection, QueryDsl, RunQueryDsl};
use redis::AsyncCommands;
use uuid::Uuid;

use diesel::result::Error as DieselError;

pub fn get_user_by_username(conn: &mut PgConnection, user_name: &str) -> Result<Option<Users>, DieselError> {
    users.filter(username.eq(user_name))
        .first::<Users>(conn)
        .optional()  // This will return Ok(None) if no user is found
}

pub fn get_user_by_user_id(conn: &mut PgConnection, userId: &str) -> Result<Option<Users>, DieselError> {
    users.filter(user_id.eq(userId))
        .first::<Users>(conn)
        .optional()  // This will return Ok(None) if no user is found
}


#[post("/login")]
async fn login(
    pool: web::Data<DbPool>,
    redis_client: web::Data<redis::Client>,
    login_data: web::Json<LoginRequest>,
) -> Result<HttpResponse, Error> {
    let username_clone = login_data.username.clone(); // Clone username for use inside the closure

    let mut conn = pool.get().map_err(|_| {
        actix_web::error::ErrorInternalServerError("Failed to get DB connection")
    })?;

    // Find user by username
    let user =
        web::block(move || get_user_by_username(&mut conn, &username_clone))
            .await
            .map_err(|_| actix_web::error::ErrorUnauthorized("Invalid credentials"))?;

    // Check if user exists and verify password
    if let Ok(Some(user)) = user {
        if verify(&login_data.password, &user.password_hash).map_err(|_| {
            actix_web::error::ErrorUnauthorized("Invalid credentials")
        })? {
            // Generate a session ID
            let session_id = Uuid::new_v4().to_string();

            // Store session in Redis
            let session_data =
                SessionData {
                    user_id: user.user_id.clone(),
                    role: user.user_type.clone(),
                };

            let mut redis_conn = redis_client
                .get_async_connection()
                .await
                .map_err(|_| actix_web::error::ErrorInternalServerError("Failed to connect to Redis"))?;

            let session_key = format!("session:{}", session_id);

            let session_value = serde_json::to_string(&session_data)
                .map_err(|_| actix_web::error::ErrorInternalServerError("Failed to serialize session data"))?;

            redis_conn.set_ex(session_key, session_value, 3600).await.map_err(|_| {
                actix_web::error::ErrorInternalServerError("Failed to store session in Redis")
            })?;

            // Set session ID as cookie
            let cookie =
                Cookie::build("session_id", session_id)
                    .path("/")
                    // .secure(true)  // Only send over HTTPS  doesnt work on localhost
                    .http_only(true)
                    .finish();

            Ok(HttpResponse::Ok()
                .cookie(cookie)
                .body("Login successful"))
        } else {
            Ok(HttpResponse::Unauthorized().body("Invalid credentials"))
        }
    } else {
        Ok(HttpResponse::Unauthorized().body("User not found"))
    }
}

async fn check_user_session(
    redis_client: web::Data<redis::Client>,
    session_id: &str,
) -> Result<SessionData, HttpResponse> {
    let mut redis_conn = redis_client
        .get_async_connection()
        .await
        .map_err(|_| HttpResponse::InternalServerError().body("Failed to connect to Redis"))?;

    let session_key = format!("session:{}", session_id);
    let session_json: Option<String> = redis_conn.get(session_key).await.map_err(|_| {
        HttpResponse::InternalServerError().body("Failed to fetch session from Redis")
    })?;

    if let Some(session_str) = session_json {
        let session_data: SessionData = serde_json::from_str(&session_str).unwrap();
        Ok(session_data)
    } else {
        Err(HttpResponse::Unauthorized().body("Invalid or expired session"))
    }
}

use actix_web::error::InternalError;
use actix_web::{App, HttpServer};
use dotenv::dotenv;
use log::error;
use std::env;
use crate::connectors::postgres_connector::DbPool;
use crate::models::LoginRequest::LoginRequest;
use crate::models::LogoutResponse::LogoutResponse;
use crate::models::SessionData::SessionData;
use crate::schemas::user_schema::users::dsl::users;
use crate::schemas::user_schema::users::{user_id, username};
use crate::table_models::users::Users;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());

    let redis_url = env::var("REDIS_URL").expect("REDIS_URL must be set");
    let redis_client = redis::Client::open(redis_url).expect("Invalid Redis URL");

    HttpServer::new(move || {
        App::new()
            .data(redis_client.clone()) // Add Redis client
        // Add routes here
    })
        .bind(format!("0.0.0.0:{}", port))?
        .run()
        .await
}

#[post("/logout")]
async fn logout(
    pool: web::Data<DbPool>,
    redis_client: web::Data<redis::Client>,
    req: HttpRequest,
) -> Result<HttpResponse, actix_web::Error> {
    // Get the session ID from the cookies
    let session_id = match req.cookie("session_id") {
        Some(cookie) => cookie.value().to_string(),
        None => {
            log::warn!("Session ID not found in cookies");
            return Ok(HttpResponse::Unauthorized().body("Session ID not found"));
        }
    };

    // Connect to Redis
    let mut redis_conn = redis_client
        .get_async_connection()
        .await
        .map_err(|err| {
            log::error!("Failed to connect to Redis: {:?}", err);
            InternalError::from_response(
                "Failed to connect to Redis",
                HttpResponse::InternalServerError().finish(),
            )
        })?;

    // Fetch session data from Redis
    let session_key = format!("session:{}", session_id);
    let session_data: Option<String> = redis_conn
        .get(&session_key)
        .await
        .map_err(|err| {
            log::error!("Failed to fetch session data from Redis: {:?}", err);
            InternalError::from_response(
                "Failed to fetch session data",
                HttpResponse::InternalServerError().finish(),
            )
        })?;

    // If no session data is found, return an error
    let session_data = match session_data {
        Some(data) => data,
        None => {
            log::warn!("Session not found or expired for session ID: {}", session_id);
            return Ok(HttpResponse::Unauthorized().body("Session expired or not found"));
        }
    };

    // Parse session data to extract the user ID
    let session_json: serde_json::Value = serde_json::from_str(&session_data).map_err(|err| {
        log::error!("Failed to parse session data: {:?}", err);
        InternalError::from_response(
            "Failed to parse session data",
            HttpResponse::InternalServerError().finish(),
        )
    })?;

    // Extract and clone the user_id as an owned String
    let user_id_from_redis_cache = session_json["user_id"]
        .as_str()
        .ok_or_else(|| {
            InternalError::from_response(
                "user_id not found in session data",
                HttpResponse::InternalServerError().finish(),
            )
        })?
        .to_string(); // Clone to get an owned String

    // Fetch user info from the database based on user_id
    let mut conn = pool.get().map_err(|_| {
        log::error!("Failed to get DB connection");
        actix_web::error::ErrorInternalServerError("Failed to get DB connection")
    })?;

    let user_name_result = web::block({
        let user_id_clone = user_id_from_redis_cache.clone(); // Clone the value to use in the closure
        move || get_user_by_user_id(&mut conn, &user_id_clone)
    })
        .await
        .map_err(|err| {
            log::error!("Failed to fetch user from database: {:?}", err);
            actix_web::error::ErrorUnauthorized("Invalid credentials")
        })?;

    let logout_response =
        match user_name_result {
            Ok(Some(user)) =>
                LogoutResponse {
                    username: user.username.clone(),
                    message: format!("Successfully logged out: {}", user.username),
                },
            Ok(None) => {
                log::error!("User not found in the database for user_id: {}", user_id_from_redis_cache);
                return Ok(HttpResponse::Unauthorized().body("User not found"));
            }
            Err(err) => {
                log::error!("Error occurred while fetching user: {:?}", err);
                return Err(InternalError::from_response(
                    format!("Error occurred: {}", err),
                    HttpResponse::InternalServerError().finish(),
                ).into());
            }
        };

    let user_name_clone = logout_response.username.clone();

    // Delete the session in Redis
    let delete_result: Result<(), _> = redis_conn.del(&session_key).await;

    match delete_result {
        Ok(_) => {
            // Create the response and delete the cookie
            let mut response = HttpResponse::Ok().json(logout_response);
            response.del_cookie("session_id");
            log::info!("User {} logged out successfully", user_name_clone);
            Ok(response)
        }
        Err(err) => {
            log::error!("Failed to delete session from Redis: {:?}", err);
            Ok(HttpResponse::InternalServerError().body("Failed to delete session"))
        }
    }
}
