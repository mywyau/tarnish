use chrono::NaiveDateTime;
use diesel::{Insertable, Queryable};
use serde::{Deserialize, Serialize};

use diesel::backend::Backend;
use diesel::deserialize::{self, FromSql};
use diesel::serialize::{self, IsNull, Output, ToSql};
use diesel::sql_types::Text;
use std::io::Write;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub enum UserType {
    Admin,
    Editor,
    Viewer,
}

use diesel::pg::Pg;

// Serializers and Deserializers for PostgreSQL

// Convert the enum to a string when inserting into the database
impl ToSql<Text, Pg> for UserType {
    fn to_sql<W: Write>(&self, out: &mut Output<W, Pg>) -> serialize::Result {
        let role_str = match *self {
            UserType::Admin => "admin",
            UserType::Editor => "editor",
            UserType::Viewer => "viewer",
        };
        out.write_all(role_str.as_bytes())?;
        Ok(IsNull::No)
    }
}

// Convert the string from the database back to the enum
impl FromSql<Text, Pg> for UserType {
    fn from_sql(bytes: Option<&[u8]>) -> deserialize::Result<Self> {
        match not_none!(bytes) {
            b"admin" => Ok(UserType::Admin),
            b"editor" => Ok(UserType::Editor),
            b"viewer" => Ok(UserType::Viewer),
            _ => Err("Unrecognized user type".into()),
        }
    }
}
