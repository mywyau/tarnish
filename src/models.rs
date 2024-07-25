// src/models.rs

use diesel::prelude::*;
use diesel::Queryable;
use diesel::Insertable;
use diesel::QueryableByName;
use serde::{Deserialize, Serialize};
// For querying by name, if needed
use crate::schema::posts;


#[derive(Queryable, Serialize, Deserialize)]
pub struct Post {
    pub id: i32,
    pub title: String,
    pub body: String,
}

#[derive(Insertable, Serialize, Deserialize)]
#[diesel(table_name = posts)]
pub struct NewPost {
    pub title: String,
    pub body: String,
}
