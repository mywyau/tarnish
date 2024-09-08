use actix_web::{delete, get, post, put, web, Error, HttpResponse};
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::schemas::blog_schema::posts;
use crate::table_models::blog_models::{NewPost, Post};
// Import schema
use chrono::DateTime;

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

#[cfg(test)]
mod blog_controller_spec {
    use std::env;

    use crate::controllers::blog_controller::*;
    // use crate::{delete_post, posts, DbPool, NewPost};

    use actix_web::{body::to_bytes, http::StatusCode, test, web, App};
    use bytes::Bytes;
    use chrono::NaiveDateTime;
    use diesel::r2d2::{ConnectionManager, PooledConnection};
    use diesel::{r2d2, PgConnection};
    use dotenv::dotenv;
    use serde_json::{json, Value};

    #[ctor::ctor]
    fn init() {
        let pool = establish_connection();
        let mut conn: PooledConnection<ConnectionManager<PgConnection>> =
            pool.get().expect("Failed to get connection from pool");

        diesel::sql_query("TRUNCATE TABLE posts RESTART IDENTITY CASCADE;")
            .execute(&mut conn)
            .expect("Failed to reset ID sequence");
    }

    struct TestGuard {
        pool: web::Data<DbPool>,
        post_ids: Vec<String>,
    }

    impl TestGuard {
        fn new(pool: web::Data<DbPool>, posts_to_insert: Vec<NewPost>) -> Self {
            let mut conn: PooledConnection<ConnectionManager<PgConnection>> =
                pool.get().expect("Failed to get connection from pool");

            for post in &posts_to_insert {
                diesel::insert_into(posts::table)
                    .values(post)
                    .execute(&mut conn)
                    .expect("Failed to insert test post");
            }

            let post_ids = posts_to_insert.into_iter().map(|p| p.post_id).collect();
            TestGuard { pool, post_ids }
        }
    }

    impl Drop for TestGuard {
        fn drop(&mut self) {
            let mut conn: PooledConnection<ConnectionManager<PgConnection>> =
                self.pool.get().expect("Failed to get connection from pool");

            for post_id in &self.post_ids {
                diesel::delete(posts::table.filter(posts::post_id.eq(post_id)))
                    .execute(&mut conn)
                    .expect("Failed to delete test post");
            }
        }
    }

    pub fn establish_connection() -> DbPool {
        dotenv().ok();
        let database_url = env::var("TEST_DATABASE_URL").expect("TEST_DATABASE_URL must be set");
        let manager = ConnectionManager::<PgConnection>::new(database_url);
        r2d2::Pool::builder().build(manager).expect("Failed to create pool.")
    }

    #[actix_rt::test]
    async fn run_all_tests_in_order_blog() {
        test_get_by_post_id().await;
        test_get_all_posts().await;
        test_create_post().await;
        test_update_post().await;
        test_delete_post().await;
        test_delete_all_posts().await;
    }

    async fn test_create_post() {
        let pool = web::Data::new(establish_connection());

        let app =
            test::init_service(
                App::new()
                    .app_data(pool.clone())
                    .service(get_by_post_id)
                    .service(create_post)
                    .service(update_post)
                    .service(delete_post),
            )
                .await;

        let posts_to_insert = vec![
            NewPost {
                post_id: "abc123".to_string(),
                title: "Test Post 1".to_string(),
                body: "This is the first test post.".to_string(),
                created_at: chrono::Utc::now().naive_utc(), // Current time in ISO 8601 format
                updated_at: chrono::Utc::now().naive_utc(), // Current time in ISO 8601 format
            },
            NewPost {
                post_id: "def456".to_string(),
                title: "Test Post 2".to_string(),
                body: "This is the second test post.".to_string(),
                created_at: chrono::Utc::now().naive_utc(), // Current time in ISO 8601 format
                updated_at: chrono::Utc::now().naive_utc(), // Current time in ISO 8601 format
            },
        ];

        let _guard = TestGuard::new(pool.clone(), posts_to_insert);

        let payload =
            json!({
            "id": 200,
            "post_id": "abc200",
            "title": "Test Post",
            "body": "This is a test post.",
                "created_at": "2023-08-29T14:00:00Z",
                "updated_at": "2023-08-29T14:00:00Z"
        });

        let create_req = test::TestRequest::post()
            .uri("/blog/post/create")
            .set_json(&payload)
            .to_request();

        let create_resp = test::call_service(&app, create_req).await;
        assert_eq!(create_resp.status(), StatusCode::CREATED);

        let body = test::read_body(create_resp).await;
        let body_str = std::str::from_utf8(&body).unwrap();
        let json_body: Value = serde_json::from_str(body_str).unwrap();

        let post_id = json_body.get("post_id").unwrap().as_str().unwrap();
        assert_eq!(post_id, "abc200");
    }

    async fn test_get_by_post_id() {
        let pool = web::Data::new(establish_connection());

        let app =
            test::init_service(
                App::new()
                    .app_data(pool.clone())
                    .service(get_by_post_id)
                    .service(create_post)
                    .service(update_post)
                    .service(delete_post),
            )
                .await;

        let posts_to_insert = vec![
            NewPost {
                post_id: "abc123".to_string(),
                title: "Test Post 1".to_string(),
                body: "This is the first test post.".to_string(),
                created_at: chrono::Utc::now().naive_utc(), // Current time in ISO 8601 format
                updated_at: chrono::Utc::now().naive_utc(), // Current time in ISO 8601 format
            },
            NewPost {
                post_id: "def456".to_string(),
                title: "Test Post 2".to_string(),
                body: "This is the second test post.".to_string(),
                created_at: chrono::Utc::now().naive_utc(), // Current time in ISO 8601 format
                updated_at: chrono::Utc::now().naive_utc(), // Current time in ISO 8601 format
            },
        ];

        let _guard = TestGuard::new(pool.clone(), posts_to_insert);

        let req = test::TestRequest::get()
            .uri("/blog/post/retrieve/post-id/def456")
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let body: Bytes = to_bytes(resp.into_body()).await.unwrap();
        let body_str = std::str::from_utf8(&body).unwrap();
        let json_body: Value = serde_json::from_str(body_str).unwrap();

        let post_id_field = json_body.get("post_id").unwrap().as_str().unwrap();
        let title_field = json_body.get("title").unwrap().as_str().unwrap();
        let body_field = json_body.get("body").unwrap().as_str().unwrap();

        assert_eq!(post_id_field, "def456");
        assert_eq!(title_field, "Test Post 2");
        assert_eq!(body_field, "This is the second test post.");
    }

    async fn test_get_all_posts() {
        let pool = web::Data::new(establish_connection());

        let app = test::init_service(
            App::new()
                .app_data(pool.clone())
                .service(get_all_posts),
        )
            .await;

        let post_1_datetime =
            NaiveDateTime::parse_from_str("2024-08-29 14:30:00", "%Y-%m-%d %H:%M:%S")
                .expect("Failed to parse date");
        let post_2_datetime =
            NaiveDateTime::parse_from_str("2024-08-29 14:30:01", "%Y-%m-%d %H:%M:%S")
                .expect("Failed to parse date");
        let post_3_datetime =
            NaiveDateTime::parse_from_str("2024-08-29 14:30:02", "%Y-%m-%d %H:%M:%S")
                .expect("Failed to parse date");

        let posts_to_insert = vec![
            NewPost {
                post_id: "fake_id_1".to_string(),
                title: "Test Post 1".to_string(),
                body: "This is the first test post.".to_string(),

                created_at: post_1_datetime, // Current time in ISO 8601 format
                updated_at: post_1_datetime, // Current time in ISO 8601 format
            },
            NewPost {
                post_id: "fake_id_2".to_string(),
                title: "Test Post 2".to_string(),
                body: "This is the second test post.".to_string(),
                created_at: post_2_datetime, // Current time in ISO 8601 format
                updated_at: post_2_datetime, // Current time in ISO 8601 format
            },
            NewPost {
                post_id: "fake_id_3".to_string(),
                title: "Test Post 3".to_string(),
                body: "This is the third test post.".to_string(),
                created_at: post_3_datetime, // Current time in ISO 8601 format
                updated_at: post_3_datetime, // Current time in ISO 8601 format
            },
        ];

        let _guard = TestGuard::new(pool.clone(), posts_to_insert);

        let req = test::TestRequest::get()
            .uri("/blog/post/get/all")  // Ensure this matches your actual route
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let body: Bytes = to_bytes(resp.into_body()).await.unwrap();
        let json_body: Value = serde_json::from_slice(&body).unwrap();

        // Assuming the response is an array of posts
        let posts = json_body.as_array().expect("Expected an array of posts");

        // Assert the length of the array
        assert_eq!(posts.len(), 3);

        // Assert the content of each post

        assert_eq!(posts[0]["post_id"], "fake_id_3");
        assert_eq!(posts[0]["title"], "Test Post 3");
        assert_eq!(posts[0]["body"], "This is the third test post.");

        assert_eq!(posts[1]["post_id"], "fake_id_2");
        assert_eq!(posts[1]["title"], "Test Post 2");
        assert_eq!(posts[1]["body"], "This is the second test post.");

        assert_eq!(posts[2]["post_id"], "fake_id_1");
        assert_eq!(posts[2]["title"], "Test Post 1");
        assert_eq!(posts[2]["body"], "This is the first test post.");
    }

    async fn test_update_post() {
        let pool = web::Data::new(establish_connection());

        let app =
            test::init_service(
                App::new()
                    .app_data(pool.clone())
                    .service(get_by_post_id)
                    .service(create_post)
                    .service(update_post)
                    .service(delete_post),
            )
                .await;

        let posts_to_insert = vec![
            NewPost {
                post_id: "abc888".to_string(),
                title: "Test Post 1".to_string(),
                body: "This is the first test post.".to_string(),
                created_at: chrono::Utc::now().naive_utc(), // Current time in ISO 8601 format
                updated_at: chrono::Utc::now().naive_utc(), // Current time in ISO 8601 format
            },
        ];

        let _guard = TestGuard::new(pool.clone(), posts_to_insert);

        let payload = json!({
            "id": 1,
            "post_id": "abc888",
            "title": "Updated Title",
            "body": "Updated body content.",
            "created_at": "2023-08-29T14:00:00Z", // Example timestamp
            "updated_at": "2023-08-29T14:00:01Z"  // Example timestamp
        });

        let put_req = test::TestRequest::put()
            .uri("/blog/posts/update/abc888")
            .set_json(&payload)
            .to_request();

        let put_resp = test::call_service(&app, put_req).await;
        assert!(put_resp.status().is_success());

        let body: Bytes = to_bytes(put_resp.into_body()).await.unwrap();
        let body_str = std::str::from_utf8(&body).unwrap();
        let json_body: Value = serde_json::from_str(body_str).unwrap();

        let expected_message = json!({
            "message": "Blog post 'Test Post 1' has been updated"
        });

        assert_eq!(json_body, expected_message);
    }

    async fn test_delete_post() {
        let pool = web::Data::new(establish_connection());

        let app =
            test::init_service(
                App::new()
                    .app_data(pool.clone())
                    .service(get_by_post_id)
                    .service(create_post)
                    .service(update_post)
                    .service(delete_post),
            )
                .await;

        let posts_to_insert = vec![
            NewPost {
                post_id: "abc200".to_string(),
                title: "Test Post 1".to_string(),
                body: "This is the first test post.".to_string(),

                created_at: chrono::Utc::now().naive_utc(), // Current time in ISO 8601 format
                updated_at: chrono::Utc::now().naive_utc(), // Current time in ISO 8601 format
            },
            NewPost {
                post_id: "def456".to_string(),
                title: "Test Post 2".to_string(),
                body: "This is the second test post.".to_string(),
                created_at: chrono::Utc::now().naive_utc(), // Current time in ISO 8601 format
                updated_at: chrono::Utc::now().naive_utc(), // Current time in ISO 8601 format
            },
        ];

        let _guard = TestGuard::new(pool.clone(), posts_to_insert);

        let delete_request = test::TestRequest::delete()
            .uri("/blog/post/single/abc200")
            .to_request();

        let delete_response = test::call_service(&app, delete_request).await;
        assert!(delete_response.status().is_success());

        let mut conn: PooledConnection<ConnectionManager<PgConnection>> =
            pool.get().expect("Failed to get connection from pool");

        let deleted_post = posts::table
            .filter(posts::post_id.eq("abc200"))
            .first::<Post>(&mut conn)
            .optional()
            .expect("Failed to check for deleted post");

        assert!(deleted_post.is_none());

        let deleted_post = posts::table
            .filter(posts::post_id.eq("def456"))
            .first::<Post>(&mut conn)
            .optional()
            .expect("Failed to check for deleted post");

        assert!(deleted_post.is_some());
    }

    async fn test_delete_all_posts() {
        let pool = web::Data::new(establish_connection());

        let app =
            test::init_service(
                App::new()
                    .app_data(pool.clone())
                    .service(get_by_post_id)
                    .service(create_post)
                    .service(update_post)
                    .service(delete_all_posts),
            )
                .await;

        let posts_to_insert = vec![
            NewPost {
                post_id: "fake_id_1".to_string(),
                title: "Test Post 1".to_string(),
                body: "This is the first test post.".to_string(),
                created_at: chrono::Utc::now().naive_utc(), // Current time in ISO 8601 format
                updated_at: chrono::Utc::now().naive_utc(), // Current time in ISO 8601 format
            },
            NewPost {
                post_id: "fake_id_2".to_string(),
                title: "Test Post 2".to_string(),
                body: "This is the second test post.".to_string(),
                created_at: chrono::Utc::now().naive_utc(), // Current time in ISO 8601 format
                updated_at: chrono::Utc::now().naive_utc(), // Current time in ISO 8601 format
            },
        ];

        let _guard = TestGuard::new(pool.clone(), posts_to_insert);

        let delete_request =
            test::TestRequest::delete()
                .uri("/blog/post/all")
                .to_request();

        let delete_response =
            test::call_service(&app, delete_request).await;

        assert!(delete_response.status().is_success());

        let mut conn: PooledConnection<ConnectionManager<PgConnection>> =
            pool.get().expect("Failed to get connection from pool");

        let deleted_post = posts::table
            .filter(posts::post_id.eq("fake_id_1"))
            .first::<Post>(&mut conn)
            .optional()
            .expect("Failed to check for deleted post");

        assert!(deleted_post.is_none());

        let deleted_post = posts::table
            .filter(posts::post_id.eq("fake_id_2"))
            .first::<Post>(&mut conn)
            .optional()
            .expect("Failed to check for deleted post");

        assert!(deleted_post.is_none());
    }
}

