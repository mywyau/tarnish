use std::env;

use actix_web::{delete, Error, get, HttpResponse, post, put, web};
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use dotenv::dotenv;
use serde::{Deserialize, Serialize};

// Import schema
use crate::models::{NewPost, Post};
use crate::schema::posts;

pub type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;

pub fn establish_connection() -> DbPool {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    r2d2::Pool::builder().build(manager).expect("Failed to create pool.")
}

#[derive(Serialize, Deserialize)]
struct PostInput {
    title: String,
    body: String,
}

#[post("/posts")]
async fn create_post(
    pool: web::Data<DbPool>,
    post: web::Json<PostInput>
) -> Result<HttpResponse, actix_web::Error> {
    let new_post = NewPost {
        title: post.title.clone(),
        body: post.body.clone(),
    };

    let mut conn = pool.get().map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Couldn't get db connection from pool: {}", e))
    })?;

    conn.transaction::<_, diesel::result::Error, _>(|conn| {
        // Check if the table is empty
        let post_count: i64 = posts::table.count().get_result(conn)?;
        if post_count == 0 {
            // Reset the sequence
            diesel::sql_query("ALTER SEQUENCE posts_id_seq RESTART WITH 1").execute(conn)?;
        }

        // Insert the new post
        diesel::insert_into(posts::table)
            .values(&new_post)
            .get_result::<Post>(conn)
            .map_err(|e| {
                eprintln!("Error inserting new post: {:?}", e);
                e
            })
    }).map_err(|e| actix_web::error::ErrorInternalServerError(format!("Transaction failed: {}", e)))
        .map(|post| HttpResponse::Created().json(post))
}


#[get("/posts/{id}")]
async fn get_post(
    path: web::Path<i32>,
    pool: web::Data<DbPool>,
) -> Result<HttpResponse, Error> {
    let id = path.into_inner();
    let conn = pool.get().expect("Couldn't get db connection from pool");

    // Convert to mutable reference
    let mut conn = conn;

    match posts::table.find(id).first::<Post>(&mut conn) { // Use mutable reference
        Ok(post) => Ok(HttpResponse::Ok().json(post)),
        Err(_) => Ok(HttpResponse::NotFound().finish()),
    }
}

#[put("/posts/{id}")]
async fn update_post(
    path: web::Path<i32>,
    post: web::Json<PostInput>,
    pool: web::Data<DbPool>,
) -> Result<HttpResponse, Error> {
    let id = path.into_inner();
    let post_input = post.into_inner();

    let conn = pool.get().expect("Couldn't get db connection from pool");

    // Convert to mutable reference
    let mut conn = conn;

    match diesel::update(posts::table.find(id))
        .set((
            posts::title.eq(post_input.title),
            posts::body.eq(post_input.body)
        ))
        .execute(&mut conn) // Use mutable reference
    {
        Ok(_) => Ok(HttpResponse::Ok().finish()),
        Err(e) => {
            eprintln!("Error updating post: {:?}", e);
            Ok(HttpResponse::InternalServerError().finish())
        }
    }
}

#[delete("/posts/{id}")]
async fn delete_post(
    path: web::Path<i32>,
    pool: web::Data<DbPool>,
) -> Result<HttpResponse, Error> {
    let id = path.into_inner();
    let conn = pool.get().expect("Couldn't get db connection from pool");

    // Convert to mutable reference
    let mut conn = conn;

    match diesel::delete(posts::table.find(id)).execute(&mut conn) { // Use mutable reference
        Ok(_) => Ok(HttpResponse::Ok().finish()),
        Err(e) => {
            eprintln!("Error deleting post: {:?}", e);
            Ok(HttpResponse::InternalServerError().finish())
        }
    }
}