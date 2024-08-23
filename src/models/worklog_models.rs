use diesel::{Insertable, Queryable};
use serde::{Deserialize, Serialize};

#[derive(Insertable, Queryable, Serialize, Deserialize)]
pub struct Worklog {
    pub id: i32,
    pub work_id: String,
    pub work_title: String,
    pub body: String,
    pub time_created: String,
    pub time_updated: String
}
