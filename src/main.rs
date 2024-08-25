use std::env;

use actix_cors::Cors;
use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use dotenv::dotenv;

use tarnish::controllers::blog_controller::{create_post, delete_all_posts, delete_post, establish_connection, get_all_posts, get_by_post_id, get_post, update_post};

use tarnish::controllers::skills_controller::{
    create_skill, delete_skill, get_all_skills, get_skill, update_skill,
};
use tarnish::controllers::worklog_controller::{
    create_worklog, delete_worklog, get_all_worklog, get_worklog, update_worklog,
};

// Define a simple health check endpoint
#[get("/health")]
async fn health_check() -> impl Responder {
    HttpResponse::Ok().body("OK")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let pool = web::Data::new(establish_connection());

    HttpServer::new(move || {
        App::new()
            .wrap(
                Cors::default()
                    .allow_any_origin()
                    .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
                    .allow_any_header()
                    .supports_credentials()
                    .max_age(3600),
            )
            .app_data(pool.clone())
            .service(health_check)
            // Blog Post Endpoints
            .service(create_post)
            .service(get_post)
            .service(get_by_post_id)
            .service(get_all_posts)
            .service(update_post)
            .service(delete_post)
            .service(delete_all_posts)
            // .service(delete_all_posts_with_body)
            // Worklog Endpoints
            .service(create_worklog)
            .service(get_worklog)
            .service(update_worklog)
            .service(delete_worklog)
            .service(get_all_worklog)
            // Skills Endpoints
            .service(create_skill)
            .service(get_skill)
            .service(update_skill)
            .service(delete_skill)
            .service(get_all_skills)
    })
        .bind(format!("0.0.0.0:{}", port))?
        .run()
        .await
}
