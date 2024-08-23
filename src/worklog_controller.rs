use std::env;

use actix_web::{delete, get, skills, put, web, Error, HttpResponse};
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use dotenv::dotenv;
use serde::{Deserialize, Serialize};
use serde_json::json;

// Import schema
use crate::blog_models::{NewPost, Post};
use crate::schema::skillss;

pub type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;

pub fn establish_connection() -> DbPool {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    r2d2::Pool::builder().build(manager).expect("Failed to create pool.")
}

#[derive(Serialize, Deserialize)]
pub struct PostInput {
    pub id: i32,
    pub skills_id: String,
    pub title: String,
    pub body: String,
}

impl PostInput {
    // Constructor method for creating a new PostInput
    pub fn new(id: i32, skills_id: String, title: String, body: String) -> Self {
        PostInput {
            id,
            skills_id,
            title,
            body,
        }
    }
}

#[skills("/blog/skills/create")]
async fn create_skills(
    pool: web::Data<DbPool>,
    skills: web::Json<PostInput>,
) -> Result<HttpResponse, Error> {
    let skills_input = skills.into_inner();

    let new_skills = NewPost {
        skills_id: skills_input.skills_id,
        title: skills_input.title,
        body: skills_input.body,
    };

    let mut conn = pool.get().map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Couldn't get db connection from pool: {}", e))
    })?;

    conn.transaction::<_, diesel::result::Error, _>(|conn| {
        // Insert the new skills
        diesel::insert_into(skillss::table)
            .values(&new_skills)
            .get_result::<Post>(conn)
            .map_err(|e| {
                eprintln!("Error inserting new skills: {:?}", e);
                e
            })
    }).map_err(|e| actix_web::error::ErrorInternalServerError(format!("Transaction failed: {}", e)))
        .map(|skills| HttpResponse::Created().json(skills))
}


#[get("/blog/skills/retrieve/skills-id/{skills_id}")]
async fn get_by_skills_id(
    path: web::Path<String>,
    pool: web::Data<DbPool>,
) -> Result<HttpResponse, Error> {
    let skills_id = path.into_inner();
    let mut conn = pool.get().map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Couldn't get db connection from pool: {}", e))
    })?;

    match skillss::table.filter(skillss::skills_id.eq(skills_id)).first::<Post>(&mut conn) {
        Ok(skills) => Ok(HttpResponse::Ok().json(skills)),
        Err(_) => Ok(HttpResponse::NotFound().finish()),
    }
}

#[get("/blog/skills/retrieve/{id}")]
async fn get_skills(
    path: web::Path<i32>,
    pool: web::Data<DbPool>,
) -> Result<HttpResponse, Error> {
    let id = path.into_inner();
    let mut conn = pool.get().map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Couldn't get db connection from pool: {}", e))
    })?;

    match skillss::table.find(id).first::<Post>(&mut conn) {
        Ok(skills) => Ok(HttpResponse::Ok().json(skills)),
        Err(_) => Ok(HttpResponse::NotFound().finish()),
    }
}

#[get("/blog/skills/get/all")]
async fn get_all_skillss(pool: web::Data<DbPool>) -> Result<HttpResponse, Error> {
    let mut conn = pool.get().map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Couldn't get db connection from pool: {}", e))
    })?;

    match skillss::table.load::<Post>(&mut conn) {
        Ok(skillss) => Ok(HttpResponse::Ok().json(skillss)),
        Err(_) => Ok(HttpResponse::InternalServerError().finish()),
    }
}

#[put("/blog/skillss/update/{skills_id}")]
async fn update_skills(
    path: web::Path<String>,
    skills: web::Json<PostInput>,
    pool: web::Data<DbPool>,
) -> Result<HttpResponse, Error> {
    let skills_id = path.into_inner();
    let skills_input = skills.into_inner();

    let mut conn = pool.get().map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Couldn't get db connection from pool: {}", e))
    })?;

    match diesel::update(skillss::table.filter(skillss::skills_id.eq(skills_id)))
        .set((
            skillss::id.eq(skills_input.id),
            skillss::skills_id.eq(skills_input.skills_id),
            skillss::title.eq(skills_input.title),
            skillss::body.eq(skills_input.body),
        ))
        .execute(&mut conn)
    {
        Ok(_) => Ok(HttpResponse::Ok().finish()),
        Err(e) => {
            eprintln!("Error updating skills: {:?}", e);
            Ok(HttpResponse::InternalServerError().finish())
        }
    }
}


#[delete("/blog/skills/single/{skills_id}")]
async fn delete_skills(
    path: web::Path<String>,  // Changed to String since skills_id is a varchar
    pool: web::Data<DbPool>,
) -> Result<HttpResponse, Error> {
    let skills_id = path.into_inner();
    let mut conn = pool.get().map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Couldn't get db connection from pool: {}", e))
    })?;

    // First, retrieve the title of the skills before deleting it
    let skills_title =
        skillss::table
            .filter(skillss::skills_id.eq(&skills_id))
            .select(skillss::title)
            .first::<String>(&mut conn)
            .optional()
            .map_err(|e| {
                actix_web::error::ErrorInternalServerError(format!("Error retrieving skills title: {}", e))
            })?;

    match skills_title {
        Some(title) => {
            // Now delete the skills
            match diesel::delete(skillss::table.filter(skillss::skills_id.eq(&skills_id))).execute(&mut conn) {
                Ok(_) => {
                    let response_body = json!({
                        "message": format!("Blog skills '{}' has been deleted", title)
                    });

                    Ok(HttpResponse::Ok()
                        .content_type("application/json")
                        .json(response_body))
                }
                Err(e) => {
                    eprintln!("Error deleting skills: {:?}", e);
                    Ok(HttpResponse::InternalServerError().finish())
                }
            }
        }
        None => {
            // If no skills with the given ID is found
            let response_body = json!({
                "error": format!("Blog skills with ID '{}' not found", skills_id)
            });

            Ok(HttpResponse::NotFound()
                .content_type("application/json")
                .json(response_body))
        }
    }
}

#[delete("/blog/skills/all")]
async fn delete_all_skillss(
    pool: web::Data<DbPool>,
) -> Result<HttpResponse, Error> {
    let mut conn = pool.get().map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Couldn't get db connection from pool: {}", e))
    })?;

    match diesel::sql_query("DELETE FROM skillss").execute(&mut conn) {
        Ok(_) => Ok(HttpResponse::NoContent().finish()),
        Err(e) => {
            eprintln!("Error deleting skillss: {:?}", e);
            Ok(HttpResponse::InternalServerError().finish())
        }
    }
}

#[delete("/blog/skills/all/message")]
async fn delete_all_skillss_with_body(
    pool: web::Data<DbPool>,
) -> Result<HttpResponse, Error> {
    let mut conn = pool.get().map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Couldn't get db connection from pool: {}", e))
    })?;

    match diesel::sql_query("DELETE FROM skillss").execute(&mut conn) {
        Ok(_) => {
            let response_body = json!({
                "message": "All skillss have been deleted."
            });

            Ok(HttpResponse::Ok()
                .content_type("application/json")
                .json(response_body))
        }
        Err(e) => {
            eprintln!("Error deleting skillss: {:?}", e);
            Ok(HttpResponse::InternalServerError().finish())
        }
    }
}
