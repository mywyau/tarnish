// use actix_rt::test;
// use actix_web::{App, test as actix_test, web};
// use diesel::prelude::*;
// use diesel::r2d2::Pool;
// use log::info;
// use serde_json::json;
//
// use my_project::{create_post, DbPool, delete_post, establish_connection, get_post, update_post};
// use my_project::models::Post;
// use my_project::schema::posts;
//
// fn setup_db() -> DbPool {
//     dotenv::dotenv().ok();
//     let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
//     let manager = diesel::r2d2::ConnectionManager::<PgConnection>::new(database_url);
//     let pool = Pool::builder().build(manager).expect("Failed to create pool.");
//
//     let mut conn = pool.get().expect("Couldn't get db connection from pool");
//     diesel::delete(posts::table).execute(&mut conn).expect("Failed to clear posts table");
//
//     // Reset the sequence
//     diesel::sql_query("ALTER SEQUENCE posts_id_seq RESTART WITH 1")
//         .execute(&mut conn)
//         .expect("Failed to reset sequence");
//
//     pool
// }
//
// #[actix_rt::test]
// async fn test_create_post_with_sequence_reset() {
//     let pool = setup_db();
//
//     let mut app = actix_test::init_service(
//         App::new()
//             .app_data(web::Data::new(pool.clone()))
//             .service(create_post)
//     ).await;
//
//     let req = actix_test::TestRequest::post()
//         .uri("/posts")
//         .set_json(&json!({"title": "Test title", "body": "Test body"}))
//         .to_request();
//
//     let resp = actix_test::call_service(&mut app, req).await;
//     assert!(resp.status().is_success());
//
//     let body: Post = actix_test::read_body_json(resp).await;
//     assert_eq!(body.id, 1); // Ensure ID starts from 1
//     assert_eq!(body.title, "Test title");
//     assert_eq!(body.body, "Test body");
// }
//
// #[actix_rt::test]
// async fn test_get_post() {
//     let pool = setup_db();
//     let mut app = actix_test::init_service(
//         App::new()
//             .app_data(web::Data::new(pool.clone()))
//             .service(create_post)
//             .service(get_post)
//     ).await;
//
//     // First, create a post
//     let req = actix_test::TestRequest::post()
//         .uri("/posts")
//         .set_json(&json!({"title": "Unique Test Title", "body": "Unique Test Body"}))
//         .to_request();
//
//     let create_resp = actix_test::call_service(&mut app, req).await;
//     info!("{:?}", create_resp);
//     assert!(create_resp.status().is_success());
//
//     let created_post: my_project::models::Post = actix_test::read_body_json(create_resp).await;
//     let post_id = created_post.id;
//
//     // Then, get the post by ID
//     let req = actix_test::TestRequest::get()
//         .uri(&format!("/posts/{}", post_id))
//         .to_request();
//
//     let get_resp = actix_test::call_service(&mut app, req).await;
//     info!("{:?}", get_resp);
//
//     assert!(get_resp.status().is_success());
//
//     let body: my_project::models::Post = actix_test::read_body_json(get_resp).await;
//     assert_eq!(body.title, "Unique Test Title");
//     assert_eq!(body.body, "Unique Test Body");
// }
//
// // Similarly, you can write tests for update_post and delete_post
//
//
