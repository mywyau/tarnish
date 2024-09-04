#[cfg(test)]
mod worklog_controller_spec {
    use std::env;
    use tarnish::connectors::postgres_connector::DbPool;
    use tarnish::controllers::worklog_controller::{create_worklog, delete_all_worklog, delete_worklog, get_all_worklog, get_by_worklog_id, update_worklog};
    use tarnish::schemas::worklog_schema::worklog;
    use tarnish::{NewWorklog, Worklog};

    use actix_web::{body::to_bytes, http::StatusCode, test, web, App};
    use bytes::Bytes;
    use diesel::prelude::*;
    use diesel::r2d2::{ConnectionManager, PooledConnection};
    use diesel::{r2d2, PgConnection};
    use dotenv::dotenv;
    use serde_json::{json, Value};

    #[ctor::ctor]
    fn init() {
        let pool = establish_connection();
        let mut conn: PooledConnection<ConnectionManager<PgConnection>> =
            pool.get().expect("Failed to get connection from pool");

        diesel::sql_query("TRUNCATE TABLE worklog RESTART IDENTITY CASCADE;")
            .execute(&mut conn)
            .expect("Failed to reset ID sequence");
    }

    struct TestGuard {
        pool: web::Data<DbPool>,
        worklog_ids: Vec<String>,
    }

    impl TestGuard {
        fn new(pool: web::Data<DbPool>, worklog_to_insert: Vec<NewWorklog>) -> Self {
            let mut conn: PooledConnection<ConnectionManager<PgConnection>> =
                pool.get().expect("Failed to get connection from pool");

            for worklog in &worklog_to_insert {
                diesel::insert_into(worklog::table)
                    .values(worklog)
                    .execute(&mut conn)
                    .expect("Failed to insert test worklog");
            }

            let worklog_ids =
                worklog_to_insert.into_iter().map(|p| p.worklog_id).collect();

            TestGuard { pool, worklog_ids }
        }
    }

    impl Drop for TestGuard {
        fn drop(&mut self) {
            let mut conn: PooledConnection<ConnectionManager<PgConnection>> =
                self.pool.get().expect("Failed to get connection from pool");

            for worklog_id in &self.worklog_ids {
                diesel::delete(worklog::table.filter(worklog::worklog_id.eq(worklog_id)))
                    .execute(&mut conn)
                    .expect("Failed to delete test worklog");
            }
        }
    }

    pub fn establish_connection() -> DbPool {
        dotenv().ok();
        let database_url = env::var("TEST_DATABASE_URL").expect("TEST_DATABASE_URL must be set");
        let manager = ConnectionManager::<PgConnection>::new(database_url);
        r2d2::Pool::builder().build(manager).expect("Failed to create pool.")
    }

    #[actix_rt::test]
    async fn run_all_tests_in_order_worklog() {
        test_get_by_worklog_id().await;
        test_get_all_worklog().await;
        test_create_worklog().await;
        test_update_worklog().await;
        test_delete_worklog().await;
        test_delete_all_worklog().await;
    }

    async fn test_get_by_worklog_id() {
        let pool = web::Data::new(establish_connection());

        let app =
            test::init_service(
                App::new()
                    .app_data(pool.clone())
                    .service(get_by_worklog_id)
                    .service(create_worklog)
                    .service(update_worklog)
                    .service(delete_worklog),
            )
                .await;

        let worklog_to_insert = vec![
            NewWorklog {
                worklog_id: "worklog4".to_string(),
                work_title: "Cats Work".to_string(),
                body: "This is the first worklog.".to_string(),
                created_at: chrono::Utc::now().naive_utc(), // Current time in ISO 8601 format
                updated_at: chrono::Utc::now().naive_utc(), // Current time in ISO 8601 format
            },
            NewWorklog {
                worklog_id: "worklog5".to_string(),
                work_title: "Latex Work".to_string(),
                body: "This is the second worklog.".to_string(),
                created_at: chrono::Utc::now().naive_utc(), // Current time in ISO 8601 format
                updated_at: chrono::Utc::now().naive_utc(), // Current time in ISO 8601 format
            },
        ];

        let _guard = TestGuard::new(pool.clone(), worklog_to_insert);

        let req = test::TestRequest::get()
            .uri("/blog/worklog/retrieve/worklog-id/worklog5")
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let body: Bytes = to_bytes(resp.into_body()).await.unwrap();
        let body_str = std::str::from_utf8(&body).unwrap();
        let json_body: Value = serde_json::from_str(body_str).unwrap();

        let worklog_id_field = json_body.get("worklog_id").unwrap().as_str().unwrap();
        let work_title_field = json_body.get("work_title").unwrap().as_str().unwrap();
        let body_field = json_body.get("body").unwrap().as_str().unwrap();

        assert_eq!(worklog_id_field, "worklog5");
        assert_eq!(work_title_field, "Latex Work");
        assert_eq!(body_field, "This is the second worklog.");
    }

    async fn test_get_all_worklog() {
        let pool = web::Data::new(establish_connection());

        let app = test::init_service(
            App::new()
                .app_data(pool.clone())
                .service(get_all_worklog),
        )
            .await;

        let worklog_to_insert = vec![
            NewWorklog {
                worklog_id: "worklog10".to_string(),
                work_title: "Raking Leaves".to_string(),
                body: "Some content 1".to_string(),
                created_at: chrono::Utc::now().naive_utc(), // Current time in ISO 8601 format
                updated_at: chrono::Utc::now().naive_utc(), // Current time in ISO 8601 format
            },
            NewWorklog {
                worklog_id: "worklog11".to_string(),
                work_title: "Cutting Potatoes".to_string(),
                body: "Some content 2".to_string(),
                created_at: chrono::Utc::now().naive_utc(), // Current time in ISO 8601 format
                updated_at: chrono::Utc::now().naive_utc(), // Current time in ISO 8601 format
            },
            NewWorklog {
                worklog_id: "worklog12".to_string(),
                work_title: "Farming Pigeons".to_string(),
                body: "Some content 3".to_string(),
                created_at: chrono::Utc::now().naive_utc(), // Current time in ISO 8601 format
                updated_at: chrono::Utc::now().naive_utc(), // Current time in ISO 8601 format
            },
        ];

        let _guard = TestGuard::new(pool.clone(), worklog_to_insert);

        let req = test::TestRequest::get()
            .uri("/blog/worklog/get/all")  // Ensure this matches your actual route
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let body: Bytes = to_bytes(resp.into_body()).await.unwrap();
        let json_body: Value = serde_json::from_slice(&body).unwrap();

        // Assuming the response is an array of worklog
        let worklog_json_array = json_body.as_array().expect("Expected an array of worklog");

        // Assert the length of the array
        assert_eq!(worklog_json_array.len(), 3);

        // Assert the content of each worklog
        assert_eq!(worklog_json_array[0]["worklog_id"], "worklog10");
        assert_eq!(worklog_json_array[0]["work_title"], "Raking Leaves");
        assert_eq!(worklog_json_array[0]["body"], "Some content 1");

        assert_eq!(worklog_json_array[1]["worklog_id"], "worklog11");
        assert_eq!(worklog_json_array[1]["work_title"], "Cutting Potatoes");
        assert_eq!(worklog_json_array[1]["body"], "Some content 2");

        assert_eq!(worklog_json_array[2]["worklog_id"], "worklog12");
        assert_eq!(worklog_json_array[2]["work_title"], "Farming Pigeons");
        assert_eq!(worklog_json_array[2]["body"], "Some content 3");
    }

    async fn test_create_worklog() {
        let pool = web::Data::new(establish_connection());

        let app =
            test::init_service(
                App::new()
                    .app_data(pool.clone())
                    .service(get_by_worklog_id)
                    .service(create_worklog)
                    .service(update_worklog)
                    .service(delete_worklog),
            )
                .await;

        let worklog_to_insert = vec![
            NewWorklog {
                worklog_id: "worklog1".to_string(),
                work_title: "Python".to_string(),
                body: "Some content about the worklog".to_string(),
                created_at: chrono::Utc::now().naive_utc(), // Current time in ISO 8601 format
                updated_at: chrono::Utc::now().naive_utc(), // Current time in ISO 8601 format
            },
            NewWorklog {
                worklog_id: "worklog2".to_string(),
                work_title: "Typescript".to_string(),
                body: "Some content about the worklog 2".to_string(),
                created_at: chrono::Utc::now().naive_utc(), // Current time in ISO 8601 format
                updated_at: chrono::Utc::now().naive_utc(), // Current time in ISO 8601 format
            },
        ];

        let _guard = TestGuard::new(pool.clone(), worklog_to_insert);

        let payload = json!({
            "id": 200,
            "worklog_id": "worklog3",
            "work_title": "Rust",
            "body": "Some content about Rust",
            "created_at": "2023-08-29T14:00:00Z", // Example timestamp
            "updated_at": "2023-08-29T14:00:01Z"  // Example timestamp
        });

        let create_req =
            test::TestRequest::post()
                .uri("/blog/worklog/create")
                .set_json(&payload)
                .to_request();

        let create_resp = test::call_service(&app, create_req).await;
        assert_eq!(create_resp.status(), StatusCode::CREATED);

        let body = test::read_body(create_resp).await;
        let body_str = std::str::from_utf8(&body).unwrap();
        let json_body: Value = serde_json::from_str(body_str).unwrap();

        let worklog_id = json_body.get("worklog_id").unwrap().as_str().unwrap();
        assert_eq!(worklog_id, "worklog3");
    }

    async fn test_update_worklog() {
        let pool = web::Data::new(establish_connection());

        let app =
            test::init_service(
                App::new()
                    .app_data(pool.clone())
                    .service(get_by_worklog_id)
                    .service(create_worklog)
                    .service(update_worklog)
                    .service(delete_worklog),
            )
                .await;

        let worklog_to_insert = vec![
            NewWorklog {
                worklog_id: "worklog_25".to_string(),
                work_title: "Eating Watermelon 3".to_string(),
                body: "Fake content".to_string(),
                created_at: chrono::Utc::now().naive_utc(), // Current time in ISO 8601 format
                updated_at: chrono::Utc::now().naive_utc(), // Current time in ISO 8601 format
            },
        ];

        let _guard = TestGuard::new(pool.clone(), worklog_to_insert);

        let payload = json!({
            "id": 1,
            "worklog_id": "worklog_25",
            "work_title": "Eating Onions",
            "body": "Updated body content.",
            "created_at": "2023-08-29T14:00:00Z", // Example timestamp
            "updated_at": "2023-08-29T14:00:01Z"  // Example timestamp
        });

        let put_req = test::TestRequest::put()
            .uri("/blog/worklog/update/worklog_25")
            .set_json(&payload)
            .to_request();

        let put_resp = test::call_service(&app, put_req).await;
        assert!(put_resp.status().is_success());

        let body: Bytes = to_bytes(put_resp.into_body()).await.unwrap();
        let body_str = std::str::from_utf8(&body).unwrap();
        let json_body: Value = serde_json::from_str(body_str).unwrap();

        let expected_message = json!({
            "message": "Work 'Eating Watermelon 3' has been updated"
        });

        assert_eq!(json_body, expected_message);
    }

    async fn test_delete_worklog() {
        let pool = web::Data::new(establish_connection());

        let app =
            test::init_service(
                App::new()
                    .app_data(pool.clone())
                    .service(get_by_worklog_id)
                    .service(create_worklog)
                    .service(update_worklog)
                    .service(delete_worklog),
            )
                .await;

        let worklog_to_insert = vec![
            NewWorklog {
                worklog_id: "abc200".to_string(),
                work_title: "Fake Worklog".to_string(),
                body: "This is the first test worklog.".to_string(),
                created_at: chrono::Utc::now().naive_utc(), // Current time in ISO 8601 format
                updated_at: chrono::Utc::now().naive_utc(), // Current time in ISO 8601 format
            },
            NewWorklog {
                worklog_id: "def456".to_string(),
                work_title: "Fake Worklog 2".to_string(),
                body: "This is the second test worklog.".to_string(),
                created_at: chrono::Utc::now().naive_utc(), // Current time in ISO 8601 format
                updated_at: chrono::Utc::now().naive_utc(), // Current time in ISO 8601 format
            },
        ];

        let _guard = TestGuard::new(pool.clone(), worklog_to_insert);

        let delete_request = test::TestRequest::delete()
            .uri("/blog/worklog/single/abc200")
            .to_request();

        let delete_response = test::call_service(&app, delete_request).await;
        assert!(delete_response.status().is_success());

        let mut conn: PooledConnection<ConnectionManager<PgConnection>> =
            pool.get().expect("Failed to get connection from pool");

        let deleted_worklog_1 =
            worklog::table
                .filter(worklog::worklog_id.eq("abc200"))
                .first::<Worklog>(&mut conn)
                .optional()
                .expect("Failed to check for deleted worklog");

        assert!(deleted_worklog_1.is_none());

        let deleted_worklog_2 =
            worklog::table
                .filter(worklog::worklog_id.eq("def456"))
                .first::<Worklog>(&mut conn)
                .optional()
                .expect("Failed to check for deleted worklog");

        assert!(deleted_worklog_2.is_some());
    }

    async fn test_delete_all_worklog() {
        let pool = web::Data::new(establish_connection());

        let app =
            test::init_service(
                App::new()
                    .app_data(pool.clone())
                    .service(get_by_worklog_id)
                    .service(create_worklog)
                    .service(update_worklog)
                    .service(delete_all_worklog),
            )
                .await;

        let worklog_to_insert = vec![
            NewWorklog {
                worklog_id: "fake_id_1".to_string(),
                work_title: "Fake Worklog 1".to_string(),
                body: "This is the first worklog.".to_string(),
                created_at: chrono::Utc::now().naive_utc(), // Current time in ISO 8601 format
                updated_at: chrono::Utc::now().naive_utc(), // Current time in ISO 8601 format
            },
            NewWorklog {
                worklog_id: "fake_id_2".to_string(),
                work_title: "Fake Worklog 2".to_string(),
                body: "This is the second worklog.".to_string(),
                created_at: chrono::Utc::now().naive_utc(), // Current time in ISO 8601 format
                updated_at: chrono::Utc::now().naive_utc(), // Current time in ISO 8601 format
            },
        ];

        let _guard = TestGuard::new(pool.clone(), worklog_to_insert);

        let delete_request =
            test::TestRequest::delete()
                .uri("/blog/worklog/all")
                .to_request();

        let delete_response =
            test::call_service(&app, delete_request).await;

        assert!(delete_response.status().is_success());

        let mut conn: PooledConnection<ConnectionManager<PgConnection>> =
            pool.get().expect("Failed to get connection from pool");

        let deleted_worklog_1 = worklog::table
            .filter(worklog::worklog_id.eq("fake_id_1"))
            .first::<Worklog>(&mut conn)
            .optional()
            .expect("Failed to check for deleted worklog");

        assert!(deleted_worklog_1.is_none());

        let deleted_worklog_2 = worklog::table
            .filter(worklog::worklog_id.eq("fake_id_2"))
            .first::<Worklog>(&mut conn)
            .optional()
            .expect("Failed to check for deleted worklog");

        assert!(deleted_worklog_2.is_none());
    }
}
