use actix_web::{test, web, App};
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use diesel::connection::Connection;
use dotenv::dotenv;
use std::env;

// Use the correct path based on your project name in Cargo.toml
use tarnish::schema::test_posts;
use tarnish::models::{Post, NewPost};
use tarnish::blog_controller::*;  // Assuming the functions are in blog_controller.rs

type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;

fn setup_test_db() -> DbPool {
    dotenv().ok(); // Load environment variables from `.test-env`
    let database_url = env::var("TEST_DATABASE_URL").expect("TEST_DATABASE_URL must be set");
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    r2d2::Pool::builder().build(manager).expect("Failed to create pool.")
}

#[actix_rt::test]
async fn test_create_post() {
    let pool = setup_test_db();
    let mut conn = pool.get().expect("Couldn't get db connection");
    conn.transaction::<_, diesel::result::Error, _>(|conn| {
        let new_post = PostInput::new(
            1,
            "123".into(),
            "Test Post".into(),
            "This is a test post".into(),
        );

        actix_rt::System::new().block_on(async {
            let app = test::init_service(App::new().app_data(web::Data::new(pool.clone())).service(create_post)).await;

            let req = test::TestRequest::post()
                .uri("/blog/post/create")
                .set_json(&new_post)
                .to_request();

            let resp = test::call_service(&app, req).await;
            assert_eq!(resp.status(), 201);

            let result: Post = test::read_body_json(resp).await;
            assert_eq!(result.post_id, "123");
        });

        Ok(())
    }).unwrap();
}

#[actix_rt::test]
async fn test_get_post() {
    let pool = setup_test_db();
    let mut conn = pool.get().expect("Couldn't get db connection");

    conn.transaction::<_, diesel::result::Error, _>(|conn| {
        // Insert a post into the database for retrieval
        diesel::insert_into(test_posts::table)
            .values(NewPost {
                id: 1,
                post_id: "123".into(),
                title: "Test Post".into(),
                body: "This is a test post".into(),
            })
            .execute(conn)
            .expect("Error inserting test post");

        actix_rt::System::new().block_on(async {
            let app = test::init_service(App::new().app_data(web::Data::new(pool.clone())).service(get_post)).await;

            let req = test::TestRequest::get()
                .uri("/blog/post/retrieve/1")
                .to_request();

            let resp = test::call_service(&app, req).await;
            assert_eq!(resp.status(), 200);

            let result: Post = test::read_body_json(resp).await;
            assert_eq!(result.title, "Test Post");
        });

        Ok(())
    }).unwrap();
}

#[actix_rt::test]
async fn test_delete_post() {
    let pool = setup_test_db();
    let mut conn = pool.get().expect("Couldn't get db connection");

    conn.transaction::<_, diesel::result::Error, _>(|conn| {
        // Insert a post into the database for deletion
        diesel::insert_into(test_posts::table)
            .values(NewPost {
                id: 1,
                post_id: "123".into(),
                title: "Test Post".into(),
                body: "This is a test post".into(),
            })
            .execute(conn)
            .expect("Error inserting test post");

        actix_rt::System::new().block_on(async {
            let app = test::init_service(App::new().app_data(web::Data::new(pool.clone())).service(delete_post)).await;

            let req = test::TestRequest::delete()
                .uri("/blog/post/single/123")
                .to_request();

            let resp = test::call_service(&app, req).await;
            assert_eq!(resp.status(), 200);

            // Verify the post was deleted
            let post_count: i64 = test_posts::table.filter(test_posts::post_id.eq("123")).count().get_result(conn).unwrap();
            assert_eq!(post_count, 0);
        });

        Ok(())
    }).unwrap();
}
