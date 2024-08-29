use std::env;

use actix_web::{delete, get, post, put, web, Error, HttpResponse};
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use dotenv::dotenv;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::schemas::skills_schema::skills;
use crate::{posts, NewSkill, Skill};

pub type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;
#[derive(Serialize, Deserialize)]
pub struct SkillInput {
    pub id: i32,
    pub skill_id: String,
    pub skill_name: String,
    pub body: String,
}

impl SkillInput {
    pub fn new(id: i32, skill_id: String, skill_name: String, body: String) -> Self {
        SkillInput {
            id,
            skill_id,
            skill_name,
            body,
        }
    }
}

#[post("/blog/skill/create")]
async fn create_skill(
    pool: web::Data<DbPool>,
    skill: web::Json<SkillInput>,
) -> Result<HttpResponse, Error> {
    let skill_input = skill.into_inner();

    let new_skill =
        NewSkill {
            skill_id: skill_input.skill_id,
            skill_name: skill_input.skill_name,
            body: skill_input.body,
        };

    let mut conn = pool.get().map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Couldn't get db connection from pool: {}", e))
    })?;

    conn.transaction::<_, diesel::result::Error, _>(|conn| {
        // Insert the new skill
        diesel::insert_into(skills::table)
            .values(&new_skill)
            .get_result::<Skill>(conn)
            .map_err(|e| {
                eprintln!("Error inserting new skill: {:?}", e);
                e
            })
    }).map_err(|e| actix_web::error::ErrorInternalServerError(format!("Transaction failed: {}", e)))
        .map(|skill| HttpResponse::Created().json(skill))
}


#[get("/blog/skill/retrieve/skill-id/{skill_id}")]
async fn get_by_skill_id(
    path: web::Path<String>,
    pool: web::Data<DbPool>,
) -> Result<HttpResponse, Error> {
    let skill_id_path = path.into_inner();
    let mut conn = pool.get().map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Couldn't get db connection from pool: {}", e))
    })?;

    match skills::table.filter(skills::skill_id.eq(skill_id_path)).first::<Skill>(&mut conn) {
        Ok(skill) => Ok(HttpResponse::Ok().json(skill)),
        Err(_) => Ok(HttpResponse::NotFound().finish()),
    }
}

#[get("/blog/skill/retrieve/id/{id}")]
async fn get_skill(
    path: web::Path<i32>,
    pool: web::Data<DbPool>,
) -> Result<HttpResponse, Error> {
    let id = path.into_inner();
    let mut conn = pool.get().map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Couldn't get db connection from pool: {}", e))
    })?;

    match skills::table.find(id).first::<Skill>(&mut conn) {
        Ok(skill) => Ok(HttpResponse::Ok().json(skill)),
        Err(_) => Ok(HttpResponse::NotFound().finish()),
    }
}

#[get("/blog/skill/get/all")]
async fn get_all_skills(pool: web::Data<DbPool>) -> Result<HttpResponse, Error> {
    let mut conn = pool.get().map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Couldn't get db connection from pool: {}", e))
    })?;

    match skills::table.load::<Skill>(&mut conn) {
        Ok(skills) => Ok(HttpResponse::Ok().json(skills)),
        Err(_) => Ok(HttpResponse::InternalServerError().finish()),
    }
}

#[put("/blog/skill/update/{skill_id}")]
async fn update_skill(
    path: web::Path<String>,
    skill: web::Json<SkillInput>,
    pool: web::Data<DbPool>,
) -> Result<HttpResponse, Error> {
    let skill_id_path = path.into_inner();
    let skill_input = skill.into_inner();

    let mut conn = pool.get().map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Couldn't get db connection from pool: {}", e))
    })?;

    // First, retrieve the title of the post before deleting it
    let skill_name =
        skills::table
            .filter(skills::skill_id.eq(&skill_id_path))
            .select(skills::skill_name)
            .first::<String>(&mut conn)
            .optional()
            .map_err(|e| {
                actix_web::error::ErrorInternalServerError(format!("Error retrieving skill name: {}", e))
            })?;

    match skill_name {
        Some(skill) => {
            match diesel::update(skills::table.filter(skills::skill_id.eq(skill_id_path)))
                .set((
                    skills::id.eq(skill_input.id),
                    skills::skill_id.eq(skill_input.skill_id),
                    skills::skill_name.eq(skill_input.skill_name),
                    skills::body.eq(skill_input.body),
                ))
                .execute(&mut conn)
            {
                Ok(_) => {
                    let response_body =
                        json!({"message": format!("Skill '{}' has been updated", skill)});
                    Ok(HttpResponse::Ok()
                        .content_type("application/json")
                        .json(response_body))
                }
                Err(e) => {
                    eprintln!("Error updating post: {:?}", e);
                    Ok(HttpResponse::InternalServerError().finish())
                }
            }
        }
        None => {
            // If no skill with the given ID is found
            let response_body = json!({
                "error": format!("Skill with ID '{}' not found", skill_id_path)
            });

            Ok(HttpResponse::NotFound()
                .content_type("application/json")
                .json(response_body))
        }
    }
}


#[delete("/blog/skill/single/{skill_id}")]
async fn delete_skill(
    path: web::Path<String>,  // Changed to String since skill_id is a varchar
    pool: web::Data<DbPool>,
) -> Result<HttpResponse, Error> {
    let skill_id_path = path.into_inner();
    let mut conn = pool.get().map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Couldn't get db connection from pool: {}", e))
    })?;

    // First, retrieve the skill name before deleting it
    let skill_name =
        skills::table
            .filter(skills::skill_id.eq(&skill_id_path))
            .select(skills::skill_name)
            .first::<String>(&mut conn)
            .optional()
            .map_err(|e| {
                actix_web::error::ErrorInternalServerError(format!("Error retrieving skill name: {}", e))
            })?;

    match skill_name {
        Some(skill_name) => {
            // Now delete the skill
            match diesel::delete(skills::table.filter(skills::skill_id.eq(&skill_id_path))).execute(&mut conn) {
                Ok(_) => {
                    let response_body = json!({
                        "message": format!("Skill '{}' has been deleted", skill_name)
                    });

                    Ok(HttpResponse::Ok()
                        .content_type("application/json")
                        .json(response_body))
                }
                Err(e) => {
                    eprintln!("Error deleting skill: {:?}", e);
                    Ok(HttpResponse::InternalServerError().finish())
                }
            }
        }
        None => {
            // If no skill with the given ID is found
            let response_body = json!({
                "error": format!("Skill with ID '{}' not found", skill_id_path)
            });

            Ok(HttpResponse::NotFound()
                .content_type("application/json")
                .json(response_body))
        }
    }
}

#[delete("/blog/skill/all")]
async fn delete_all_skills(
    pool: web::Data<DbPool>,
) -> Result<HttpResponse, Error> {
    let mut conn = pool.get().map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Couldn't get db connection from pool: {}", e))
    })?;

    match diesel::sql_query("DELETE FROM skills").execute(&mut conn) {
        Ok(_) => {
            let response_body = json!({
                "message": "All skills have been deleted."
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
