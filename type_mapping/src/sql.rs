//! SQL type conversion utilities
//!
//! This module handles conversion between Rust types
//! and their SQL equivalents.

/// Map Rust type names to PostgreSQL types for DDL generation
pub fn rust_type_to_pg_type(rust_type: &str) -> &'static str {
    // Normalize type string by removing all whitespace for consistent matching
    let normalized = rust_type.replace(" ", "");
    match normalized.as_str() {
        "Uuid" | "uuid::Uuid" => "UUID",
        "Option<Uuid>" => "UUID",
        "String" => "VARCHAR",
        "i8" => "SMALLINT",
        "i16" => "SMALLINT",
        "i32" => "INTEGER",
        "i64" => "BIGINT",
        "u16" => "INTEGER",
        "u32" => "BIGINT",
        "u64" => "NUMERIC(20,0)", // PostgreSQL doesn't have native u64
        "f32" => "REAL",
        "f64" => "DOUBLE PRECISION",
        "bool" => "BOOLEAN",
        "chrono::DateTime<chrono::Utc>" | "chrono::NaiveDateTime" => "TIMESTAMP WITH TIME ZONE",
        "chrono::Date<chrono::Utc>" | "chrono::NaiveDate" => "DATE",
        "rust_decimal::Decimal" => "NUMERIC(28,10)",
        "bigdecimal::BigDecimal" => "NUMERIC",
        "serde_json::Value" | "Value" => "JSONB",
        "Option<serde_json::Value>" | "Option<Value>" => "JSONB",
        // Optional timestamp types (both full and short paths)
        "Option<chrono::DateTime<chrono::Utc>>" | "Option<DateTime<Utc>>" | "Option<chrono::NaiveDateTime>" => "TIMESTAMP WITH TIME ZONE",
        "Option<chrono::Date<chrono::Utc>>" | "Option<chrono::NaiveDate>" => "DATE",
        // Optional basic types
        "Option<String>" => "VARCHAR",
        "Option<i8>" => "SMALLINT",
        "Option<i16>" => "SMALLINT",
        "Option<i32>" => "INTEGER",
        "Option<i64>" => "BIGINT",
        "Option<u16>" => "INTEGER",
        "Option<u32>" => "BIGINT",
        "Option<u64>" => "NUMERIC(20,0)",
        "Option<f32>" => "REAL",
        "Option<f64>" => "DOUBLE PRECISION",
        "Option<bool>" => "BOOLEAN",
        // Optional decimal types
        "Option<rust_decimal::Decimal>" => "NUMERIC(28,10)",
        "Option<bigdecimal::BigDecimal>" => "NUMERIC",
        // Vec types
        "Vec<String>" => "TEXT[]",
        _ => "VARCHAR", // default fallback
    }
}

/// Get the PostgresValue variant name for a Rust type
/// This is used for consistency checks and code generation
pub fn rust_type_to_postgres_value_variant(rust_type: &str) -> &'static str {
    match rust_type.trim() {
        "String" | "&str" => "Text",
        "i8" | "i16" => "SmallInt",
        "i32" => "Integer",
        "i64" => "BigInt",
        "u16" | "u32" => "Integer", // Fit in i32 range for most values
        "u64" => "Decimal",         // Use string representation for large numbers
        "f32" | "f64" => "Float",
        "bool" => "Boolean",
        "Uuid" | "uuid::Uuid" => "Uuid",
        "Option<Uuid>" | "Option < Uuid >" => "Uuid",
        "chrono::DateTime<chrono::Utc>" | "chrono::NaiveDateTime" => "Timestamp",
        "serde_json::Value" | "Value" => "Json",
        "Option<serde_json::Value>" | "Option<Value>" => "Json",
        // Optional timestamp types
        "Option<chrono::DateTime<chrono::Utc>>" | "Option<chrono::NaiveDateTime>" => "Timestamp",
        "Option<chrono::Date<chrono::Utc>>" | "Option<chrono::NaiveDate>" => "Timestamp",
        // Optional basic types
        "Option<String>" => "Text",
        "Option<i8>" | "Option<i16>" => "SmallInt",
        "Option<i32>" => "Integer",
        "Option<i64>" => "BigInt",
        "Option<u16>" | "Option<u32>" => "Integer",
        "Option<u64>" => "Decimal",
        "Option<f32>" | "Option<f64>" => "Float",
        "Option<bool>" => "Boolean",
        "Vec<String>" => "Json", // Will be serialized as JSON array
        _ => "Json", // Default to JSON for unknown types
    }
}

/// Check if a Rust type is Optional (nullable in SQL)
pub fn is_optional_type(rust_type: &str) -> bool {
    rust_type.trim().starts_with("Option")
}

/// Get size hint for PostgreSQL type (for optimization)
pub fn pg_type_size_hint(pg_type: &str) -> Option<usize> {
    match pg_type {
        "BOOLEAN" => Some(1),
        "SMALLINT" => Some(2),
        "INTEGER" => Some(4),
        "BIGINT" => Some(8),
        "REAL" => Some(4),
        "DOUBLE PRECISION" => Some(8),
        "UUID" => Some(16),
        "DATE" => Some(4),
        "TIMESTAMP WITH TIME ZONE" => Some(8),
        _ => None, // Variable size types
    }
}
