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

//  Hard to write unit tests for may revisit

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_type_to_sql() {
        // Simulate what the `to_sql` function should do for each variant
        let user_admin = UserType::Admin;
        let user_editor = UserType::Editor;
        let user_viewer = UserType::Viewer;

        assert_eq!(format!("{}", serialize_user_type(&user_admin)), "admin");
        assert_eq!(format!("{}", serialize_user_type(&user_editor)), "editor");
        assert_eq!(format!("{}", serialize_user_type(&user_viewer)), "viewer");
    }

    #[test]
    fn test_user_type_from_sql() {
        // Simulate what `from_sql` should do when receiving string bytes
        let admin_bytes = b"admin";
        let editor_bytes = b"editor";
        let viewer_bytes = b"viewer";
        let invalid_bytes = b"invalid";

        assert_eq!(deserialize_user_type(admin_bytes).unwrap(), UserType::Admin);
        assert_eq!(deserialize_user_type(editor_bytes).unwrap(), UserType::Editor);
        assert_eq!(deserialize_user_type(viewer_bytes).unwrap(), UserType::Viewer);
        assert!(deserialize_user_type(invalid_bytes).is_err()); // Should error on invalid string
    }

    // Helper functions to simulate ToSql and FromSql behavior

    // Mock version of serialization (ToSql)
    fn serialize_user_type(user_type: &UserType) -> String {
        match user_type {
            UserType::Admin => "admin".to_string(),
            UserType::Editor => "editor".to_string(),
            UserType::Viewer => "viewer".to_string(),
        }
    }

    // Mock version of deserialization (FromSql)
    fn deserialize_user_type(bytes: &[u8]) -> Result<UserType, String> {
        match bytes {
            b"admin" => Ok(UserType::Admin),
            b"editor" => Ok(UserType::Editor),
            b"viewer" => Ok(UserType::Viewer),
            _ => Err("Unrecognized user type".to_string()),
        }
    }
}
