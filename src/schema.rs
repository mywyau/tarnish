// @generated automatically by Diesel CLI.

diesel::table! {
    posts (id) {
        id -> Int4,
        post_id -> Varchar,
        title -> Varchar,
        body -> Text,
        created_at -> Nullable<Timestamptz>,
        updated_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    skills (id) {
        id -> Int4,
        skill_id -> Varchar,
        skill_name -> Varchar,
        body -> Text,
        created_at -> Nullable<Timestamptz>,
        updated_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    worklog (id) {
        id -> Int4,
        worklog_id -> Varchar,
        worklog_title -> Varchar,
        body -> Text,
        created_at -> Nullable<Timestamptz>,
        updated_at -> Nullable<Timestamptz>,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    posts,
    skills,
    worklog,
);
