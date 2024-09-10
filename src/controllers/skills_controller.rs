use actix_web::{delete, get, post, put, web, Error, HttpResponse};
use chrono::DateTime;
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use serde::{Deserialize, Serialize};
use serde_json::json;
use crate::schemas::skills_schema::skills;
use crate::table_models::skills_models::{NewSkill, Skill};

pub type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;
#[derive(Serialize, Deserialize)]
pub struct SkillInput {
    pub id: i32,
    pub skill_id: String,
    pub skill_name: String,
    pub body: String,
    pub created_at: String,
    pub updated_at: String,
}

impl SkillInput {
    pub fn new(id: i32, skill_id: String, skill_name: String, body: String, created_at: String, updated_at: String) -> Self {
        SkillInput {
            id,
            skill_id,
            skill_name,
            body,
            created_at,
            updated_at,
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
            created_at: DateTime::parse_from_rfc3339(&skill_input.created_at)
                .unwrap()
                .naive_utc(), // Convert to NaiveDateTime
            updated_at: DateTime::parse_from_rfc3339(&skill_input.updated_at)
                .unwrap()
                .naive_utc(), // Convert to NaiveDateTime
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


#[cfg(test)]
mod skills_controller_spec {
    // use tarnish::connectors::postgres_connector::DbPool;
    // use tarnish::controllers::skills_controller::{create_skill, delete_all_skills, delete_skill, get_all_skills, get_by_skill_id, update_skill};
    // use tarnish::schemas::skills_schema::skills;
    // use tarnish::{NewSkill, Skill};

    use actix_web::{body::to_bytes, http::StatusCode, test, web, App};
    use bytes::Bytes;
    use diesel::r2d2::{ConnectionManager, PooledConnection};
    use diesel::{r2d2, OptionalExtension, PgConnection, RunQueryDsl};
    use dotenv::dotenv;
    use serde_json::{json, Value};
    use std::env;
    // use tarnish::controllers::skills_controller::{delete_all_skills, get_all_skills, get_by_skill_id};
    // use tarnish::{create_skill, delete_skill, skills, update_skill, DbPool, NewSkill, Skill};
    use crate::connectors::postgres_connector::DbPool;
    use crate::controllers::skills_controller::{create_skill, delete_all_skills, delete_skill, get_all_skills, get_by_skill_id, update_skill};
    use crate::schemas::skills_schema::skills;
    use crate::table_models::skills_models::{NewSkill, Skill};

    #[ctor::ctor]
    fn init() {
        let pool = establish_connection();
        let mut conn: PooledConnection<ConnectionManager<PgConnection>> =
            pool.get().expect("Failed to get connection from pool");

        diesel::sql_query("TRUNCATE TABLE skills RESTART IDENTITY CASCADE;")
            .execute(&mut conn)
            .expect("Failed to reset ID sequence");
    }

    struct TestGuard {
        pool: web::Data<DbPool>,
        skill_ids: Vec<String>,
    }

    impl TestGuard {
        fn new(pool: web::Data<DbPool>, skills_to_insert: Vec<NewSkill>) -> Self {
            let mut conn: PooledConnection<ConnectionManager<PgConnection>> =
                pool.get().expect("Failed to get connection from pool");

            for skill in &skills_to_insert {
                diesel::insert_into(skills::table)
                    .values(skill)
                    .execute(&mut conn)
                    .expect("Failed to insert test skill");
            }

            let skill_ids = skills_to_insert.into_iter().map(|p| p.skill_id).collect();
            TestGuard { pool, skill_ids }
        }
    }

    impl Drop for TestGuard {
        fn drop(&mut self) {
            let mut conn: PooledConnection<ConnectionManager<PgConnection>> =
                self.pool.get().expect("Failed to get connection from pool");

            for skill_id in &self.skill_ids {
                diesel::delete(skills::table.filter(skills::skill_id.eq(skill_id)))
                    .execute(&mut conn)
                    .expect("Failed to delete test skill");
            }
        }
    }

    pub fn establish_connection() -> DbPool {
        dotenv().ok();
        let database_url = env::var("TEST_DATABASE_URL").expect("TEST_DATABASE_URL must be set");
        let manager = ConnectionManager::<PgConnection>::new(database_url);
        r2d2::Pool::builder().build(manager).expect("Failed to create pool.")
    }

    #[actix_rt::test]
    async fn run_all_tests_in_order_skill() {
        test_get_by_skill_id().await;
        test_get_all_skills().await;
        test_create_skill().await;
        test_update_skill().await;
        test_delete_skill().await;
        test_delete_all_skills().await;
    }

    async fn test_create_skill() {
        let pool = web::Data::new(establish_connection());

        let app =
            test::init_service(
                App::new()
                    .app_data(pool.clone())
                    .service(get_by_skill_id)
                    .service(create_skill)
                    .service(update_skill)
                    .service(delete_skill),
            )
                .await;

        let skills_to_insert = vec![
            NewSkill {
                skill_id: "skill1".to_string(),
                skill_name: "Python".to_string(),
                body: "Some content about the skill".to_string(),
                created_at: chrono::Utc::now().naive_utc(), // Current time in ISO 8601 format
                updated_at: chrono::Utc::now().naive_utc(), // Current time in ISO 8601 format
            },
            NewSkill {
                skill_id: "skill2".to_string(),
                skill_name: "Typescript".to_string(),
                body: "Some content about the skill 2".to_string(),
                created_at: chrono::Utc::now().naive_utc(), // Current time in ISO 8601 format
                updated_at: chrono::Utc::now().naive_utc(), // Current time in ISO 8601 format
            },
        ];

        let _guard = TestGuard::new(pool.clone(), skills_to_insert);

        let payload = json!({
            "id": 200,
            "skill_id": "skill3",
            "skill_name": "Rust",
            "body": "Some content about Rust",
            "created_at": "2023-08-29T14:00:00Z", // Example timestamp
            "updated_at": "2023-08-29T14:00:01Z"  // Example timestamp
        });

        let create_req =
            test::TestRequest::post()
                .uri("/blog/skill/create")
                .set_json(&payload)
                .to_request();

        let create_resp = test::call_service(&app, create_req).await;
        assert_eq!(create_resp.status(), StatusCode::CREATED);

        let body = test::read_body(create_resp).await;
        let body_str = std::str::from_utf8(&body).unwrap();
        let json_body: Value = serde_json::from_str(body_str).unwrap();

        let skill_id = json_body.get("skill_id").unwrap().as_str().unwrap();
        assert_eq!(skill_id, "skill3");
    }

    async fn test_get_by_skill_id() {
        let pool = web::Data::new(establish_connection());

        let app =
            test::init_service(
                App::new()
                    .app_data(pool.clone())
                    .service(get_by_skill_id)
                    .service(create_skill)
                    .service(update_skill)
                    .service(delete_skill),
            )
                .await;

        let skills_to_insert = vec![
            NewSkill {
                skill_id: "skill4".to_string(),
                skill_name: "Cats".to_string(),
                body: "This is the first skill.".to_string(),
                created_at: chrono::Utc::now().naive_utc(), // Current time in ISO 8601 format
                updated_at: chrono::Utc::now().naive_utc(), // Current time in ISO 8601 format
            },
            NewSkill {
                skill_id: "skill5".to_string(),
                skill_name: "Latex".to_string(),
                body: "This is the second skill.".to_string(),
                created_at: chrono::Utc::now().naive_utc(), // Current time in ISO 8601 format
                updated_at: chrono::Utc::now().naive_utc(), // Current time in ISO 8601 format
            },
        ];

        let _guard = TestGuard::new(pool.clone(), skills_to_insert);

        let req = test::TestRequest::get()
            .uri("/blog/skill/retrieve/skill-id/skill5")
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let body: Bytes = to_bytes(resp.into_body()).await.unwrap();
        let body_str = std::str::from_utf8(&body).unwrap();
        let json_body: Value = serde_json::from_str(body_str).unwrap();

        let skill_id_field = json_body.get("skill_id").unwrap().as_str().unwrap();
        let skill_name_field = json_body.get("skill_name").unwrap().as_str().unwrap();
        let body_field = json_body.get("body").unwrap().as_str().unwrap();

        assert_eq!(skill_id_field, "skill5");
        assert_eq!(skill_name_field, "Latex");
        assert_eq!(body_field, "This is the second skill.");
    }

    async fn test_get_all_skills() {
        let pool = web::Data::new(establish_connection());

        let app = test::init_service(
            App::new()
                .app_data(pool.clone())
                .service(get_all_skills),
        )
            .await;

        let skills_to_insert = vec![
            NewSkill {
                skill_id: "skill10".to_string(),
                skill_name: "Raking Leaves".to_string(),
                body: "Some content 1".to_string(),
                created_at: chrono::Utc::now().naive_utc(), // Current time in ISO 8601 format
                updated_at: chrono::Utc::now().naive_utc(), // Current time in ISO 8601 format
            },
            NewSkill {
                skill_id: "skill11".to_string(),
                skill_name: "Cutting Potatoes".to_string(),
                body: "Some content 2".to_string(),
                created_at: chrono::Utc::now().naive_utc(), // Current time in ISO 8601 format
                updated_at: chrono::Utc::now().naive_utc(), // Current time in ISO 8601 format
            },
            NewSkill {
                skill_id: "skill12".to_string(),
                skill_name: "Farming Pigeons".to_string(),
                body: "Some content 3".to_string(),
                created_at: chrono::Utc::now().naive_utc(), // Current time in ISO 8601 format
                updated_at: chrono::Utc::now().naive_utc(), // Current time in ISO 8601 format
            },
        ];

        let _guard = TestGuard::new(pool.clone(), skills_to_insert);

        let req = test::TestRequest::get()
            .uri("/blog/skill/get/all")  // Ensure this matches your actual route
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let body: Bytes = to_bytes(resp.into_body()).await.unwrap();
        let json_body: Value = serde_json::from_slice(&body).unwrap();

        // Assuming the response is an array of skills
        let skill_json_array = json_body.as_array().expect("Expected an array of skills");

        // Assert the length of the array
        assert_eq!(skill_json_array.len(), 3);

        // Assert the content of each skill
        assert_eq!(skill_json_array[0]["skill_id"], "skill10");
        assert_eq!(skill_json_array[0]["skill_name"], "Raking Leaves");
        assert_eq!(skill_json_array[0]["body"], "Some content 1");

        assert_eq!(skill_json_array[1]["skill_id"], "skill11");
        assert_eq!(skill_json_array[1]["skill_name"], "Cutting Potatoes");
        assert_eq!(skill_json_array[1]["body"], "Some content 2");

        assert_eq!(skill_json_array[2]["skill_id"], "skill12");
        assert_eq!(skill_json_array[2]["skill_name"], "Farming Pigeons");
        assert_eq!(skill_json_array[2]["body"], "Some content 3");
    }

    async fn test_update_skill() {
        let pool = web::Data::new(establish_connection());

        let app =
            test::init_service(
                App::new()
                    .app_data(pool.clone())
                    .service(get_by_skill_id)
                    .service(create_skill)
                    .service(update_skill)
                    .service(delete_skill),
            )
                .await;

        let skills_to_insert = vec![
            NewSkill {
                skill_id: "skill_25".to_string(),
                skill_name: "Eating Watermelon 3".to_string(),
                body: "Fake content".to_string(),
                created_at: chrono::Utc::now().naive_utc(), // Current time in ISO 8601 format
                updated_at: chrono::Utc::now().naive_utc(), // Current time in ISO 8601 format
            },
        ];

        let _guard = TestGuard::new(pool.clone(), skills_to_insert);

        let payload = json!({
            "id": 1,
            "skill_id": "skill_25",
            "skill_name": "Eating Onions",
            "body": "Updated body content.",
            "created_at": "2023-08-29T14:00:00Z", // Example timestamp
            "updated_at": "2023-08-29T14:00:01Z"  // Example timestamp
        });

        let put_req = test::TestRequest::put()
            .uri("/blog/skill/update/skill_25")
            .set_json(&payload)
            .to_request();

        let put_resp = test::call_service(&app, put_req).await;
        assert!(put_resp.status().is_success());

        let body: Bytes = to_bytes(put_resp.into_body()).await.unwrap();
        let body_str = std::str::from_utf8(&body).unwrap();
        let json_body: Value = serde_json::from_str(body_str).unwrap();

        let expected_message = json!({
            "message": "Skill 'Eating Watermelon 3' has been updated"
        });

        assert_eq!(json_body, expected_message);
    }

    async fn test_delete_skill() {
        let pool = web::Data::new(establish_connection());

        let app =
            test::init_service(
                App::new()
                    .app_data(pool.clone())
                    .service(get_by_skill_id)
                    .service(create_skill)
                    .service(update_skill)
                    .service(delete_skill),
            )
                .await;

        let skills_to_insert = vec![
            NewSkill {
                skill_id: "abc200".to_string(),
                skill_name: "Fake Skill".to_string(),
                body: "This is the first test skill.".to_string(),
                created_at: chrono::Utc::now().naive_utc(), // Current time in ISO 8601 format
                updated_at: chrono::Utc::now().naive_utc(), // Current time in ISO 8601 format
            },
            NewSkill {
                skill_id: "def456".to_string(),
                skill_name: "Fake Skill 2".to_string(),
                body: "This is the second test skill.".to_string(),
                created_at: chrono::Utc::now().naive_utc(), // Current time in ISO 8601 format
                updated_at: chrono::Utc::now().naive_utc(), // Current time in ISO 8601 format
            },
        ];

        let _guard = TestGuard::new(pool.clone(), skills_to_insert);

        let delete_request = test::TestRequest::delete()
            .uri("/blog/skill/single/abc200")
            .to_request();

        let delete_response = test::call_service(&app, delete_request).await;
        assert!(delete_response.status().is_success());

        let mut conn: PooledConnection<ConnectionManager<PgConnection>> =
            pool.get().expect("Failed to get connection from pool");

        let deleted_skill_1 =
            skills::table
                .filter(skills::skill_id.eq("abc200"))
                .first::<Skill>(&mut conn)
                .optional()
                .expect("Failed to check for deleted skill");

        assert!(deleted_skill_1.is_none());

        let deleted_skill_2 =
            skills::table
                .filter(skills::skill_id.eq("def456"))
                .first::<Skill>(&mut conn)
                .optional()
                .expect("Failed to check for deleted skill");

        assert!(deleted_skill_2.is_some());
    }

    async fn test_delete_all_skills() {
        let pool = web::Data::new(establish_connection());

        let app =
            test::init_service(
                App::new()
                    .app_data(pool.clone())
                    .service(get_by_skill_id)
                    .service(create_skill)
                    .service(update_skill)
                    .service(delete_all_skills),
            )
                .await;

        let skills_to_insert = vec![
            NewSkill {
                skill_id: "fake_id_1".to_string(),
                skill_name: "Fake Skill 1".to_string(),
                body: "This is the first skill.".to_string(),
                created_at: chrono::Utc::now().naive_utc(), // Current time in ISO 8601 format
                updated_at: chrono::Utc::now().naive_utc(), // Current time in ISO 8601 format
            },
            NewSkill {
                skill_id: "fake_id_2".to_string(),
                skill_name: "Fake Skill 2".to_string(),
                body: "This is the second skill.".to_string(),
                created_at: chrono::Utc::now().naive_utc(), // Current time in ISO 8601 format
                updated_at: chrono::Utc::now().naive_utc(), // Current time in ISO 8601 format
            },
        ];

        let _guard = TestGuard::new(pool.clone(), skills_to_insert);

        let delete_request =
            test::TestRequest::delete()
                .uri("/blog/skill/all")
                .to_request();

        let delete_response =
            test::call_service(&app, delete_request).await;

        assert!(delete_response.status().is_success());

        let mut conn: PooledConnection<ConnectionManager<PgConnection>> =
            pool.get().expect("Failed to get connection from pool");

        let deleted_skill_1 = skills::table
            .filter(skills::skill_id.eq("fake_id_1"))
            .first::<Skill>(&mut conn)
            .optional()
            .expect("Failed to check for deleted skill");

        assert!(deleted_skill_1.is_none());

        let deleted_skill_2 = skills::table
            .filter(skills::skill_id.eq("fake_id_2"))
            .first::<Skill>(&mut conn)
            .optional()
            .expect("Failed to check for deleted skill");

        assert!(deleted_skill_2.is_none());
    }
}
