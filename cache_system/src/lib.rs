//! Cache system for Redis-based caching
//!
//! This crate provides Redis caching functionality
//! with configurable parameters and error handling.

pub mod errors;
pub mod manager;
pub mod params;
pub mod prelude;

// Re-export centralized config
pub use config::CacheConfig;

pub use errors::CacheError;
pub use manager::CacheManager;
pub use params::CacheParams;
