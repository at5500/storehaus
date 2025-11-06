//! # StoreHaus
//!
//! A modern Rust database abstraction library for PostgreSQL with automatic code generation,
//! signals, caching, and advanced query capabilities.
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use storehaus::prelude::*;
//!
//! #[model]
//! #[table(name = "users")]
//! pub struct User {
//!     #[primary_key]
//!     pub id: Uuid,
//!
//!     #[field(create, update)]
//!     pub name: String,
//!
//!     #[field(create, update)]
//!     pub email: String,
//! }
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let config = DatabaseConfig::new(
//!         "localhost".to_string(), 5432, "storehaus".to_string(),
//!         "postgres".to_string(), "password".to_string(),
//!         1, 5, 30, 600, 3600,
//!     );
//!
//!     let mut storehaus = StoreHaus::new(config).await?;
//!     storehaus.auto_migrate::<User>(true).await?;
//!
//!     let user_store = GenericStore::<User>::new(
//!         storehaus.pool().clone(),
//!         None, // no signals
//!         None, // no cache
//!     );
//!
//!     storehaus.register_store("users".to_string(), user_store)?;
//!     let user_store = storehaus.get_store::<GenericStore<User>>("users")?;
//!
//!     let user = User::new(
//!         Uuid::new_v4(),
//!         "John Doe".to_string(),
//!         "john@example.com".to_string(),
//!     );
//!
//!     let created = user_store.create(user, None).await?;
//!     println!("Created user: {}", created.name);
//!
//!     Ok(())
//! }
//! ```

/// Conditional debug logging macros
/// These macros only compile in code when the `debug-logging` feature is enabled
#[cfg(feature = "debug-logging")]
#[macro_export]
macro_rules! debug_log {
    ($($arg:tt)*) => {
        tracing::debug!($($arg)*)
    };
}

#[cfg(not(feature = "debug-logging"))]
#[macro_export]
macro_rules! debug_log {
    ($($arg:tt)*) => {};
}

#[cfg(feature = "debug-logging")]
#[macro_export]
macro_rules! trace_log {
    ($($arg:tt)*) => {
        tracing::trace!($($arg)*)
    };
}

#[cfg(not(feature = "debug-logging"))]
#[macro_export]
macro_rules! trace_log {
    ($($arg:tt)*) => {};
}

pub mod core;
pub mod errors;
pub mod migration;
pub mod prelude;

// Re-export the main public types for convenience
pub use core::StoreHaus;
pub use errors::StoreHausError;

// Re-export centralized config
pub use config::{AppConfig, CacheConfig, DatabaseConfig, SignalConfig};

// Re-export internal crates used by macros and public API
// These MUST be public for the generated macro code to work correctly
pub use store_object;
pub use table_derive;
pub use cache_system;
pub use signal_system;
pub use type_mapping;

// Re-export external dependencies used in public API
pub use sqlx;
pub use async_trait;
