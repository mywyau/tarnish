use serde::{Deserialize, Serialize};

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

use diesel::pg::{Pg, PgValue};
// Serializers and Deserializers for PostgreSQL

// Convert the enum to a string when inserting into the database
// Implementing ToSql for PostgreSQL
impl ToSql<Text, Pg> for UserType {
    fn to_sql(&self, out: &mut Output<Pg>) -> serialize::Result {
        let role_str =
            match *self {
                UserType::Admin => "admin",
                UserType::Editor => "editor",
                UserType::Viewer => "viewer",
            };
        out.write_all(role_str.as_bytes())?;  // Write to the SQL output
        Ok(IsNull::No)
    }
}

// Convert the string from the database back to the enum
// Implementing FromSql for PostgreSQL
impl FromSql<Text, Pg> for UserType {
    fn from_sql(value: PgValue<'_>) -> deserialize::Result<Self> {
        let bytes = value.as_bytes(); // Retrieve the byte slice from PgValue
        match bytes {
            b"admin" => Ok(UserType::Admin),
            b"editor" => Ok(UserType::Editor),
            b"viewer" => Ok(UserType::Viewer),
            _ => Err("Unrecognized user type".into()),
        }
    }
}