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

#[post("/blog/post/create")]
async fn create_post(
    pool: web::Data<DbPool>,
    post: web::Json<PostInput>,
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


#[get("/blog/post/retrieve/{id}")]
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

#[put("/blog/posts/update/{id}")]
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

#[delete("/blog/post/single/{id}")]
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

#[delete("/blog/post/all")]
async fn delete_all_posts(
    pool: web::Data<DbPool>,
) -> Result<HttpResponse, Error> {
    let mut conn = pool.get().expect("Couldn't get db connection from pool");

    // Execute the DELETE statement
    match diesel::sql_query("DELETE FROM posts")
        .execute(&mut conn)
    {
        Ok(_) => Ok(HttpResponse::NoContent().finish()), // No content returned after successful deletion
        Err(e) => {
            eprintln!("Error deleting posts: {:?}", e);
            Ok(HttpResponse::InternalServerError().finish())
        }
    }
}

use diesel::prelude::*;
use serde_json::json; // Import `json` macro for creating JSON responses

#[delete("/blog/post/all/message")]
async fn delete_all_posts_with_body(
    pool: web::Data<DbPool>,
) -> Result<HttpResponse, Error> {

    let mut conn = pool.get().expect("Couldn't get db connection from pool");
    // Execute the DELETE statement
    match diesel::sql_query("DELETE FROM posts")
        .execute(&mut conn)
    {
        Ok(_) => {
            // Construct a JSON response body with a message
            let response_body = json!({
                "message": "All posts have been deleted."
            });

            Ok(HttpResponse::Ok()
                .content_type("application/json")
                .json(response_body))
        }
        Err(e) => {
            eprintln!("Error deleting posts: {:?}", e);
            Ok(HttpResponse::InternalServerError().finish())
        }
    }
}
