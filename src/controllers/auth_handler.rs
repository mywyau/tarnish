use crate::models::UserRoleResponse::UserRoleResponse;
use actix_web::error::InternalError;
use actix_web::{get, web, Error, HttpRequest, HttpResponse};
use diesel::prelude::*;
use log::error;
use redis::{AsyncCommands, Client};
// Add logging for errors

// Define your Redis session check handler
#[get("/get-user-role")]
async fn get_user_role(
    redis_client: web::Data<Client>,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {

    // Get session ID from cookies
    let session_id_cookie = req.cookie("session_id");
    let session_id =
        match session_id_cookie {
            Some(cookie) => cookie.value().to_string(),
            None => {
                error!("Session not found in cookies");
                return Err(InternalError::from_response(
                    "Session not found",
                    HttpResponse::Unauthorized().finish(),
                )
                    .into());
            }
        };

    // Connect to Redis and get session data
    let mut redis_conn =
        redis_client
            .get_async_connection()
            .await
            .map_err(|err| {
                error!("Failed to connect to Redis: {:?}", err);
                InternalError::from_response(
                    "Failed to connect to Redis",
                    HttpResponse::InternalServerError().finish(),
                )
            })?;

    let session_key = format!("session:{}", session_id);

    // Fetch session data from Redis
    let session_data: Option<String> = redis_conn.get(&session_key).await.map_err(|err| {
        error!("Failed to fetch session from Redis: {:?}", err);
        InternalError::from_response(
            "Failed to fetch session",
            HttpResponse::InternalServerError().finish(),
        )
    })?;

    // If session exists, return user role
    if let Some(data) =
        session_data {
        // Log the session data
        log::info!("Session data retrieved: {}", data);

        // Parse the session data from Redis
        let user_role: serde_json::Value =
            serde_json::from_str(&data).map_err(|err| {
                error!("Failed to parse session data: {:?}", err);
                InternalError::from_response(
                    "Failed to parse session data",
                    HttpResponse::InternalServerError().finish(),
                )
            })?;

        // Extract the role from the session data
        let role = user_role["role"].as_str().unwrap_or("viewer");

        // Create a JSON response that includes the user role
        let response =
            UserRoleResponse {
                role: role.to_string(),
                message: format!("User role is {}", role),
            };

        // Return the JSON response
        Ok(HttpResponse::Ok().json(response))
    } else {
        error!("Session expired or not found in Redis");
        Err(InternalError::from_response(
            "Session expired",
            HttpResponse::Unauthorized().finish(),
        )
            .into())
    }
}
