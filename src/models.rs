use diesel::{Insertable, Queryable};
use serde::{Deserialize, Serialize};

use crate::schema::posts;

#[derive(Queryable, Serialize, Deserialize)]
pub struct Post {
    pub id: i32,
    pub post_id: String,
    pub title: String,
    pub body: String,
}

#[derive(Insertable)]
#[diesel(table_name = posts)]
pub struct NewPost {
    pub id: i32,
    pub post_id: String,
    pub title: String,
    pub body: String,
}
