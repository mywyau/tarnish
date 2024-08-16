// src/schema.rs

use diesel::table;

table! {
    posts (id) {
        id -> Int4,
        post_id -> Varchar,
        title -> Varchar,
        body -> Text,
    }
}

table! {
    worklog (id) {
        id -> Int4,
        task_id -> Varchar,
        title -> Varchar,
        body -> Text,
    }
}

// table! {
//     test_posts (id) {
//         id -> Int4,
//         post_id -> Varchar,
//         title -> Varchar,
//         body -> Text,
//     }
// }