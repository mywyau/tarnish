#[cfg(test)]
mod tests {
    use std::env;
    use tarnish::connectors::postgres_connector::DbPool;
    use tarnish::controllers::blog_controller::{
        create_post,
        delete_all_posts,
        delete_post,
        get_all_posts,
        get_by_post_id,
        get_post,
        update_post,
    };
    use tarnish::models::blog_models::Post;
    use tarnish::schemas::blog_schema::posts;

    use actix_web::http::StatusCode;
    use actix_web::{test, web, App};

    use diesel::prelude::*;

    use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};
    use diesel::ExpressionMethods;
    use diesel::{r2d2, PgConnection};
    use dotenv::dotenv;
    use serde_json::json;
    use serde_json::Value;

    use std::sync::Once;
    use tarnish::NewPost;

    struct TestGuard {
        pool: web::Data<DbPool>,
        post_ids: Vec<String>, // Store the post IDs for cleanup
    }

    impl TestGuard {
        fn new(pool: web::Data<DbPool>, posts_to_insert: Vec<NewPost>) -> Self {
            let mut conn: PooledConnection<ConnectionManager<PgConnection>> =
                pool.get().expect("Failed to get connection from pool");

            // Insert multiple posts into the database
            for post in &posts_to_insert {
                diesel::insert_into(posts::table)
                    .values(post)
                    .execute(&mut conn)
                    .expect("Failed to insert test post");
            }

            // Collect the post IDs for cleanup
            let post_ids = posts_to_insert.into_iter().map(|p| p.post_id).collect();

            TestGuard { pool, post_ids }
        }
    }

    impl Drop for TestGuard {
        fn drop(&mut self) {
            let mut conn: PooledConnection<ConnectionManager<PgConnection>> =
                self.pool.get().expect("Failed to get connection from pool");

            // Clean up: Delete the posts by their post_ids
            for post_id in &self.post_ids {
                diesel::delete(posts::table.filter(posts::post_id.eq(post_id)))
                    .execute(&mut conn)
                    .expect("Failed to delete test post");
            }

            // Reset ID sequence
            diesel::sql_query("ALTER SEQUENCE posts_id_seq RESTART WITH 1")
                .execute(&mut conn)
                .expect("Failed to reset ID sequence");
        }
    }

    pub fn establish_connection() -> DbPool {
        dotenv().ok();
        let database_url = env::var("TEST_DATABASE_URL").expect("TEST_DATABASE_URL must be set");
        let manager = ConnectionManager::<PgConnection>::new(database_url);
        r2d2::Pool::builder().build(manager).expect("Failed to create pool.")
    }

    // #[actix_rt::test]
    // async fn test_create_post() {
    //     dotenv::from_filename(".env.test").ok();
    //
    //     let pool = web::Data::new(establish_connection());
    //     let mut app =
    //         test::init_service(
    //             App::new()
    //                 .app_data(pool.clone())
    //                 .service(get_by_post_id)
    //                 .service(create_post)
    //         ).await;
    //
    //     // First, create a post to ensure there is something to retrieve
    //     let payload =
    //         json!({
    //             "id": 200,
    //             "post_id": "abc200",
    //             "title": "Test Post",
    //             "body": "This is a test post."
    //         });
    //
    //     let create_req =
    //         test::TestRequest::post()
    //             .uri("/blog/post/create")
    //             .set_json(&payload)
    //             .to_request();
    //
    //     let create_resp = test::call_service(&mut app, create_req).await;
    //
    //     assert_eq!(create_resp.status(), StatusCode::CREATED);
    //
    //     // // Extract the status before resp is moved
    //     // let status = create_resp.status();
    //
    //     let body = test::read_body(create_resp).await;  // Move resp here
    //
    //     // Assert that the status is success
    //
    //     // Convert the body to a string
    //     let body_str = std::str::from_utf8(&body).unwrap();
    //
    //     // Optionally, if you expect JSON and want to compare JSON structures
    //     let json_body: Value = serde_json::from_str(body_str).unwrap();
    //
    //     // Example comparison with expected JSON
    //     let expected_json: Value = serde_json::json!({
    //         "body": "This is a test post.",
    //         "id": 2,
    //         "post_id": "abc200",
    //         "title": "Test Post"
    //     });
    //
    //     assert_eq!(json_body, expected_json);
    // }


    mod get_by_post_id_test {

        use crate::tests::TestGuard;
        use crate::tests::establish_connection;

        use std::env;
        use tarnish::connectors::postgres_connector::DbPool;
        use tarnish::controllers::blog_controller::{
            create_post,
            delete_all_posts,
            delete_post,
            get_all_posts,
            get_by_post_id,
            get_post,
            update_post,
        };
        use tarnish::models::blog_models::Post;
        use tarnish::schemas::blog_schema::posts;

        use actix_web::http::StatusCode;
        use actix_web::{test, web, App};

        use diesel::prelude::*;

        use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};
        use diesel::ExpressionMethods;
        use diesel::{r2d2, PgConnection};
        use dotenv::dotenv;
        use serde_json::json;
        use serde_json::Value;

        use std::sync::Once;
        use tarnish::NewPost;

        #[actix_rt::test]
        async fn test_get_by_post_id() {

            dotenv::from_filename(".env.test").ok();
            let pool = web::Data::new(establish_connection());

            // Define multiple posts to insert before the test
            let posts_to_insert = vec![
                NewPost {
                    post_id: "abc123".to_string(),
                    title: "Test Post 1".to_string(),
                    body: "This is the first test post.".to_string(),
                },
                NewPost {
                    post_id: "def456".to_string(),
                    title: "Test Post 2".to_string(),
                    body: "This is the second test post.".to_string(),
                },
            ];

            // Create the guard to insert data and perform cleanup after the test
            let _guard = TestGuard::new(pool.clone(), posts_to_insert);

            let mut app = test::init_service(
                App::new()
                    .app_data(pool.clone())
                    .service(get_by_post_id)
                    .service(create_post)
            ).await;

            // Now, try to retrieve the first post by its ID
            let req = test::TestRequest::get()
                .uri("/blog/post/retrieve/post-id/abc123")
                .to_request();


            let resp = test::call_service(&mut app, req).await;

            // Extract the status before resp is moved
            let status = resp.status();

            let body = test::read_body(resp).await;

            // Assert that the status is success
            assert!(status.is_success());

            // Convert the body to a string
            let body_str = std::str::from_utf8(&body).unwrap();

            // Optionally, if you expect JSON and want to compare JSON structures
            let json_body: Value = serde_json::from_str(body_str).unwrap();

            // Example comparison with expected JSON
            let expected_json: Value = serde_json::json!({
                "body": "This is the first test post.",
                "id": 1,
                "post_id": "abc123",
                "title": "Test Post 1"
            });

            assert_eq!(json_body, expected_json);
        }
    }

    // #[actix_rt::test]
    // async fn test_update_post() {
    //     let pool = web::Data::new(establish_connection());
    //     let mut app = test::init_service(App::new().app_data(web::Data::new(pool.clone())).service(update_post)).await;
    //
    //     let payload = json!({
    //         "id": 1,
    //         "post_id": "abc123",
    //         "title": "Updated Title",
    //         "body": "Updated body content."
    //     });
    //
    //     let req = test::TestRequest::put()
    //         .uri("/blog/posts/update/abc123")
    //         .set_json(&payload)
    //         .to_request();
    //
    //     let resp = test::call_service(&mut app, req).await;
    //
    //     assert!(resp.status().is_success());
    // }
    //
    // #[actix_rt::test]
    // async fn test_delete_post() {
    //     dotenv::from_filename(".env.test").ok();
    //     let pool = web::Data::new(establish_connection());
    //     let mut app = test::init_service(
    //         App::new()
    //             .app_data(pool.clone())
    //             .service(delete_post)
    //             .service(create_post)
    //     ).await;
    //
    //     // First, create a post to ensure there is something to retrieve
    //     let payload =
    //         json!({
    //             "id": 100,
    //             "post_id": "abc200",
    //             "title": "Test Post",
    //             "body": "This is a test post."
    //         });
    //
    //     let create_req =
    //         test::TestRequest::post()
    //             .uri("/blog/post/create")
    //             .set_json(&payload)
    //             .to_request();
    //
    //     let create_resp = test::call_service(&mut app, create_req).await;
    //
    //     assert_eq!(create_resp.status(), StatusCode::CREATED);
    //
    //     let delete_request =
    //         test::TestRequest::delete()
    //             .uri("/blog/post/single/abc200")
    //             .to_request();
    //
    //     let delete_response =
    //         test::call_service(&mut app, delete_request).await;
    //
    //     assert!(delete_response.status().is_success());
    //
    //     // Clean up: Delete the post by post_id
    //     let mut conn: PooledConnection<ConnectionManager<PgConnection>> =
    //         pool.get().expect("Failed to get connection from pool");
    //
    //     diesel::sql_query("ALTER SEQUENCE posts_id_seq RESTART WITH 1")
    //         .execute(&mut conn)
    //         .expect("Failed to reset ID sequence");
    //
    //     // Optionally verify the deletion
    //     let deleted_post = posts::table
    //         .filter(posts::post_id.eq("abc200"))
    //         .first::<Post>(&mut conn)
    //         .optional()
    //         .expect("Failed to check for deleted post");
    //
    //     assert!(deleted_post.is_none());
    // }
}