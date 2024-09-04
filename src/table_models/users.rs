use chrono::NaiveDateTime;
use diesel::{Insertable, Queryable};
use serde::{Deserialize, Serialize};

use crate::schemas::user_schema::users;

#[derive(Queryable, Serialize, Deserialize)]
#[diesel(table_name = users)]  // Ensure this points to the correct table in your schema
pub struct Users {
    pub id: i32,
    pub role_id: String,
    pub user_type: String,
    pub username: String,
    pub password_hash: String,
    pub email: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

// // Manually implement Queryable for Users (optional, only if needed)
// impl<ST> diesel::Queryable<ST, Pg> for Users
// where
//     (i32, String, UserType, String, String, String, NaiveDateTime, NaiveDateTime): FromSqlRow<ST, Pg>,
// {
//     type Row = (i32, String, UserType, String, String, String, NaiveDateTime, NaiveDateTime);
//
//     fn build(row: Self::Row) {
//         Users {
//             id: row.0,
//             role_id: row.1,
//             user_type: row.2,
//             username: row.3,
//             password_hash: row.4,
//             email: row.5,
//             created_at: row.6,
//             updated_at: row.7,
//         }
//     }
// }

#[derive(Insertable)]
#[diesel(table_name = users)]  // Ensure this points to the correct table in your schema
pub struct NewUsers {
    pub role_id: String,
    pub user_type: String,
    pub username: String,
    pub password_hash: String,
    pub email: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

// Optionally, Roles struct (uncomment if needed)
// #[derive(Insertable, Queryable, Serialize, Deserialize)]
// pub struct Roles {
//     pub id: i32,
//     pub user_type: UserType
// }
