// src/blog_schema

use diesel::table;

table! {
    skills (id) {
        id -> Int4,
        skill_id -> Varchar,
        skill_name -> Varchar,
        body -> Text,
        created_at -> Timestamp,
        updated_at -> Timestamp
    }
}