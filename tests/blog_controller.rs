use actix_web::{test, web, App};
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager, PooledConnection};
use diesel::Connection;
use dotenv::dotenv;
use std::env;

use tarnish::blog_controller::*;
use tarnish::models::{NewPost, Post};
// Use the correct path based on your project name in Cargo.toml
use tarnish::schema::posts;

use lazy_static::lazy_static;
use std::sync::Mutex;

lazy_static! {
    static ref DB_MUTEX: Mutex<()> = Mutex::new(());
}

type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;

fn reset_database(conn: &mut PgConnection) {
    println!("Resetting database...");
    diesel::sql_query("TRUNCATE TABLE posts RESTART IDENTITY;")
        .execute(conn)
        .expect("Failed to reset database");
    println!("Database reset completed.");
}

fn delete_posts_by_ids(conn: &mut PgConnection, ids: Vec<i32>) {
    println!("Deleting specific posts...");

    // Convert the list of IDs into a comma-separated string
    let id_list = ids.into_iter().map(|id| id.to_string()).collect::<Vec<String>>().join(",");

    // Execute the DELETE SQL command to remove the specific posts
    let query = format!("DELETE FROM posts WHERE id IN ({})", id_list);
    diesel::sql_query(query)
        .execute(conn)
        .expect("Failed to delete specific posts");

    println!("Posts deleted.");
}


fn setup_test_db() -> DbPool {
    dotenv().ok();
    let database_url = env::var("TEST_DATABASE_URL").expect("TEST_DATABASE_URL must be set");
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    r2d2::Pool::builder().build(manager).expect("Failed to create pool.")
}

// fn setup() -> (DbPool, PooledConnection<ConnectionManager<PgConnection>>) {
//     let pool = setup_test_db();
//     let mut conn: PooledConnection<ConnectionManager<PgConnection>> = pool.get().expect("Couldn't get db connection");
//
//     // Lock the mutex to ensure only one test resets the database at a time
//     let _lock = DB_MUTEX.lock().unwrap();
//
//     // Drop the lock explicitly (optional, will automatically be dropped when it goes out of scope)
//     drop(_lock);
//
//     (pool, conn)
// }


// fn setup() -> (DbPool, PooledConnection<ConnectionManager<PgConnection>>) {
//     let pool = setup_test_db();
//     let mut conn = pool.get().expect("Couldn't get db connection");
//
//     // Lock the mutex to ensure only one test resets the database at a time
//     let _lock = DB_MUTEX.lock().unwrap();
//
//     // Reset the database before running the test
//     reset_database(&mut conn);
//
//     // Mutex lock will be automatically released when `_lock` goes out of scope
//     (pool, conn)
// }

fn setup() -> (DbPool, PooledConnection<ConnectionManager<PgConnection>>) {
    let pool = setup_test_db();
    let mut conn = pool.get().expect("Couldn't get db connection");

    // Lock the mutex and handle PoisonError
    let _lock = DB_MUTEX.lock().unwrap_or_else(|e| e.into_inner());

    // Reset the database before running the test
    reset_database(&mut conn);

    // Mutex lock will be automatically released when `_lock` goes out of scope
    (pool, conn)
}



#[actix_rt::test]
async fn test_create_post() {
    // Lock the mutex to ensure only one test accesses the database at a time
    let _lock = DB_MUTEX.lock().unwrap();

    let pool = setup_test_db();
    let mut conn = pool.get().expect("Couldn't get db connection");

    let new_post = Post {
        id: 1,
        post_id: "test_post_1".into(),
        title: "Test Post".into(),
        body: "This is a test post.".into(),
    };

    let app = test::init_service(App::new().app_data(web::Data::new(pool.clone())).service(create_post)).await;

    let req = test::TestRequest::post()
        .uri("/blog/post/create")
        .set_json(&new_post)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);

    let result: Post = test::read_body_json(resp).await;
    assert_eq!(result.post_id, "test_post_1");

    delete_posts_by_ids(&mut conn, vec![1]);

    // The mutex lock is automatically released here when `_lock` goes out of scope
}

#[actix_rt::test]
async fn test_update_post() {
    // Lock the mutex to ensure only one test accesses the database at a time
    let _lock = DB_MUTEX.lock().unwrap();

    let pool = setup_test_db();
    let mut conn = pool.get().expect("Couldn't get db connection");

    // Insert a post to be updated
    diesel::insert_into(posts::table)
        .values(&Post {
            id: 2,
            post_id: "test_post_2".into(),
            title: "Test Post".into(),
            body: "This is a test post.".into(),
        })
        .execute(&mut conn)
        .expect("Error inserting test post");

    let updated_post = PostInput {
        id: 2,
        post_id: "test_post_2".into(),
        title: "Updated Post".into(),
        body: "This post has been updated.".into(),
    };

    let app = test::init_service(App::new().app_data(web::Data::new(pool.clone())).service(update_post)).await;

    let req = test::TestRequest::put()
        .uri("/blog/posts/update/test_post_2")
        .set_json(&updated_post)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    // Verify the update
    let updated_post: Post =
        posts::table.filter(posts::post_id.eq("test_post_2"))
            .first(&mut conn)
            .expect("Post not found");

    assert_eq!(updated_post.title, "Updated Post");

    delete_posts_by_ids(&mut conn, vec![2]);

    // The mutex lock is automatically released here when `_lock` goes out of scope
}

#[actix_rt::test]
async fn test_delete_post() {
    // Lock the mutex to ensure only one test accesses the database at a time
    let _lock = DB_MUTEX.lock().unwrap();

    let pool = setup_test_db();
    let mut conn = pool.get().expect("Couldn't get db connection");

    // Insert a post to be deleted
    diesel::insert_into(posts::table)
        .values(&Post {
            id: 3,
            post_id: "test_post_3".into(),
            title: "Test Post".into(),
            body: "This is a test post.".into(),
        })
        .execute(&mut conn)
        .expect("Error inserting test post");

    let app = test::init_service(App::new().app_data(web::Data::new(pool.clone())).service(delete_post)).await;

    let req = test::TestRequest::delete()
        .uri("/blog/post/single/test_post_3")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    // Verify the post was deleted
    let deleted_post: Result<Post, _> = posts::table.filter(posts::post_id.eq("test_post_3"))
        .first(&mut conn);

    assert!(deleted_post.is_err());

    // The mutex lock is automatically released here when `_lock` goes out of scope
}

// #[actix_rt::test]
// async fn test_get_all_posts() {
//     // Lock the mutex to ensure only one test accesses the database at a time
//     let _lock = DB_MUTEX.lock().unwrap();
//
//     let pool = setup_test_db();
//     let mut conn = pool.get().expect("Couldn't get db connection");
//
//     // Insert some posts
//     diesel::insert_into(posts::table)
//         .values(&vec![
//             Post {
//                 id: 4,
//                 post_id: "post_4".into(),
//                 title: "First Post".into(),
//                 body: "This is the first test post.".into(),
//             },
//             Post {
//                 id: 5,
//                 post_id: "post_5".into(),
//                 title: "Second Post".into(),
//                 body: "This is the second test post.".into(),
//             },
//         ])
//         .execute(&mut conn)
//         .expect("Error inserting test posts");
//
//     let app = test::init_service(App::new().app_data(web::Data::new(pool.clone())).service(get_all_posts)).await;
//
//     let req = test::TestRequest::get()
//         .uri("/blog/post/retrieve/all")
//         .to_request();
//
//     let resp = test::call_service(&app, req).await;
//     assert_eq!(resp.status(), 200);
//
//     let result: Vec<Post> = test::read_body_json(resp).await;
//     assert_eq!(result.len(), 2);
//
//     delete_posts_by_ids(&mut conn, vec![4, 5]);
//
//     // The mutex lock is automatically released here when `_lock` goes out of scope
// }

#[actix_rt::test]
async fn test_get_all_posts() {
    // Setup with Mutex lock to ensure serialized access to the database
    let (pool, mut conn) = setup();

    // Insert some posts
    diesel::insert_into(posts::table)
        .values(&vec![
            Post {
                id: 4,
                post_id: "post_4".into(),
                title: "First Post".into(),
                body: "This is the first test post.".into(),
            },
            Post {
                id: 5,
                post_id: "post_5".into(),
                title: "Second Post".into(),
                body: "This is the second test post.".into(),
            },
        ])
        .execute(&mut conn)
        .expect("Error inserting test posts");

    let app = test::init_service(App::new().app_data(web::Data::new(pool.clone())).service(get_all_posts)).await;

    let req = test::TestRequest::get()
        .uri("/blog/post/retrieve/all")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let result: Vec<Post> = test::read_body_json(resp).await;
    assert_eq!(result.len(), 2);
}

