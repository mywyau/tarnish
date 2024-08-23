// src/blog_schema

use diesel::table;

table! {
    worklog (id) {
        id -> Int4,
        worklog_id -> Varchar,
        work_title -> Varchar,
        body -> Text,
        time_created -> Text,
        time_updated -> Text,
    }
}