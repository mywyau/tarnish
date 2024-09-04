use diesel::r2d2;
use std::env;
use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;
use dotenv::dotenv;

pub type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;

pub fn establish_connection() -> Result<DbPool, String> {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").map_err(|_| "DATABASE_URL must be set".to_string())?;

    let manager = ConnectionManager::<PgConnection>::new(database_url);

    r2d2::Pool::builder()
        .build(manager)
        .map_err(|_| "Failed to create pool.".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Once;
    use std::env;

    // This ensures the `.env` file is loaded only once
    static INIT: Once = Once::new();

    // Helper function to initialize the environment
    fn init_env() {
        INIT.call_once(|| {
            dotenv().ok();
        });
    }

    #[test]
    fn test_establish_connection_success() {
        // Initialize the environment (once)
        init_env();

        // Ensure the correct DATABASE_URL is set for testing
        env::set_var("DATABASE_URL", "postgres://test:test-password@localhost:5430/test_db");

        // Call the function to test
        let connection_result = establish_connection();

        // Check that the connection pool is created successfully
        assert!(connection_result.is_ok(), "Connection should be established");
    }

    #[test]
    fn test_establish_connection_missing_database_url() {
        // Ensure that DATABASE_URL is removed before loading .env
        env::remove_var("DATABASE_URL");

        // Now load the environment variables (this ensures DATABASE_URL is not set)
        dotenv().ok();

        // Call the function to test and expect it to return an error
        let connection_result = establish_connection();

        // Check that the result is an error, as the DATABASE_URL is missing
        assert!(connection_result.is_err(), "Connection should fail without DATABASE_URL");
        assert_eq!(connection_result.unwrap_err(), "DATABASE_URL must be set");
    }

    #[test]
    fn test_establish_connection_invalid_database_url() {
        // Set an invalid DATABASE_URL before loading .env
        env::set_var("DATABASE_URL", "invalid_url");

        // Now load the environment variables
        dotenv().ok();

        // Call the function to test
        let connection_result = establish_connection();

        // This should return an error because the database URL is invalid
        assert!(connection_result.is_err(), "Connection should fail with an invalid URL");
        assert_eq!(connection_result.unwrap_err(), "Failed to create pool.");
    }

    #[test]
    fn test_reset_database_url_after_tests() {
        // Ensure tests don't affect each other's environment variables
        init_env();

        // Set a valid database URL for this test
        env::set_var("DATABASE_URL", "postgres://test:test-password@localhost:5430/test_db");

        let connection_result = establish_connection();

        // Check that the connection pool is created successfully
        assert!(connection_result.is_ok(), "Connection should be established");

        // Reset DATABASE_URL to its previous state after the test (clean up)
        env::remove_var("DATABASE_URL");
    }
}
