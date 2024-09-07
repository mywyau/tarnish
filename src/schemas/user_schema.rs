// src/blog_schema

use diesel::table;

table! {
    users (id) {
        id -> Int4,
        user_id -> Varchar,
        user_type -> Varchar,
        username -> Varchar,
        password_hash -> Varchar,
        email -> Varchar,
        created_at -> Timestamp,
        updated_at -> Timestamp
    }
}