//! Serialization utilities
//!
//! This module provides serialization functions
//! for converting Rust types to PostgreSQL format.

/// Serialization functions for converting Rust data to PostgresValue
use crate::types::PostgresValue;
use serde::Serialize;
use std::collections::HashMap;

/// Convert serializable data to PostgresValue::Record
pub fn serialize_to_postgres_record<T: Serialize>(data: &T) -> PostgresValue {
    let payload = serialize_to_postgres_payload(data);
    PostgresValue::Record(payload)
}

/// Default implementation using JSON serialization as fallback
pub fn serialize_to_postgres_payload<T: Serialize>(data: &T) -> HashMap<String, PostgresValue> {
    let mut payload = HashMap::new();

    // Serialize to JSON first, then extract fields
    if let Ok(serde_json::Value::Object(map)) = serde_json::to_value(data) {
        for (key, value) in map {
            let postgres_value = match value {
                serde_json::Value::String(s) => {
                    // Try to parse as RFC3339 timestamp first
                    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(&s) {
                        PostgresValue::Timestamp(dt.with_timezone(&chrono::Utc))
                    } else {
                        PostgresValue::Text(s)
                    }
                },
                serde_json::Value::Number(n) => {
                    if let Some(i) = n.as_i64() {
                        if i >= i32::MIN as i64 && i <= i32::MAX as i64 {
                            PostgresValue::Integer(i as i32)
                        } else {
                            PostgresValue::BigInt(i)
                        }
                    } else if let Some(f) = n.as_f64() {
                        PostgresValue::Decimal(f.to_string())
                    } else {
                        PostgresValue::Json(serde_json::Value::Number(n))
                    }
                }
                serde_json::Value::Bool(b) => PostgresValue::Boolean(b),
                serde_json::Value::Null => PostgresValue::Null,
                other => PostgresValue::Json(other),
                };
                payload.insert(key, postgres_value);
            }
    }

    payload
}
