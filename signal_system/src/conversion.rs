use std::collections::HashMap;
use uuid::Uuid;
use serde::Serialize;

use crate::types::PostgresValue;

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
    T: Into<PostgresValue>
{
    fn from(val: Option<T>) -> Self {
        match val {
            Some(v) => v.into(),
            None => PostgresValue::Null,
        }
    }
}

/// Convert serializable data to PostgresValue::Record
pub fn serialize_to_postgres_record<T: Serialize>(data: &T) -> PostgresValue {
    let payload = serialize_to_postgres_payload(data);
    PostgresValue::Record(payload)
}

/// Default implementation using JSON serialization as fallback
pub fn serialize_to_postgres_payload<T: Serialize>(data: &T) -> HashMap<String, PostgresValue> {
    let mut payload = HashMap::new();

    // Serialize to JSON first, then extract fields
    if let Ok(json_value) = serde_json::to_value(data) {
        if let serde_json::Value::Object(map) = json_value {
            for (key, value) in map {
                let postgres_value = match value {
                    serde_json::Value::String(s) => PostgresValue::Text(s),
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
                    },
                    serde_json::Value::Bool(b) => PostgresValue::Boolean(b),
                    serde_json::Value::Null => PostgresValue::Null,
                    other => PostgresValue::Json(other),
                };
                payload.insert(key, postgres_value);
            }
        }
    }

    payload
}