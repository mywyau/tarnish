use actix_web::{post, web, Error, HttpResponse};
use bcrypt::verify;
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};

use crate::models::users::{get_user_by_username, Users};
// Import your user model and method
use crate::schemas::user_schema::users;
// Your Diesel schema

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

// Define the database connection pool type
type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;

// Define the secret for signing the JWT (use a more secure method in production)
const SECRET_KEY: &[u8] = b"mysecret";

fn init_env() {
    dotenv().ok(); // Load .env variables into the environment
}

// Helper function to generate JWT token
fn generate_jwt(username: &str, role: &str) -> Result<String, Error> {
    init_env(); // Initialize environment to load variables

    let secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");


    let expiration = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::hours(24))
        .expect("valid timestamp")
        .timestamp() as usize;

    let jwtClaims = JWTClaims {
        sub: username.to_owned(),
        role: role.to_owned(),
        exp: expiration,
    };

    let token = encode(
        &Header::default(),
        &jwtClaims,
        &EncodingKey::from_secret(secret.as_ref()),
    )
        .map_err(|_| HttpResponse::InternalServerError().body("Token creation error"))?;

    Ok(token)
}

use crate::table_models::users::Users;
use diesel::prelude::*;
// Your user model

pub fn get_user_by_username(conn: &PgConnection, username: &str) -> Result<Option<Users>, diesel::result::Error> {
    use crate::schemas::user_schema::users::dsl::*;
    users.filter(username.eq(username))
        .first::<Users>(conn)
        .optional()
}

// Define the POST /login endpoint
#[post("/login")]
async fn login(
    pool: web::Data<DbPool>,
    login_data: web::Json<LoginRequest>,
) -> Result<HttpResponse, Error> {
    let conn = pool.get().map_err(|_| HttpResponse::InternalServerError().body("Database error"))?;

    // Find user by username
    let user = web::block(move || get_user_by_username(&conn, &login_data.username))
        .await
        .map_err(|_| HttpResponse::Unauthorized().body("Invalid credentials"))?;

    // Check if user exists and verify password
    if let Some(user) = user {
        if verify(&login_data.password, &user.password_hash).map_err(|_| HttpResponse::Unauthorized().body("Invalid credentials"))? {
            // Generate JWT token if credentials are correct
            let token = generate_jwt(&user.username, &user.role)?;

            Ok(HttpResponse::Ok().json(token))  // Return the token in the response
        } else {
            Err(HttpResponse::Unauthorized().body("Invalid credentials").into())
        }
    } else {
        Err(HttpResponse::Unauthorized().body("User not found").into())
    }
}


