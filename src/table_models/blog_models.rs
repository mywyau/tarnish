use chrono::NaiveDateTime;
use diesel::{Insertable, Queryable};
use serde::{Deserialize, Serialize};

use crate::schemas::blog_schema::posts;

#[derive(Insertable, Queryable, Serialize, Deserialize)]
pub struct Post {
    pub id: i32,
    pub post_id: String,
    pub title: String,
    pub body: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Insertable, Queryable, Serialize, Deserialize)]
#[diesel(table_name = posts)]
pub struct NewPost {
    // pub id: i32,
    pub post_id: String,
    pub title: String,
    pub body: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}