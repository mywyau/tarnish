use crate::schemas::user_schema::users;
use crate::table_models::users::{NewUsers, Users};

use actix_web::{post, web, Error, HttpResponse};
use bcrypt::{hash, DEFAULT_COST};
use chrono::DateTime;
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use diesel::result::Error as DieselError;
use log::{debug, error, info}; // Import log macros

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct CreateUserInput {
    pub user_id: String,
    pub username: String,
    pub password: String,
    pub email: String,
    pub user_type: String, // admin, editor, viewer
    pub created_at: String,
    pub updated_at: String,
}

// Define the database connection pool type
type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;

// Define the POST /create/account/user endpoint
#[post("/create/account/user")]
async fn create_user(
    pool: web::Data<DbPool>,
    user_input: web::Json<CreateUserInput>,
) -> Result<HttpResponse, Error> {

    // Log incoming request for user creation
    info!("Received request to create user: {:?}", user_input.username);

    // Get a connection from the pool
    let mut conn = pool.get().map_err(|e| {
        error!("Couldn't get db connection from pool: {}", e);
        actix_web::error::ErrorInternalServerError("Failed to retrieve connection from pool")
    })?;

    // Hash the password using bcrypt
    let hashed_password = hash(&user_input.password, DEFAULT_COST).map_err(|e| {
        error!("Password hashing failed: {}", e);
        actix_web::error::ErrorInternalServerError("Failed to hash password")
    })?;

    // Log password hashing success
    debug!("Password successfully hashed for user: {}", user_input.username);

    // Create a new user struct
    let new_user = NewUsers {
        username: user_input.username.clone(),
        password_hash: hashed_password,
        email: user_input.email.clone(),
        user_id: user_input.user_id.clone(),
        user_type: user_input.user_type.clone(),
        created_at: DateTime::parse_from_rfc3339(&user_input.created_at)
            .unwrap()
            .naive_utc(), // Convert to NaiveDateTime
        updated_at: DateTime::parse_from_rfc3339(&user_input.updated_at)
            .unwrap()
            .naive_utc(), // Convert to NaiveDateTime
    };

    // Log user struct creation
    debug!("New user struct created: {:?}", new_user.username);

    // Insert the new user into the database using a transaction
    let inserted_user = web::block(move || {
        conn.transaction::<_, diesel::result::Error, _>(|conn| {
            diesel::insert_into(users::table)
                .values(&new_user)
                .get_result::<Users>(conn)
        })
    })
        .await
        .map_err(|e| {
            error!("Failed to insert user into the database: {}", e);
            actix_web::error::ErrorInternalServerError("Failed to insert user into database")
        })?;

    // Check for the result of the database operation
    match inserted_user {
        Ok(user) => {
            info!("User {} created successfully", user.username);
            Ok(HttpResponse::Created().json(user))
        }
        Err(DieselError::DatabaseError(_, info)) => {
            error!("Database error: {:?}", info);
            Ok(HttpResponse::BadRequest().body(format!("Database error: {}", info.message())))
        }
        Err(e) => {
            error!("Error inserting user: {:?}", e);
            Ok(HttpResponse::InternalServerError().body("Internal server error"))
        }
    }
}
