use actix_web::cookie::Cookie;
use actix_web::{get, post, web, Error, HttpRequest, HttpResponse};
use bcrypt::verify;
use diesel::prelude::*;
use diesel::{PgConnection, QueryDsl, RunQueryDsl};
use redis::{AsyncCommands, Client};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::schemas::user_schema::users::dsl::*;
use crate::table_models::users::Users;
use crate::DbPool;
use diesel::result::Error as DieselError;

// JWT Claims Struct
#[derive(Debug, Serialize, Deserialize)]
struct JWTClaims {
    sub: String, // Subject (username)
    role: String,
    exp: usize,  // Expiration time as a timestamp
}

// Login Request Payload
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

// Session Data to store in Redis
#[derive(Debug, Serialize, Deserialize)]
pub struct SessionData {
    pub user_id: String,
    pub role: String, // admin, viewer, etc.
}

// Function to get user by username
pub fn get_user_by_username(conn: &mut PgConnection, user_name: &str) -> Result<Option<Users>, DieselError> {
    users.filter(username.eq(user_name))
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
            let session_data = SessionData {
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

            redis_conn.set_ex(session_key, session_value, 86400).await.map_err(|_| {
                actix_web::error::ErrorInternalServerError("Failed to store session in Redis")
            })?;

            // Set session ID as cookie
            let cookie = Cookie::build("session_id", session_id)
                .path("/")
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

#[get("/admin/only")]
async fn admin_only(
    redis_client: web::Data<redis::Client>,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    let session_id_cookie = req.cookie("session_id").ok_or_else(|| {
        actix_web::error::ErrorUnauthorized("No session cookie found")
    })?;
    let session_id = session_id_cookie.value();

    let session_data = check_user_session(redis_client, session_id).await.map_err(|e| {
        actix_web::error::ErrorUnauthorized("Unauthorized")
    })?;

    // Verify that the user is an admin
    if session_data.role == "admin" {
        Ok(HttpResponse::Ok().body("Welcome admin!"))
    } else {
        Ok(HttpResponse::Forbidden().body("Access restricted to admins only"))
    }
}

use actix_web::{App, HttpServer};
use dotenv::dotenv;
use std::env;

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


// // Define restricted content structure
// #[derive(Debug, Serialize)]
// pub struct RestrictedContent {
//     pub content: String,
// }
//
// // Route to check the user type
// #[get("/check-role")]
// async fn check_role(
//     redis_client: web::Data<Client>,
//     req: HttpRequest,
// ) -> Result<HttpResponse, Error> {
//
//     // Extract session ID from the request cookie
//     let session_id = req
//         .cookie("session_id")
//         .map(|cookie| cookie.value().to_string())
//         .ok_or_else(|| {
//             InternalError::from_response(
//                 "No session ID",
//                 HttpResponse::Unauthorized().finish(),
//             )
//                 .into()
//         })?;
//
//     // Fetch session data from Redis
//     let mut redis_conn = redis_client.get_async_connection().await.map_err(|_| {
//         InternalError::from_response(
//             "Failed to connect to Redis",
//             HttpResponse::InternalServerError().finish(),
//         )
//             .into()
//     })?;
//
//     let session_key = format!("session:{}", session_id);
//     let session_json: Option<String> = redis_conn.get(session_key).await.map_err(|_| {
//         InternalError::from_response(
//             "Failed to get session data",
//             HttpResponse::InternalServerError().finish(),
//         )
//             .into()
//     })?;
//
//     // Check if session exists and deserialize
//     if let Some(session_str) = session_json {
//         let session_data: SessionData = serde_json::from_str(&session_str).map_err(|_| {
//             InternalError::from_response(
//                 "Failed to parse session data",
//                 HttpResponse::InternalServerError().finish(),
//             )
//                 .into()
//         })?;
//
//         // Verify if the user is an admin
//         if session_data.role == "admin" {
//             let restricted_content = RestrictedContent {
//                 content: "This is admin-only content.".to_string(),
//             };
//             return Ok(HttpResponse::Ok().json(restricted_content));
//         } else {
//             return Ok(HttpResponse::Forbidden().body("Access restricted to admins only"));
//         }
//     } else {
//         Ok(HttpResponse::Unauthorized().body("Invalid session"))
//     }
// }

use actix_web::error::InternalError;

// Define your Redis session check handler
#[get("/api/get-user-role")]
async fn get_user_role(
    redis_client: web::Data<Client>,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    // Get session ID from cookies
    let session_id_cookie = req.cookie("session_id");
    let session_id = match session_id_cookie {
        Some(cookie) => cookie.value().to_string(),
        None => {
            return Err(InternalError::from_response(
                "Session not found",
                HttpResponse::Unauthorized().finish(),
            )
                .into())
        }
    };

    // Connect to Redis and get session data
    let mut redis_conn = redis_client
        .get_async_connection()
        .await
        .map_err(|_| InternalError::from_response(
            "Failed to connect to Redis",
            HttpResponse::InternalServerError().finish(),
        ))?;

    let session_key = format!("session:{}", session_id);
    let session_data: Option<String> = redis_conn.get(&session_key).await.map_err(|_| {
        InternalError::from_response(
            "Failed to fetch session",
            HttpResponse::InternalServerError().finish(),
        )
    })?;

    // If session exists, return user role
    if let Some(data) = session_data {
        let user_role: serde_json::Value = serde_json::from_str(&data).map_err(|_| {
            InternalError::from_response(
                "Failed to parse session data",
                HttpResponse::InternalServerError().finish(),
            )
        })?;
        let role = user_role["role"].as_str().unwrap_or("viewer");
        Ok(HttpResponse::Ok().json(role))
    } else {
        Err(InternalError::from_response(
            "Session expired",
            HttpResponse::Unauthorized().finish(),
        )
            .into())
    }
}
