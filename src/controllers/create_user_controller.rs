use crate::models::{NewUsers, UserType, Users};
use diesel::prelude::*;


use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct CreateUserInput {
    pub username: String,
    pub password: String,
    pub email: String,
    pub user_type: String, // admin, editor, viewer
}


use crate::models::{NewUsers, UserType, Users};
use crate::schema::users;
use actix_web::{post, web, Error, HttpResponse};
use bcrypt::{hash, DEFAULT_COST};
use chrono::Utc;
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};

// Define the database connection pool type
type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;

// Define the POST /users endpoint
#[post("/users")]
async fn create_user(
    pool: web::Data<DbPool>,
    user_input: web::Json<CreateUserInput>,
) -> Result<HttpResponse, Error> {
    let conn = pool.get().map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Couldn't get db connection from pool: {}", e))
    })?;

    // Hash the password using bcrypt
    let password_hash = hash(&user_input.password, DEFAULT_COST)
        .map_err(|e| actix_web::error::ErrorInternalServerError(format!("Password hashing failed: {}", e)))?;

    // Convert the user_type string to the UserType enum
    let user_type_enum = match user_input.user_type.as_str() {
        "admin" => UserType::Admin,
        "editor" => UserType::Editor,
        "viewer" => UserType::Viewer,
        _ => return Err(actix_web::error::ErrorBadRequest("Invalid user type")),
    };

    // Create a new user struct
    let new_user =
        NewUsers {
            username: user_input.username.clone(),
            password_hash,
            email: user_input.email.clone(),
            role_id: 1, // Role id, adjust as needed
            user_type: user_type_enum,
            created_at: Utc::now().naive_utc(),
            updated_at: Utc::now().naive_utc(),
        };

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
            actix_web::error::ErrorInternalServerError(format!("Failed to insert user: {}", e))
        })?;

    Ok(HttpResponse::Created().json(inserted_user))
}
