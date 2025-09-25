pub mod errors;
pub mod config;
pub mod core;
pub mod migration;

// Re-export the main public types for convenience
pub use errors::DispatcherError;
pub use config::DatabaseConfig;
pub use core::Dispatcher;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_dispatcher_creation() {
        let config = DatabaseConfig::default();

        // This test would require a running PostgreSQL instance
        // In a real test environment, you might want to use a test database
        // or mock the database connection

        // For now, just test the configuration creation
        assert_eq!(config.host, "localhost");
        assert_eq!(config.port, 5432);
        assert_eq!(config.database, "storehaus");
    }

    #[test]
    fn test_database_config() {
        let config = DatabaseConfig::new(
            "localhost".to_string(),
            5432,
            "test_db".to_string(),
            "user".to_string(),
            "pass".to_string(),
        ).with_max_connections(20);

        assert_eq!(config.max_connections, 20);
        assert_eq!(
            config.connection_string(),
            "postgresql://user:pass@localhost:5432/test_db"
        );
    }
}