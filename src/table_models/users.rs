use chrono::NaiveDateTime;
use diesel::{Insertable, Queryable};
use serde::{Deserialize, Serialize};

use crate::schemas::user_schema::users;

#[derive(Queryable, Serialize, Deserialize)]
#[diesel(table_name = users)]  // Ensure this points to the correct table in your schema
pub struct Users {
    pub id: i32,
    pub user_id: String,
    pub user_type: String,
    pub username: String,
    pub password_hash: String,
    pub email: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}


#[derive(Insertable)]
#[diesel(table_name = users)]  // Ensure this points to the correct table in your schema
pub struct NewUsers {
    pub user_id: String,
    pub user_type: String,
    pub username: String,
    pub password_hash: String,
    pub email: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
