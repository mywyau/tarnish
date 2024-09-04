use actix_web::{delete, get, post, put, web, Error, HttpResponse};
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use serde::{Deserialize, Serialize};
use serde_json::json;

// Import schema
use crate::{posts, NewPost, Post};
use chrono::{DateTime};

pub type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;
#[derive(Serialize, Deserialize)]
pub struct PostInput {
    pub id: i32,
    pub post_id: String,
    pub title: String,
    pub body: String,
    pub created_at: String,
    pub updated_at: String,
}

impl PostInput {
    // Constructor method for creating a new PostInput
    pub fn new(id: i32, post_id: String, title: String, body: String, created_at: String, updated_at: String) -> Self {
        PostInput {
            id,
            post_id,
            title,
            body,
            created_at,
            updated_at,
        }
    }
}

#[post("/blog/post/create")]
async fn create_post(
    pool: web::Data<DbPool>,
    post: web::Json<PostInput>,
) -> Result<HttpResponse, Error> {
    let post_input = post.into_inner();

    let new_post =
        NewPost {
            post_id: post_input.post_id,
            title: post_input.title,
            body: post_input.body,
            created_at: DateTime::parse_from_rfc3339(&post_input.created_at)
                .unwrap()
                .naive_utc(), // Convert to NaiveDateTime
            updated_at: DateTime::parse_from_rfc3339(&post_input.updated_at)
                .unwrap()
                .naive_utc(), // Convert to NaiveDateTime
        };
    let mut conn =
        pool.get().map_err(|e| {
            actix_web::error::ErrorInternalServerError(format!("Couldn't get db connection from pool: {}", e))
        })?;

    conn.transaction::<_, diesel::result::Error, _>(|conn| {
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

#[get("/blog/post/get/all")]
async fn get_all_posts(pool: web::Data<DbPool>) -> Result<HttpResponse, Error> {
    // Get a connection from the pool
    let mut conn = pool.get().map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Couldn't get db connection from pool: {}", e))
    })?;

    // Query the posts, ordering them by creation time (assuming created_at is the timestamp column)
    match posts::table.order(posts::created_at.desc()).load::<Post>(&mut conn) {
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

    let mut conn =
        pool.get().map_err(|e| {
            actix_web::error::ErrorInternalServerError(format!("Couldn't get db connection from pool: {}", e))
        })?;

    // First, retrieve the title of the post before deleting it
    let post_title =
        posts::table
            .filter(posts::post_id.eq(&post_id))
            .select(posts::title)
            .first::<String>(&mut conn)
            .optional()
            .map_err(|e| {
                actix_web::error::ErrorInternalServerError(format!("Error retrieving post title: {}", e))
            })?;

    match post_title {
        Some(title) => {
            match diesel::update(posts::table.filter(posts::post_id.eq(post_id)))
                .set((
                    posts::id.eq(post_input.id),
                    posts::post_id.eq(post_input.post_id),
                    posts::title.eq(post_input.title),
                    posts::body.eq(post_input.body),
                ))
                .execute(&mut conn)
            {
                Ok(_) => {
                    let response_body =
                        json!({"message": format!("Blog post '{}' has been updated", title)});
                    Ok(HttpResponse::Ok()
                        .content_type("application/json")
                        .json(response_body))
                }
                Err(e) => {
                    eprintln!("Error updating post: {:?}", e);
                    Ok(HttpResponse::InternalServerError().finish())
                }
            }
        }
        None => {
            // If no post with the given ID is found
            let response_body = json!({
                "error": format!("Blog post with ID '{}' not found", post_id)
            });

            Ok(HttpResponse::NotFound()
                .content_type("application/json")
                .json(response_body))
        }
    }
}


#[delete("/blog/post/single/{post_id}")]
async fn delete_post(
    path: web::Path<String>,  // Changed to String since post_id is a varchar
    pool: web::Data<DbPool>,
) -> Result<HttpResponse, Error> {
    let post_id = path.into_inner();
    let mut conn =
        pool.get().map_err(|e| {
            actix_web::error::ErrorInternalServerError(format!("Couldn't get db connection from pool: {}", e))
        })?;

    // First, retrieve the title of the post before deleting it
    let post_title =
        posts::table
            .filter(posts::post_id.eq(&post_id))
            .select(posts::title)
            .first::<String>(&mut conn)
            .optional()
            .map_err(|e| {
                actix_web::error::ErrorInternalServerError(format!("Error retrieving post title: {}", e))
            })?;

    match post_title {
        Some(title) => {
            // Now delete the post
            match diesel::delete(posts::table.filter(posts::post_id.eq(&post_id))).execute(&mut conn) {
                Ok(_) => {
                    let response_body = json!({
                        "message": format!("Blog post '{}' has been deleted", title)
                    });

                    Ok(HttpResponse::Ok()
                        .content_type("application/json")
                        .json(response_body))
                }
                Err(e) => {
                    eprintln!("Error deleting post: {:?}", e);
                    Ok(HttpResponse::InternalServerError().finish())
                }
            }
        }
        None => {
            // If no post with the given ID is found
            let response_body = json!({
                "error": format!("Blog post with ID '{}' not found", post_id)
            });

            Ok(HttpResponse::NotFound()
                .content_type("application/json")
                .json(response_body))
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

    match diesel::sql_query("TRUNCATE TABLE posts RESTART IDENTITY CASCADE").execute(&mut conn) {
        Ok(_) => {
            let response_body = json!({
                "message": "All posts have been deleted."
            });
            Ok(HttpResponse::Ok()
                .content_type("application/json")
                .json(response_body))
        }
        Err(e) => {
            eprintln!("Error truncating posts: {:?}", e);
            Ok(HttpResponse::InternalServerError().finish())
        }
    }
}

