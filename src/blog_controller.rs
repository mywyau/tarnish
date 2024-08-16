use std::env;

use actix_web::{delete, Error, get, HttpResponse, post, put, web};
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use dotenv::dotenv;
use serde::{Deserialize, Serialize};
use serde_json::json;

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
pub struct PostInput {
    pub id: i32,
    pub post_id: String,
    pub title: String,
    pub body: String,
}

impl PostInput {
    // Constructor method for creating a new PostInput
    pub fn new(id: i32, post_id: String, title: String, body: String) -> Self {
        PostInput {
            id,
            post_id,
            title,
            body,
        }
    }
}

#[post("/blog/post/create")]
async fn create_post(
    pool: web::Data<DbPool>,
    post: web::Json<PostInput>,
) -> Result<HttpResponse, Error> {
    let post_input = post.into_inner();

    let new_post = NewPost {
        id: post_input.id,
        post_id: post_input.post_id,
        title: post_input.title,
        body: post_input.body,
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

#[get("/blog/post/retrieve/post-id/{post_id}")]
async fn get_by_post_id(
    path: web::Path<String>,
    pool: web::Data<DbPool>,
) -> Result<HttpResponse, Error> {
    let post_id = path.into_inner();
    let mut conn = pool.get().map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Couldn't get db connection from pool: {}", e))
    })?;

    match posts::table.filter(posts::post_id.eq(post_id)).first::<Post>(&mut conn) {
        Ok(post) => Ok(HttpResponse::Ok().json(post)),
        Err(_) => Ok(HttpResponse::NotFound().finish()),
    }
}

#[get("/blog/post/retrieve/{id}")]
async fn get_post(
    path: web::Path<i32>,
    pool: web::Data<DbPool>,
) -> Result<HttpResponse, Error> {
    let id = path.into_inner();
    let mut conn = pool.get().map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Couldn't get db connection from pool: {}", e))
    })?;

    match posts::table.find(id).first::<Post>(&mut conn) {
        Ok(post) => Ok(HttpResponse::Ok().json(post)),
        Err(_) => Ok(HttpResponse::NotFound().finish()),
    }
}

#[get("/blog/post/retrieve/all")]
async fn get_all_posts(pool: web::Data<DbPool>) -> Result<HttpResponse, Error> {
    let mut conn = pool.get().map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Couldn't get db connection from pool: {}", e))
    })?;

    match posts::table.load::<Post>(&mut conn) {
        Ok(posts) => Ok(HttpResponse::Ok().json(posts)),
        Err(_) => Ok(HttpResponse::InternalServerError().finish()),
    }
}

#[put("/blog/posts/update/{post_id}")]
async fn update_post(
    path: web::Path<String>,
    post: web::Json<PostInput>,
    pool: web::Data<DbPool>,
) -> Result<HttpResponse, Error> {

    let post_id = path.into_inner();
    let post_input = post.into_inner();

    let mut conn = pool.get().map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Couldn't get db connection from pool: {}", e))
    })?;

    match diesel::update(posts::table.filter(posts::post_id.eq(post_id)))
        .set((
            posts::id.eq(post_input.id),
            posts::post_id.eq(post_input.post_id),
            posts::title.eq(post_input.title),
            posts::body.eq(post_input.body),
        ))
        .execute(&mut conn)
    {
        Ok(_) => Ok(HttpResponse::Ok().finish()),
        Err(e) => {
            eprintln!("Error updating post: {:?}", e);
            Ok(HttpResponse::InternalServerError().finish())
        }
    }
}

#[delete("/blog/post/single/{post_id}")]
async fn delete_post(
    path: web::Path<String>,  // Changed to String since post_id is a varchar
    pool: web::Data<DbPool>,
) -> Result<HttpResponse, Error> {
    let post_id = path.into_inner();
    let mut conn = pool.get().map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Couldn't get db connection from pool: {}", e))
    })?;

    match diesel::delete(posts::table.filter(posts::post_id.eq(post_id))).execute(&mut conn) {
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
    let mut conn = pool.get().map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Couldn't get db connection from pool: {}", e))
    })?;

    match diesel::sql_query("DELETE FROM posts").execute(&mut conn) {
        Ok(_) => Ok(HttpResponse::NoContent().finish()),
        Err(e) => {
            eprintln!("Error deleting posts: {:?}", e);
            Ok(HttpResponse::InternalServerError().finish())
        }
    }
}

#[delete("/blog/post/all/message")]
async fn delete_all_posts_with_body(
    pool: web::Data<DbPool>,
) -> Result<HttpResponse, Error> {
    let mut conn = pool.get().map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Couldn't get db connection from pool: {}", e))
    })?;

    match diesel::sql_query("DELETE FROM posts").execute(&mut conn) {
        Ok(_) => {
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
