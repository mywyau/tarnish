#[cfg(test)]
mod tests {
    use std::env;
    // use super::;

    use tarnish::connectors::postgres_connector::DbPool;
    use tarnish::models::blog_models::Post;
    use tarnish::schemas::blog_schema::posts;
    use tarnish::controllers::blog_controller::{
        create_post,
        delete_all_posts,
        delete_all_posts_with_body,
        delete_post,
        get_all_posts,
        get_by_post_id,
        get_post,
        update_post,
    };

    use actix_web::http::StatusCode;
    use actix_web::{test, web, App};

    use diesel::prelude::*;

    use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};
    use diesel::ExpressionMethods;
    use diesel::{r2d2, PgConnection};
    use dotenv::dotenv;
    use serde_json::json;
    use serde_json::Value;


    pub fn establish_connection() -> DbPool {
        dotenv().ok();
        let database_url = env::var("TEST_DATABASE_URL").expect("TEST_DATABASE_URL must be set");
        let manager = ConnectionManager::<PgConnection>::new(database_url);
        r2d2::Pool::builder().build(manager).expect("Failed to create pool.")
    }

    // fn setup_db() -> Pool<ConnectionManager<PgConnection>> {
    //     // Use a test database URL or in-memory database if supported
    //     let database_url = env::var("TEST_DATABASE_URL").expect("TEST_DATABASE_URL must be set");
    //     let manager = ConnectionManager::<PgConnection>::new(database_url);
    //     Pool::builder().build(manager).expect("Failed to create pool.")
    // }

    #[actix_rt::test]
    async fn test_create_post() {
        let pool = web::Data::new(establish_connection());
        let mut app = test::init_service(App::new().app_data(web::Data::new(pool.clone())).service(create_post)).await;

        let payload = json!({
            "id": 1,
            "post_id": "abc123",
            "title": "Test Post",
            "body": "This is a test post."
        });

        let req = test::TestRequest::post()
            .uri("/blog/post/create")
            .set_json(&payload)
            .to_request();

        let resp = test::call_service(&mut app, req).await;

        assert!(resp.status().is_success());
    }

    #[actix_rt::test]
    async fn test_get_by_post_id() {
        dotenv::from_filename(".env.test").ok();
        let pool = web::Data::new(establish_connection());
        let mut app = test::init_service(
            App::new()
                .app_data(pool.clone())
                .service(get_by_post_id)
                .service(create_post)
        ).await;

        // First, create a post to ensure there is something to retrieve
        let payload = json!({
        "id": 1,
        "post_id": "abc123",
        "title": "Test Post",
        "body": "This is a test post."
    });

        let create_req = test::TestRequest::post()
            .uri("/blog/post/create")
            .set_json(&payload)
            .to_request();

        let create_resp = test::call_service(&mut app, create_req).await;
        assert_eq!(create_resp.status(), StatusCode::CREATED);

        // Now, try to retrieve the post by its ID
        let req = test::TestRequest::get()
            .uri("/blog/post/retrieve/post-id/abc123")
            .to_request();

        let resp = test::call_service(&mut app, req).await;

        // Extract the status before resp is moved
        let status = resp.status();

        let body = test::read_body(resp).await;  // Move resp here

        // Assert that the status is success
        assert!(status.is_success());

        // Convert the body to a string
        let body_str = std::str::from_utf8(&body).unwrap();

        // Optionally, if you expect JSON and want to compare JSON structures
        let json_body: Value = serde_json::from_str(body_str).unwrap();

        // Example comparison with expected JSON
        let expected_json: Value = serde_json::json!({
            "body": "This is a test post.",
            "id": 1,
            "post_id": "abc123",
            "title": "Test Post"
        });

        assert_eq!(json_body, expected_json);

        // Clean up: Delete the post by post_id
        let mut conn: PooledConnection<ConnectionManager<PgConnection>> =
            pool.get().expect("Failed to get connection from pool");
            diesel::delete(posts::table.filter(posts::post_id.eq("abc123")))
            .execute(& mut conn)
            .expect("Failed to delete test post");

        // Optionally verify the deletion
        let deleted_post = posts::table
            .filter(posts::post_id.eq("abc123"))
            .first::<Post>(& mut conn)
            .optional()
            .expect("Failed to check for deleted post");

        assert!(deleted_post.is_none());
    }

    #[actix_rt::test]
    async fn test_update_post() {
        // let pool = setup_db();
        let pool = web::Data::new(establish_connection());
        let mut app = test::init_service(App::new().app_data(web::Data::new(pool.clone())).service(update_post)).await;

        let payload = json!({
            "id": 1,
            "post_id": "abc123",
            "title": "Updated Title",
            "body": "Updated body content."
        });

        let req = test::TestRequest::put()
            .uri("/blog/posts/update/abc123")
            .set_json(&payload)
            .to_request();

        let resp = test::call_service(&mut app, req).await;

        assert!(resp.status().is_success());
    }

    #[actix_rt::test]
    async fn test_delete_post() {
        // let pool = setup_db();
        let pool = web::Data::new(establish_connection());
        let mut app = test::init_service(App::new().app_data(web::Data::new(pool.clone())).service(delete_post)).await;

        // Assuming a post with post_id "abc123" exists
        let req = test::TestRequest::delete()
            .uri("/blog/post/single/abc123")
            .to_request();

        let resp = test::call_service(&mut app, req).await;

        assert!(resp.status().is_success());
    }
}
