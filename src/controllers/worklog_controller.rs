use std::env;

use actix_web::{delete, get, post, put, web, Error, HttpResponse};
use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use dotenv::dotenv;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::connectors::postgres_connector::DbPool;
use crate::models::worklog_models::NewWorklog;
use crate::models::worklog_models::Worklog;
use crate::worklog;

#[derive(Serialize, Deserialize)]
pub struct WorklogInput {
    pub id: i32,
    pub worklog_id: String,
    pub work_title: String,
    pub body: String,
    pub time_created: NaiveDateTime,  // Allow for optional input
    pub time_updated: NaiveDateTime,  // Allow for optional input
}


impl WorklogInput {
    // Constructor method for creating a new WorklogInput
    pub fn new(id: i32, worklog_id: String, work_title: String, body: String, time_created: NaiveDateTime, time_updated: NaiveDateTime) -> Self {
        WorklogInput {
            id,
            worklog_id,
            work_title,
            body,
            time_created,
            time_updated,
        }
    }
}


#[post("/blog/worklog/create")]
async fn create_worklog(
    pool: web::Data<DbPool>,
    worklog: web::Json<WorklogInput>,
) -> Result<HttpResponse, Error> {
    let worklog_input = worklog.into_inner();

    let new_worklog = NewWorklog {
        worklog_id: worklog_input.worklog_id,
        worklog_title: worklog_input.work_title,
        body: worklog_input.body,
        created_at: chrono::Utc::now().naive_utc(),
        updated_at: chrono::Utc::now().naive_utc(),
    };

    let mut conn = pool.get().map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Couldn't get db connection from pool: {}", e))
    })?;

    conn.transaction::<_, diesel::result::Error, _>(|conn| {
        // Insert the new worklog
        diesel::insert_into(worklog::table)
            .values(&new_worklog)
            .get_result::<Worklog>(conn)  // This can return the inserted record with the `id`
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

    match worklog::table.filter(worklog::worklog_id.eq(worklog_id)).first::<Worklog>(&mut conn) {
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

    match worklog::table.find(id).first::<Worklog>(&mut conn) {
        Ok(worklog) => Ok(HttpResponse::Ok().json(worklog)),
        Err(diesel::result::Error::NotFound) => Ok(HttpResponse::NotFound().finish()),
        Err(_) => Ok(HttpResponse::InternalServerError().finish()),
    }
}


#[get("/blog/worklog/get/all")]
async fn get_all_worklog(pool: web::Data<DbPool>) -> Result<HttpResponse, Error> {
    // Get a connection from the pool
    let mut conn = pool.get().map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Couldn't get db connection from pool: {}", e))
    })?;

    // Query all worklogs from the database
    match worklog::table.load::<Worklog>(&mut conn) {
        Ok(worklogs) => Ok(HttpResponse::Ok().json(worklogs)),
        Err(_) => Ok(HttpResponse::InternalServerError().finish()),
    }
}

#[put("/blog/worklog/update/{worklog_id}")]
async fn update_worklog(
    path: web::Path<String>,
    worklog: web::Json<WorklogInput>,
    pool: web::Data<DbPool>,
) -> Result<HttpResponse, Error> {
    let worklog_id = path.into_inner();
    let worklog_input = worklog.into_inner();

    let mut conn = pool.get().map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Couldn't get db connection from pool: {}", e))
    })?;

    match diesel::update(worklog::table.filter(worklog::worklog_id.eq(worklog_id)))
        .set((
            worklog::id.eq(worklog_input.id),
            worklog::worklog_id.eq(worklog_input.worklog_id),
            worklog::worklog_title.eq(worklog_input.work_title),
            worklog::body.eq(worklog_input.body),
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
        worklog::table
            .filter(worklog::worklog_id.eq(&worklog_id))
            .select(worklog::worklog_title)
            .first::<String>(&mut conn)
            .optional()
            .map_err(|e| {
                actix_web::error::ErrorInternalServerError(format!("Error retrieving worklog title: {}", e))
            })?;

    match worklog_title {
        Some(title) => {
            // Now delete the worklog
            match diesel::delete(worklog::table.filter(worklog::worklog_id.eq(&worklog_id))).execute(&mut conn) {
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

    match diesel::sql_query("DELETE FROM worklog").execute(&mut conn) {
        Ok(_) => Ok(HttpResponse::NoContent().finish()),
        Err(e) => {
            eprintln!("Error deleting worklog: {:?}", e);
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

    match diesel::sql_query("DELETE FROM worklog").execute(&mut conn) {
        Ok(_) => {
            let response_body = json!({
                "message": "All worklog have been deleted."
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
