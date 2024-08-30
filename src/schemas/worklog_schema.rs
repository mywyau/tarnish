use diesel::table;


table! {
    worklog (id) {
        id -> Int4,
        worklog_id -> Varchar,
        work_title -> Varchar,
        body -> Text,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}
