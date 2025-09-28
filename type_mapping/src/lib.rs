//! Unified type mapping between Rust types and PostgreSQL
//! This crate provides consistent mapping logic used across the storehaus ecosystem

pub mod serialize;
pub mod sql;
pub mod types;
pub mod validate;

// Re-export commonly used items for backward compatibility
pub use serialize::{serialize_to_postgres_payload, serialize_to_postgres_record};
pub use sql::{pg_type_size_hint, rust_type_to_pg_type, rust_type_to_postgres_value_variant, is_optional_type};
pub use types::{PostgresValue, ToPostgresPayload};
pub use validate::supports_direct_postgres_conversion;
