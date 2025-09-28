//! # Configuration Management for StoreHaus
//!
//! This crate provides centralized configuration structures for all StoreHaus components,
//! including database, cache, and signal system settings.
//!
//! ## Quick Start
//!
//! ### Programmatic Configuration
//! ```rust
//! use config::{DatabaseConfig, CacheConfig, SignalConfig};
//!
//! // Database configuration
//! let db_config = DatabaseConfig::new(
//!     "localhost".to_string(), 5432, "myapp".to_string(),
//!     "postgres".to_string(), "password".to_string(),
//!     1, 10, 30, 600, 3600,
//! );
//!
//! // Cache configuration
//! let cache_config = CacheConfig::new(
//!     "redis://localhost:6379".to_string(),
//!     10, 5000, 100, 3000,
//! );
//!
//! // Signal configuration
//! let signal_config = SignalConfig::new(
//!     30, 100, true, 3, 60, true, 300,
//! );
//! ```
//!
//! ### TOML File Configuration
//! ```toml
//! [database]
//! host = "localhost"
//! port = 5432
//! database = "myapp"
//! username = "postgres"
//! password = "password"
//! min_connections = 1
//! max_connections = 10
//! connection_timeout_seconds = 30
//! idle_timeout_seconds = 600
//! max_lifetime_seconds = 3600
//!
//! [cache]
//! redis_url = "redis://localhost:6379"
//! pool_size = 10
//! timeout_ms = 5000
//! max_connections = 100
//! connection_timeout_ms = 3000
//!
//! [signal]
//! callback_timeout_seconds = 30
//! max_callbacks = 100
//! remove_failing_callbacks = true
//! max_consecutive_failures = 3
//! cleanup_interval_seconds = 60
//! auto_remove_inactive_callbacks = true
//! inactive_callback_threshold_seconds = 300
//! ```
//!
//! Load configuration:
//! ```rust
//! use config::AppConfig;
//!
//! // Load from storehaus.toml
//! let config = AppConfig::load()?;
//!
//! // Or load from custom path
//! let config = AppConfig::from_file("config/production.toml")?;
//! ```

use serde::{Deserialize, Serialize};
use std::{env, path::Path};
use thiserror::Error;

const DEFAULT_CONFIG_PATH: &str = "./storehaus.toml";

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("TOML parsing error: {0}")]
    Toml(#[from] toml::de::Error),
    #[error("Environment variable error: {0}")]
    Env(#[from] env::VarError),
    #[error("Dotenvy error: {0}")]
    Dotenvy(#[from] dotenvy::Error),
    #[error("Invalid configuration: {0}")]
    Invalid(String),
}

/// Complete application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub database: DatabaseConfig,
    pub cache: CacheConfig,
    pub signal: SignalConfig,
}

/// Database configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub database: String,
    pub username: String,
    pub password: String,
    pub min_connections: u32,
    pub max_connections: u32,
    pub connection_timeout_seconds: u64,
    pub idle_timeout_seconds: u64,
    pub max_lifetime_seconds: u64,
}

/// Cache configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    pub redis_url: String,
    pub pool_size: u32,
    pub timeout_ms: u64,
    pub max_connections: u32,
    pub connection_timeout_ms: u64,
}

/// Signal system configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalConfig {
    pub callback_timeout_seconds: u64,
    pub max_callbacks: usize,
    pub remove_failing_callbacks: bool,
    pub max_consecutive_failures: u32,
    pub cleanup_interval_seconds: u64,
    pub auto_remove_inactive_callbacks: bool,
    pub inactive_callback_threshold_seconds: u64,
}

impl AppConfig {
    /// Load configuration from TOML file specified in .env or defaults
    pub fn load() -> Result<Self, ConfigError> {
        let config = {
            dotenvy::dotenv()?;

            // Try to load .env file for STOREHAUS_CONFIG path
            if let Ok(config_path) = env::var("STOREHAUS_CONFIG") {
                Self::from_file(&config_path)
            }
            // Try to load config from DEFAULT_CONFIG_PATH
            else if Path::new(DEFAULT_CONFIG_PATH).exists() {
                Self::from_file(DEFAULT_CONFIG_PATH)
            }
            // Return error if neither .env file nor default config file exists
            else {
                Err(ConfigError::Invalid(format!(
                    "Config path must be specified in .env file as STOREHAUS_CONFIG or in {} file",
                    DEFAULT_CONFIG_PATH
                )))
            }
        }?;

        config.validate()?;
        Ok(config)
    }

    /// Load configuration from TOML file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        let content = std::fs::read_to_string(path)?;
        let config: Self = toml::from_str(&content)?;
        config.validate()?;
        Ok(config)
    }

    /// Validate configuration values
    fn validate(&self) -> Result<(), ConfigError> {
        // Database validations
        if self.database.host.is_empty() {
            return Err(ConfigError::Invalid(
                "Database host cannot be empty".to_string(),
            ));
        }
        if self.database.port == 0 {
            return Err(ConfigError::Invalid(
                "Database port cannot be zero".to_string(),
            ));
        }
        if self.database.database.is_empty() {
            return Err(ConfigError::Invalid(
                "Database name cannot be empty".to_string(),
            ));
        }
        if self.database.username.is_empty() {
            return Err(ConfigError::Invalid(
                "Database username cannot be empty".to_string(),
            ));
        }
        if self.database.min_connections == 0 {
            return Err(ConfigError::Invalid(
                "Database min_connections must be greater than 0".to_string(),
            ));
        }
        if self.database.max_connections == 0 {
            return Err(ConfigError::Invalid(
                "Database max_connections must be greater than 0".to_string(),
            ));
        }
        if self.database.min_connections > self.database.max_connections {
            return Err(ConfigError::Invalid(
                "Database min_connections cannot be greater than max_connections".to_string(),
            ));
        }
        if self.database.connection_timeout_seconds == 0 {
            return Err(ConfigError::Invalid(
                "Database connection_timeout_seconds must be greater than 0".to_string(),
            ));
        }

        // Cache validations
        if self.cache.redis_url.is_empty() {
            return Err(ConfigError::Invalid(
                "Redis URL cannot be empty".to_string(),
            ));
        }
        if self.cache.pool_size == 0 {
            return Err(ConfigError::Invalid(
                "Cache pool_size must be greater than 0".to_string(),
            ));
        }
        if self.cache.max_connections == 0 {
            return Err(ConfigError::Invalid(
                "Cache max_connections must be greater than 0".to_string(),
            ));
        }
        if self.cache.timeout_ms == 0 {
            return Err(ConfigError::Invalid(
                "Cache timeout_ms must be greater than 0".to_string(),
            ));
        }
        if self.cache.connection_timeout_ms == 0 {
            return Err(ConfigError::Invalid(
                "Cache connection_timeout_ms must be greater than 0".to_string(),
            ));
        }

        // Signal validations
        if self.signal.max_consecutive_failures == 0 {
            return Err(ConfigError::Invalid(
                "Signal max_consecutive_failures must be greater than 0".to_string(),
            ));
        }
        if self.signal.cleanup_interval_seconds == 0 {
            return Err(ConfigError::Invalid(
                "Signal cleanup_interval_seconds must be greater than 0".to_string(),
            ));
        }

        Ok(())
    }
}

impl CacheConfig {
    /// Create a new cache configuration
    pub fn new(
        redis_url: String,
        pool_size: u32,
        timeout_ms: u64,
        max_connections: u32,
        connection_timeout_ms: u64,
    ) -> Self {
        Self {
            redis_url,
            pool_size,
            timeout_ms,
            max_connections,
            connection_timeout_ms,
        }
    }
}

impl SignalConfig {
    /// Create a new signal configuration
    pub fn new(
        callback_timeout_seconds: u64,
        max_callbacks: usize,
        remove_failing_callbacks: bool,
        max_consecutive_failures: u32,
        cleanup_interval_seconds: u64,
        auto_remove_inactive_callbacks: bool,
        inactive_callback_threshold_seconds: u64,
    ) -> Self {
        Self {
            callback_timeout_seconds,
            max_callbacks,
            remove_failing_callbacks,
            max_consecutive_failures,
            cleanup_interval_seconds,
            auto_remove_inactive_callbacks,
            inactive_callback_threshold_seconds,
        }
    }
}

impl DatabaseConfig {
    /// Create a new database configuration
    pub fn new(
        host: String,
        port: u16,
        database: String,
        username: String,
        password: String,
        min_connections: u32,
        max_connections: u32,
        connection_timeout_seconds: u64,
        idle_timeout_seconds: u64,
        max_lifetime_seconds: u64,
    ) -> Self {
        Self {
            host,
            port,
            database,
            username,
            password,
            min_connections,
            max_connections,
            connection_timeout_seconds,
            idle_timeout_seconds,
            max_lifetime_seconds,
        }
    }

    /// Build connection string
    pub fn connection_string(&self) -> String {
        format!(
            "postgresql://{}:{}@{}:{}/{}",
            self.username, self.password, self.host, self.port, self.database
        )
    }
}
