// src/blog_schema

use diesel::table;

table! {
    users (id) {
        id -> Int4,
        role_id -> Varchar,
        user_type -> String,
        username -> Varchar,
        password_hash -> Varchar,
        email -> Varchar,
        created_at -> Timestamp,
        updated_at -> Timestamp
    }
}