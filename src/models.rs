use diesel::{Insertable, Queryable};
use serde::{Deserialize, Serialize};

use crate::schema::posts;

#[derive(Insertable, Queryable, Serialize, Deserialize)]
pub struct Post {
    pub id: i32,
    pub post_id: String,
    pub title: String,
    pub body: String,
}

#[derive(Insertable, Queryable, Serialize, Deserialize)]
#[diesel(table_name = posts)]
pub struct NewPost {
    pub id: i32,
    pub post_id: String,
    pub title: String,
    pub body: String,
}

// #[derive(Insertable, Serialize, Deserialize)]
// #[table_name = "posts"]
// pub struct NewPost<'a> {
//     pub id: i32,
//     pub post_id: &'a str,
//     pub title: &'a str,
//     pub body: &'a str,
// }