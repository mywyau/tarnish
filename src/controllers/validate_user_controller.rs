use crate::schemas::user_schema::users;
use crate::table_models::users::Users;
use actix_web::{get, web, Error, HttpResponse};
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use log::{error, info};

// Define the database connection pool type
type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;

// Struct for the API response
#[derive(serde::Serialize)]
struct CheckResponse {
    exists: bool,
}

// Struct to parse query parameters for email
#[derive(Debug, serde::Deserialize)]
struct EmailQuery {
    email: String,
}

// Check if email exists
#[get("/api/check-email")]
async fn check_email(
    pool: web::Data<DbPool>,
    query: web::Query<EmailQuery>, // Using structured query params
) -> Result<HttpResponse, Error> {
    let email = query.email.clone(); // Clone the email so it can be moved into the async block

    info!("Checking if email '{}' exists", email);

    // Get database connection
    let mut conn = pool.get().map_err(|e| {
        error!("Couldn't get db connection from pool: {}", e);
        actix_web::error::ErrorInternalServerError("Failed to retrieve connection from pool")
    })?;

    // Check if the email exists
    let exists = web::block(move || {
        users::table
            .filter(users::email.eq(email))
            .first::<Users>(&mut conn)
            .optional()
            .map(|user| user.is_some()) // If a user exists, return true
    })
        .await
        .map_err(|e| {
            error!("Database query failed for email: {}", e);
            actix_web::error::ErrorInternalServerError("Database query failed")
        })?;

    // Handle the Result
    match exists {
        Ok(exists_value) => {
            Ok(HttpResponse::Ok().json(CheckResponse { exists: exists_value }))
        }
        Err(e) => {
            error!("Database query failed for email: {}", e);
            Err(actix_web::error::ErrorInternalServerError("Database query failed"))
        }
    }
}
