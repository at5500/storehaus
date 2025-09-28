//! Type mapping definitions
//!
//! This module provides Rust to PostgreSQL type mapping
//! utilities and conversion functions.

/// PostgreSQL data types for event payloads and runtime values
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PostgresValue {
    Text(String),
    Integer(i32),
    BigInt(i64),
    SmallInt(i16),
    Float(f64),
    Boolean(bool),
    Uuid(Uuid),
    Timestamp(chrono::DateTime<chrono::Utc>),
    Decimal(String), // Store as string to preserve precision
    Json(serde_json::Value),
    Array(Vec<PostgresValue>),
    Record(HashMap<String, PostgresValue>), // Associative array for full records
    Null,
}

/// Trait for converting model fields to PostgresValue
pub trait ToPostgresPayload {
    fn to_postgres_payload(&self) -> HashMap<String, PostgresValue>;
}

/// Convert basic Rust types to PostgresValue
impl From<String> for PostgresValue {
    fn from(val: String) -> Self {
        PostgresValue::Text(val)
    }
}

impl From<&str> for PostgresValue {
    fn from(val: &str) -> Self {
        PostgresValue::Text(val.to_string())
    }
}

impl From<i32> for PostgresValue {
    fn from(val: i32) -> Self {
        PostgresValue::Integer(val)
    }
}

impl From<i64> for PostgresValue {
    fn from(val: i64) -> Self {
        PostgresValue::BigInt(val)
    }
}

impl From<i16> for PostgresValue {
    fn from(val: i16) -> Self {
        PostgresValue::SmallInt(val)
    }
}

impl From<bool> for PostgresValue {
    fn from(val: bool) -> Self {
        PostgresValue::Boolean(val)
    }
}

impl From<Uuid> for PostgresValue {
    fn from(val: Uuid) -> Self {
        PostgresValue::Uuid(val)
    }
}

impl From<chrono::DateTime<chrono::Utc>> for PostgresValue {
    fn from(val: chrono::DateTime<chrono::Utc>) -> Self {
        PostgresValue::Timestamp(val)
    }
}

impl From<serde_json::Value> for PostgresValue {
    fn from(val: serde_json::Value) -> Self {
        PostgresValue::Json(val)
    }
}

impl<T> From<Option<T>> for PostgresValue
where
    T: Into<PostgresValue>,
{
    fn from(val: Option<T>) -> Self {
        match val {
            Some(v) => v.into(),
            None => PostgresValue::Null,
        }
    }
}
