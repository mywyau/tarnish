#[cfg(test)]
mod tests {
    use std::env;
    // use std::iter::Once;
    // use std::sync::{Once};
    use tarnish::connectors::postgres_connector::DbPool;
    use tarnish::schemas::blog_schema::posts;

    use actix_web::{test, web};

    use diesel::prelude::*;

    use diesel::r2d2::{ConnectionManager, PooledConnection};
    use diesel::ExpressionMethods;
    use diesel::{r2d2, PgConnection};
    use dotenv::dotenv;

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

            // // Reset ID sequence
            // diesel::sql_query("TRUNCATE TABLE posts RESTART IDENTITY CASCADE;")
            //     .execute(&mut conn)
            //     .expect("Failed to reset ID sequence");
            //
            // // Reset ID sequence
            // diesel::sql_query("ALTER SEQUENCE posts_id_seq RESTART WITH 1")
            //     .execute(&mut conn)
            //     .expect("Failed to reset ID sequence");
        }
    }

    pub fn establish_connection() -> DbPool {
        dotenv().ok();
        let database_url = env::var("TEST_DATABASE_URL").expect("TEST_DATABASE_URL must be set");
        let manager = ConnectionManager::<PgConnection>::new(database_url);
        r2d2::Pool::builder().build(manager).expect("Failed to create pool.")
    }

    mod create_post_tests {
        use crate::tests::establish_connection;
        use crate::tests::TestGuard;

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

        use tarnish::NewPost;

        #[actix_rt::test]
        async fn test_create_post() {
            dotenv::from_filename(".env.test").ok();

            let pool = web::Data::new(establish_connection());

            // Define multiple posts to insert before the test
            let posts_to_insert =
                vec![
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

            let mut app =
                test::init_service(
                    App::new()
                        .app_data(pool.clone())
                        .service(get_by_post_id)
                        .service(create_post)
                ).await;

            // First, create a post to ensure there is something to retrieve
            let payload =
                json!({
                "id": 200,
                "post_id": "abc200",
                "title": "Test Post",
                "body": "This is a test post."
            });

            let create_req =
                test::TestRequest::post()
                    .uri("/blog/post/create")
                    .set_json(&payload)
                    .to_request();

            let create_resp = test::call_service(&mut app, create_req).await;

            assert_eq!(create_resp.status(), StatusCode::CREATED);

            let body = test::read_body(create_resp).await;

            let body_str = std::str::from_utf8(&body).unwrap();

            let json_body: Value = serde_json::from_str(body_str).unwrap();

            // let expected_json: Value =
            //     serde_json::json!({
            //         "body": "This is a test post.",
            //         "id": 2,
            //         "post_id": "abc200",
            //         "title": "Test Post"
            //     });

            // assert_eq!(json_body, expected_json);

            // Extract the post_id from the JSON response
            let post_id = json_body.get("post_id").unwrap().as_str().unwrap();

            // Assert that the post_id is what you expect
            assert_eq!(post_id, "abc200");
        }
    }

    mod get_by_post_id_test {
        use crate::tests::establish_connection;
        use crate::tests::TestGuard;

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

        use actix_web::http::StatusCode;
        use diesel::prelude::*;

        use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};
        use diesel::ExpressionMethods;
        use diesel::{r2d2, PgConnection};
        use dotenv::dotenv;
        use serde_json::json;
        use serde_json::Value;

        use tarnish::NewPost;

        use actix_web::body::to_bytes;
        use actix_web::{test, web, App};
        use bytes::Bytes;

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
                .uri("/blog/post/retrieve/post-id/def456")
                .to_request();

            let resp = test::call_service(&mut app, req).await;

            // Extract the status before resp is moved
            let status = resp.status();

            // Correctly read the body
            let body: Bytes = to_bytes(resp.into_body()).await.unwrap();

            // Assert that the status is success
            assert!(status.is_success());

            // Convert the body to a string
            let body_str = std::str::from_utf8(&body).unwrap();

            // Optionally, if you expect JSON and want to compare JSON structures
            let json_body: Value = serde_json::from_str(body_str).unwrap();

            // Extract the post_id from the JSON response
            let post_id_field = json_body.get("post_id").unwrap().as_str().unwrap();
            let title_field = json_body.get("title").unwrap().as_str().unwrap();
            let body_field = json_body.get("body").unwrap().as_str().unwrap();

            // Assert that the fields are what you expect
            assert_eq!(post_id_field, "def456");
            assert_eq!(title_field, "Test Post 2");
            assert_eq!(body_field, "This is the second test post.");
        }
    }

    mod update_post_test {
        use crate::tests::establish_connection;
        use crate::tests::TestGuard;

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

        use actix_web::body::to_bytes;
        use actix_web::http::StatusCode;
        use actix_web::{test, web, App};
        use bytes::Bytes;
        use diesel::prelude::*;

        use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};
        use diesel::ExpressionMethods;
        use diesel::{r2d2, PgConnection};
        use dotenv::dotenv;
        use serde_json::json;
        use serde_json::Value;

        use tarnish::NewPost;

        #[actix_rt::test]
        async fn test_update_post() {
            // Step 1: Establish connection to the database and initialize the service
            let pool = web::Data::new(establish_connection());

            let mut app = test::init_service(
                App::new()
                    .app_data(pool.clone())
                    .service(update_post)
                    .service(get_by_post_id)
            )
                .await;

            // Step 2: Insert a post into the database
            let posts_to_insert = vec![
                NewPost {
                    post_id: "abc888".to_string(),
                    title: "Test Post 1".to_string(),
                    body: "This is the first test post.".to_string(),
                },
            ];

            let _guard = TestGuard::new(pool.clone(), posts_to_insert);

            // Step 3: Prepare the update payload
            let payload = json!({
        "id": 1,
        "post_id": "abc888",
        "title": "Updated Title",
        "body": "Updated body content."
    });

            // Step 4: Send the PUT request to update the post
            let put_req = test::TestRequest::put()
                .uri("/blog/posts/update/abc888")
                .set_json(&payload)
                .to_request();

            let put_resp = test::call_service(&mut app, put_req).await;

            // Step 5: Assert that the update was successful
            assert!(put_resp.status().is_success());

            // Step 6: Read and parse the response body
            let body: Bytes = to_bytes(put_resp.into_body()).await.unwrap();
            let body_str = std::str::from_utf8(&body).unwrap();
            let json_body: Value = serde_json::from_str(body_str).unwrap();

            // Step 7: Assert that the response contains the correct update message
            let expected_message = json!({
        "message": "Blog post 'Test Post 1' has been updated"
    });
            assert_eq!(json_body, expected_message);

            // Optionally: Retrieve the updated post to ensure changes were applied correctly
            let get_req = test::TestRequest::get()
                .uri("/blog/post/retrieve/post-id/abc888")
                .to_request();

            let get_resp = test::call_service(&mut app, get_req).await;
            assert!(get_resp.status().is_success());

            let get_body: Bytes = to_bytes(get_resp.into_body()).await.unwrap();
            let get_body_str = std::str::from_utf8(&get_body).unwrap();
            let updated_post: Value = serde_json::from_str(get_body_str).unwrap();

            let expected_updated_post = json!({
        "id": 1,
        "post_id": "abc888",
        "title": "Updated Title",
        "body": "Updated body content."
    });

            assert_eq!(updated_post, expected_updated_post);
        }
    }

    // #[cfg(test)]
    // mod delete_tests {
    //     use super::*;
    //     use actix_web::{test, web, App};
    //     use diesel::connection::SimpleConnection;
    //     use diesel::prelude::*;
    //     use mockall::{automock, predicate::*};
    //     use mockall::mock;
    //     use serde_json::json;
    //
    //     // Mock the connection
    //     mock! {
    //     pub Conn {}
    //
    //     impl SimpleConnection for Conn {
    //         fn batch_execute(&self, query: &str) -> QueryResult<()>;
    //     }
    //
    //     impl Connection for Conn {
    //         type Backend = diesel::pg::Pg;
    //         type TransactionManager = diesel::connection::AnsiTransactionManager;
    //
    //         fn establish(database_url: &str) -> ConnectionResult<Self> where Self: Sized;
    //         fn execute_returning_count<T>(&mut self, source: &T) -> QueryResult<usize> where T: diesel::query_builder::QueryFragment<Self::Backend> + diesel::query_builder::QueryId + ?Sized;
    //         fn transaction_manager(&self) -> &Self::TransactionManager;
    //     }
    // }
    //
    //     // Mocking the posts schema
    //     mod posts {
    //         use diesel::table;
    //
    //         table! {
    //         posts (id) {
    //             id -> Int4,
    //             post_id -> Varchar,
    //             title -> Varchar,
    //             body -> Text,
    //         }
    //     }
    //     }
    //
    //     #[actix_rt::test]
    //     async fn test_delete_post_success() {
    //         // Mock the connection pool and connection
    //         let mut mock_conn = MockConn::new();
    //         let post_id = "abc123".to_string();
    //
    //         // Set up the expectations for the mocked methods
    //         mock_conn
    //             .expect_execute_returning_count()
    //             .with(predicate::always())
    //             .returning(|_| Ok(1));
    //
    //         mock_conn
    //             .expect_batch_execute()
    //             .returning(|_| Ok(()));
    //
    //         mock_conn
    //             .expect_transaction_manager()
    //             .return_const(diesel::connection::AnsiTransactionManager::new());
    //
    //         // Simulate the DB response for finding a post
    //         mock_conn
    //             .expect_query_by_index::<String>()
    //             .with(predicate::eq(posts::posts::title))
    //             .returning(|_| Ok(Some("Test Title".to_string())));
    //
    //         let pool = web::Data::new(MockConnManager { conn: mock_conn });
    //
    //         let req = test::TestRequest::delete()
    //             .uri(&format!("/blog/post/single/{}", post_id))
    //             .to_request();
    //
    //         let mut app = test::init_service(App::new().app_data(pool.clone()).service(delete_post)).await;
    //
    //         let resp = test::call_service(&mut app, req).await;
    //         assert!(resp.status().is_success());
    //
    //         let body = test::read_body(resp).await;
    //         let body_str = std::str::from_utf8(&body).unwrap();
    //         let json_body: serde_json::Value = serde_json::from_str(body_str).unwrap();
    //
    //         let expected_body = json!({
    //         "message": format!("Blog post '{}' has been deleted", "Test Title")
    //     });
    //
    //         assert_eq!(json_body, expected_body);
    //     }
    //
    //     #[actix_rt::test]
    //     async fn test_delete_post_not_found() {
    //         // Mock the connection pool and connection
    //         let mut mock_conn = MockConn::new();
    //         let post_id = "abc123".to_string();
    //
    //         // Simulate the DB response for not finding a post
    //         mock_conn
    //             .expect_query_by_index::<String>()
    //             .with(predicate::eq(posts::posts::title))
    //             .returning(|_| Ok(None));
    //
    //         let pool = web::Data::new(MockConnManager { conn: mock_conn });
    //
    //         let req = test::TestRequest::delete()
    //             .uri(&format!("/blog/post/single/{}", post_id))
    //             .to_request();
    //
    //         let mut app = test::init_service(App::new().app_data(pool.clone()).service(delete_post)).await;
    //
    //         let resp = test::call_service(&mut app, req).await;
    //         assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    //
    //         let body = test::read_body(resp).await;
    //         let body_str = std::str::from_utf8(&body).unwrap();
    //         let json_body: serde_json::Value = serde_json::from_str(body_str).unwrap();
    //
    //         let expected_body = json!({
    //         "error": format!("Blog post with ID '{}' not found", post_id)
    //     });
    //
    //         assert_eq!(json_body, expected_body);
    //     }
    // }
}