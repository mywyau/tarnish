use actix_web::{web, App, HttpServer};
use dotenv::dotenv;
use std::env;

use my_project::crud::{create_post, get_post, update_post, delete_post, establish_connection};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let pool = web::Data::new(establish_connection());

    HttpServer::new(move || {
        App::new()
            .app_data(pool.clone())
            .service(create_post)
            .service(get_post)
            .service(update_post)
            .service(delete_post)
    })
        .bind(format!("0.0.0.0:{}", port))?
        .run()
        .await
}
