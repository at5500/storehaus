//! Validation utilities for type mapping
//!
//! This module provides validation functions
//! for type mapping operations.

/// Check if a Rust type supports direct conversion to PostgresValue
pub fn supports_direct_postgres_conversion(rust_type: &str) -> bool {
    matches!(
        rust_type.trim(),
        "String"
            | "&str"
            | "i8"
            | "i16"
            | "i32"
            | "i64"
            | "u16"
            | "u32"
            | "u64"
            | "f32"
            | "f64"
            | "bool"
            | "Uuid"
            | "uuid::Uuid"
            | "chrono::DateTime<chrono::Utc>"
            | "chrono::NaiveDateTime"
            | "serde_json::Value"
    )
}
