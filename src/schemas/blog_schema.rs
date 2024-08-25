// src/blog_schema

use diesel::table;

table! {
    posts (id) {
        id -> Int4,
        post_id -> Varchar,
        title -> Varchar,
        body -> Text
    }
}