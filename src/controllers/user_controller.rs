use crate::schemas::user_schema::users;
use crate::table_models::users::{NewUsers, Users};

use actix_web::{post, web, Error, HttpResponse};

use bcrypt::{hash, DEFAULT_COST};
use chrono::DateTime;
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use diesel::result::Error as DieselError;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct CreateUserInput {
    pub role_id: String,
    pub username: String,
    pub password: String,
    pub email: String,
    pub user_type: String, // admin, editor, viewer
    pub created_at: String,
    pub updated_at: String,
}


// Define the database connection pool type
type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;

// Define the POST /users endpoint
#[post("/users")]
async fn create_user(
    pool: web::Data<DbPool>,
    user_input: web::Json<CreateUserInput>,
) -> Result<HttpResponse, Error> {
    let mut conn = pool.get().map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Couldn't get db connection from pool: {}", e))
    })?;

    // Hash the password using bcrypt
    let hashed_password = hash(&user_input.password, DEFAULT_COST)
        .map_err(|e| actix_web::error::ErrorInternalServerError(format!("Password hashing failed: {}", e)))?;

    // Create a new user struct
    let new_user =
        NewUsers {
            username: user_input.username.clone(),
            password_hash: hashed_password,
            email: user_input.email.clone(),
            role_id: user_input.role_id.clone(), // Role id, adjust as needed
            user_type: user_input.user_type.clone(),
            created_at: DateTime::parse_from_rfc3339(&user_input.created_at)
                .unwrap()
                .naive_utc(), // Convert to NaiveDateTime
            updated_at: DateTime::parse_from_rfc3339(&user_input.updated_at)
                .unwrap()
                .naive_utc(), // Convert to NaiveDateTime
        };

    // Insert the new user into the database using a transaction
    let inserted_user =
        web::block(move || {
            conn.transaction::<_, diesel::result::Error, _>(|conn| {
                diesel::insert_into(users::table)
                    .values(&new_user)
                    .get_result::<Users>(conn)
            })
        })
            .await
            .map_err(|e| {
                actix_web::error::ErrorInternalServerError(format!("Failed to insert user: {}", e))
            })?;

    match inserted_user {
        Ok(user) => Ok(HttpResponse::Created().json(user)),
        Err(DieselError::DatabaseError(_, info)) => {
            eprintln!("Database error: {:?}", info);
            Ok(HttpResponse::BadRequest().body(format!("Database error: {}", info.message())))
        }
        Err(e) => {
            eprintln!("Error inserting user: {:?}", e);
            Ok(HttpResponse::InternalServerError().body("Internal server error"))
        }
    }
}
