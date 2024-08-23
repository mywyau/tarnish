use std::env;

use actix_web::{delete, get, worklog, put, web, Error, HttpResponse};
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use dotenv::dotenv;
use serde::{Deserialize, Serialize};
use serde_json::json;

// Import schema
use crate::blog_models::{NewWorkLog, WorkLog};
use crate::blog_schema::worklogs;

pub type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;

pub fn establish_connection() -> DbPool {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    r2d2::Pool::builder().build(manager).expect("Failed to create pool.")
}

#[derive(Serialize, Deserialize)]
pub struct WorkLogInput {
    pub id: i32,
    pub worklog_id: String,
    pub work_title: String,
    pub title: String,
    pub body: String,
    pub time_created: String,
    pub time_updated: String
}



impl WorkLogInput {
    // Constructor method for creating a new WorkLogInput
    pub fn new(id: i32, worklog_id: String, work_title: String, body: String, time_created:String, time_updated:String) -> Self {
        WorkLogInput {
            id,
            worklog_id,
            work_title,
            body,
            time_created,
            time_updated
        }
    }
}

#[worklog("/blog/worklog/create")]
async fn create_worklog(
    pool: web::Data<DbPool>,
    worklog: web::Json<WorkLogInput>,
) -> Result<HttpResponse, Error> {
    let worklog_input = worklog.into_inner();

    let new_worklog = NewWorkLog {
        worklog_id: worklog_input.worklog_id,
        title: worklog_input.title,
        body: worklog_input.body,
    };

    let mut conn = pool.get().map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Couldn't get db connection from pool: {}", e))
    })?;

    conn.transaction::<_, diesel::result::Error, _>(|conn| {
        // Insert the new worklog
        diesel::insert_into(worklogs::table)
            .values(&new_worklog)
            .get_result::<WorkLog>(conn)
            .map_err(|e| {
                eprintln!("Error inserting new worklog: {:?}", e);
                e
            })
    }).map_err(|e| actix_web::error::ErrorInternalServerError(format!("Transaction failed: {}", e)))
        .map(|worklog| HttpResponse::Created().json(worklog))
}

#[get("/blog/worklog/retrieve/worklog-id/{worklog_id}")]
async fn get_by_worklog_id(
    path: web::Path<String>,
    pool: web::Data<DbPool>,
) -> Result<HttpResponse, Error> {
    let worklog_id = path.into_inner();
    let mut conn = pool.get().map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Couldn't get db connection from pool: {}", e))
    })?;

    match worklogs::table.filter(worklogs::worklog_id.eq(worklog_id)).first::<WorkLog>(&mut conn) {
        Ok(worklog) => Ok(HttpResponse::Ok().json(worklog)),
        Err(_) => Ok(HttpResponse::NotFound().finish()),
    }
}

#[get("/blog/worklog/retrieve/{id}")]
async fn get_worklog(
    path: web::Path<i32>,
    pool: web::Data<DbPool>,
) -> Result<HttpResponse, Error> {
    let id = path.into_inner();
    let mut conn = pool.get().map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Couldn't get db connection from pool: {}", e))
    })?;

    match worklogs::table.find(id).first::<WorkLog>(&mut conn) {
        Ok(worklog) => Ok(HttpResponse::Ok().json(worklog)),
        Err(_) => Ok(HttpResponse::NotFound().finish()),
    }
}

#[get("/blog/worklog/get/all")]
async fn get_all_worklogs(pool: web::Data<DbPool>) -> Result<HttpResponse, Error> {
    let mut conn = pool.get().map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Couldn't get db connection from pool: {}", e))
    })?;

    match worklogs::table.load::<WorkLog>(&mut conn) {
        Ok(worklogs) => Ok(HttpResponse::Ok().json(worklogs)),
        Err(_) => Ok(HttpResponse::InternalServerError().finish()),
    }
}

#[put("/blog/worklogs/update/{worklog_id}")]
async fn update_worklog(
    path: web::Path<String>,
    worklog: web::Json<WorkLogInput>,
    pool: web::Data<DbPool>,
) -> Result<HttpResponse, Error> {
    let worklog_id = path.into_inner();
    let worklog_input = worklog.into_inner();

    let mut conn = pool.get().map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Couldn't get db connection from pool: {}", e))
    })?;

    match diesel::update(worklogs::table.filter(worklogs::worklog_id.eq(worklog_id)))
        .set((
            worklogs::id.eq(worklog_input.id),
            worklogs::worklog_id.eq(worklog_input.worklog_id),
            worklogs::title.eq(worklog_input.title),
            worklogs::body.eq(worklog_input.body),
        ))
        .execute(&mut conn)
    {
        Ok(_) => Ok(HttpResponse::Ok().finish()),
        Err(e) => {
            eprintln!("Error updating worklog: {:?}", e);
            Ok(HttpResponse::InternalServerError().finish())
        }
    }
}


#[delete("/blog/worklog/single/{worklog_id}")]
async fn delete_worklog(
    path: web::Path<String>,  // Changed to String since worklog_id is a varchar
    pool: web::Data<DbPool>,
) -> Result<HttpResponse, Error> {
    let worklog_id = path.into_inner();
    let mut conn = pool.get().map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Couldn't get db connection from pool: {}", e))
    })?;

    // First, retrieve the title of the worklog before deleting it
    let worklog_title =
        worklogs::table
            .filter(worklogs::worklog_id.eq(&worklog_id))
            .select(worklogs::title)
            .first::<String>(&mut conn)
            .optional()
            .map_err(|e| {
                actix_web::error::ErrorInternalServerError(format!("Error retrieving worklog title: {}", e))
            })?;

    match worklog_title {
        Some(title) => {
            // Now delete the worklog
            match diesel::delete(worklogs::table.filter(worklogs::worklog_id.eq(&worklog_id))).execute(&mut conn) {
                Ok(_) => {
                    let response_body = json!({
                        "message": format!("Blog worklog '{}' has been deleted", title)
                    });

                    Ok(HttpResponse::Ok()
                        .content_type("application/json")
                        .json(response_body))
                }
                Err(e) => {
                    eprintln!("Error deleting worklog: {:?}", e);
                    Ok(HttpResponse::InternalServerError().finish())
                }
            }
        }
        None => {
            // If no worklog with the given ID is found
            let response_body = json!({
                "error": format!("Blog worklog with ID '{}' not found", worklog_id)
            });

            Ok(HttpResponse::NotFound()
                .content_type("application/json")
                .json(response_body))
        }
    }
}

#[delete("/blog/worklog/all")]
async fn delete_all_worklog(
    pool: web::Data<DbPool>,
) -> Result<HttpResponse, Error> {
    let mut conn = pool.get().map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Couldn't get db connection from pool: {}", e))
    })?;

    match diesel::sql_query("DELETE FROM worklogs").execute(&mut conn) {
        Ok(_) => Ok(HttpResponse::NoContent().finish()),
        Err(e) => {
            eprintln!("Error deleting worklogs: {:?}", e);
            Ok(HttpResponse::InternalServerError().finish())
        }
    }
}

#[delete("/blog/worklog/all/message")]
async fn delete_all_worklog_with_body(
    pool: web::Data<DbPool>,
) -> Result<HttpResponse, Error> {
    let mut conn = pool.get().map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Couldn't get db connection from pool: {}", e))
    })?;

    match diesel::sql_query("DELETE FROM worklogs").execute(&mut conn) {
        Ok(_) => {
            let response_body = json!({
                "message": "All worklogs have been deleted."
            });

            Ok(HttpResponse::Ok()
                .content_type("application/json")
                .json(response_body))
        }
        Err(e) => {
            eprintln!("Error deleting worklogs: {:?}", e);
            Ok(HttpResponse::InternalServerError().finish())
        }
    }
}
