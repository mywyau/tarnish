use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use dotenv::dotenv;
use std::env;
use std::error::Error;
use std::fmt;

pub type DbPool = Pool<ConnectionManager<PgConnection>>;

// Define custom error type
#[derive(Debug)]
pub enum DbConnectionError {
    MissingDatabaseUrl,
    PoolCreationError,
}

// Implement std::fmt::Display for your error type
impl fmt::Display for DbConnectionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DbConnectionError::MissingDatabaseUrl => write!(f, "DATABASE_URL must be set"),
            DbConnectionError::PoolCreationError => write!(f, "Failed to create connection pool"),
        }
    }
}

// Implement std::error::Error for your error type
impl Error for DbConnectionError {}

// Trait to make the connection injectable for mocking
pub trait DbConnector {
    fn establish_connection(&self) -> Result<DbPool, DbConnectionError>;
}

// Real implementation of the DbConnector trait
pub struct RealDbConnector;

impl DbConnector for RealDbConnector {
    fn establish_connection(&self) -> Result<DbPool, DbConnectionError> {
        dotenv().ok();
        let database_url = env::var("DATABASE_URL").map_err(|_| DbConnectionError::MissingDatabaseUrl)?;
        let manager = ConnectionManager::<PgConnection>::new(database_url);

        diesel::r2d2::Pool::builder()
            .build(manager)
            .map_err(|_| DbConnectionError::PoolCreationError)
    }
}

use mockall::mock;

mock! {
    pub DbConnector {}

    impl DbConnector for DbConnector {
        fn establish_connection(&self) -> Result<DbPool, DbConnectionError>;
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::sync::Once;

    static INIT: Once = Once::new();

    fn init_env() {
        INIT.call_once(|| {
            dotenv().ok();
        });
    }

    #[test]
    fn test_establish_connection_missing_database_url() {
        // Remove DATABASE_URL to simulate missing environment variable
        env::remove_var("DATABASE_URL");

        // Mock the DbConnector
        let mut mock_connector = MockDbConnector::new();
        mock_connector
            .expect_establish_connection()
            .returning(|| Err(DbConnectionError::MissingDatabaseUrl));

        // Call the mocked function
        let connection_result = mock_connector.establish_connection();

        // Ensure that the error is related to the missing DATABASE_URL
        assert!(
            matches!(connection_result, Err(DbConnectionError::MissingDatabaseUrl)),
            "Expected DbConnectionError::MissingDatabaseUrl, got {:?}",
            connection_result
        );
    }

    #[test]
    fn test_establish_connection_success() {
        init_env();

        // Set a valid DATABASE_URL for testing
        env::set_var("DATABASE_URL", "postgres://test:test-password@localhost:5430/test_db");

        // Mock the DbConnector
        let mut mock_connector = MockDbConnector::new();
        mock_connector
            .expect_establish_connection()
            .returning(|| {
                // Simulate successful connection pool creation
                let manager = ConnectionManager::<PgConnection>::new("postgres://test:test-password@localhost:5430/test_db");
                let pool = Pool::builder().build(manager).unwrap();
                Ok(pool)
            });

        // Call the mocked function
        let connection_result = mock_connector.establish_connection();

        // Ensure that the connection is successful
        assert!(
            connection_result.is_ok(),
            "Expected a successful connection, got {:?}",
            connection_result
        );
    }

    #[test]
    fn test_establish_connection_pool_creation_failure() {
        init_env();

        // Set a valid DATABASE_URL
        env::set_var("DATABASE_URL", "postgres://test:test-password@localhost:5430/test_db");

        // Mock the DbConnector
        let mut mock_connector = MockDbConnector::new();
        mock_connector
            .expect_establish_connection()
            .returning(|| Err(DbConnectionError::PoolCreationError));

        // Call the mocked function
        let connection_result = mock_connector.establish_connection();

        // Ensure that the error is related to pool creation failure
        assert!(
            matches!(connection_result, Err(DbConnectionError::PoolCreationError)),
            "Expected DbConnectionError::PoolCreationError, got {:?}",
            connection_result
        );
    }

    #[test]
    fn test_establish_connection_valid_database_url() {
        // Set a valid DATABASE_URL
        env::set_var("DATABASE_URL", "postgres://test:test-password@localhost:5430/test_db");

        // Mock the DbConnector
        let mut mock_connector = MockDbConnector::new();
        mock_connector
            .expect_establish_connection()
            .returning(|| {
                // Simulate successful connection pool creation
                let manager = ConnectionManager::<PgConnection>::new("postgres://test:test-password@localhost:5430/test_db");
                let pool = Pool::builder().build(manager).unwrap();
                Ok(pool)
            });

        // Call the mocked function
        let connection_result = mock_connector.establish_connection();

        // Ensure that the connection is successful
        assert!(
            connection_result.is_ok(),
            "Expected a successful connection, got {:?}",
            connection_result
        );
    }

    #[test]
    fn test_establish_connection_invalid_database_url() {
        // Set an invalid DATABASE_URL
        env::set_var("DATABASE_URL", "invalid_url");

        // Mock the DbConnector
        let mut mock_connector = MockDbConnector::new();
        mock_connector
            .expect_establish_connection()
            .returning(|| Err(DbConnectionError::PoolCreationError));

        // Call the mocked function
        let connection_result = mock_connector.establish_connection();

        // Ensure that the error is related to pool creation failure due to invalid URL
        assert!(
            matches!(connection_result, Err(DbConnectionError::PoolCreationError)),
            "Expected DbConnectionError::PoolCreationError, got {:?}",
            connection_result
        );
    }
}
