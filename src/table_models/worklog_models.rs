use chrono::NaiveDateTime;
use diesel::Queryable;
use serde::{Deserialize, Serialize};
#[derive(Queryable, Serialize, Deserialize)]
pub struct Worklog {
    pub id: i32,
    pub worklog_id: String,
    pub work_title: String,
    pub body: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

use crate::schemas::worklog_schema::worklog;
use diesel::Insertable;

#[derive(Insertable)]
#[diesel(table_name = worklog)]
pub struct NewWorklog {
    pub worklog_id: String,
    pub work_title: String,
    pub body: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
