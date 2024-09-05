use std::env;
use actix_cors::Cors;
use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use dotenv::dotenv;
use tarnish::connectors::postgres_connector::{DbConnector, RealDbConnector};
use tarnish::controllers::blog_controller::{
    create_post, delete_all_posts, delete_post, get_all_posts, get_by_post_id, get_post, update_post,
};
use tarnish::controllers::skills_controller::{
    create_skill, delete_skill, get_all_skills, get_by_skill_id, get_skill, update_skill,
};
use tarnish::controllers::worklog_controller::{
    create_worklog, delete_worklog, get_all_worklog, get_worklog, get_by_worklog_id, update_worklog,
};
use tarnish::controllers::register_user_controller::create_user;

// Define a simple health check endpoint
#[get("/health")]
async fn health_check() -> impl Responder {
    HttpResponse::Ok().body("OK")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());

    // Establish connection to the database
    let pool = match RealDbConnector.establish_connection() {
        Ok(pool) => web::Data::new(pool),
        Err(e) => {
            eprintln!("Failed to connect to the database: {}", e);
            return Err(std::io::Error::new(std::io::ErrorKind::Other, "Failed to connect to the database"));
        }
    };

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
            .app_data(pool.clone())  // Passing the connection pool to handlers
            .service(health_check)

            // Blog Post Endpoints
            .service(create_post)
            .service(get_post)
            .service(get_by_post_id)
            .service(get_all_posts)
            .service(update_post)
            .service(delete_post)
            .service(delete_all_posts)

            // Worklog Endpoints
            .service(create_worklog)
            .service(get_worklog)
            .service(get_by_worklog_id)
            .service(get_all_worklog)
            .service(update_worklog)
            .service(delete_worklog)

            // Skills Endpoints
            .service(create_skill)
            .service(get_skill)
            .service(get_by_skill_id)
            .service(update_skill)
            .service(delete_skill)
            .service(get_all_skills)

            // User Registration Endpoints
            .service(create_user)
    })
        .bind(format!("0.0.0.0:{}", port))?
        .run()
        .await
}
