use std::env;

use actix_cors::Cors;
use actix_web::{App, HttpServer, web};
use dotenv::dotenv;

use my_project::crud::{create_post, delete_all_posts, delete_all_posts_with_body, delete_post, establish_connection, get_by_post_id, get_post, update_post};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let pool = web::Data::new(establish_connection());

    HttpServer::new(move || {
        App::new()
            .wrap(
                Cors::default()
                    .allow_any_origin() // Allow any origin
                    .allow_any_method() // Allow any method
                    .allow_any_header() // Allow any header
                    .supports_credentials() // Allow credentials (cookies, authorization headers, etc.)
            )
            .app_data(pool.clone())
            .wrap(
                Cors::default()
                    .allow_any_origin()
                    .allow_any_method()
                    .allow_any_header()
                    .max_age(3600)
            )
            .service(create_post)
            .service(get_post)
            .service(get_by_post_id)
            .service(update_post)
            .service(delete_post)
            .service(delete_all_posts)
            .service(delete_all_posts_with_body)
    })
        .bind(format!("0.0.0.0:{}", port))?
        .run()
        .await
}
