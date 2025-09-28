# Type Mapping

Unified type conversion system for the StoreHaus ecosystem.

## Purpose

This crate centralizes all type mapping logic between Rust types and PostgreSQL, ensuring consistency across:

- **table_derive**: SQL DDL generation (CREATE TABLE statements)
- **signal_system**: Runtime value conversion for database events
- **store_object**: Query parameter binding

## Functions

### `rust_type_to_pg_type(rust_type: &str) -> &'static str`

Maps Rust type names to PostgreSQL column types for DDL generation.

```rust
use type_mapping::rust_type_to_pg_type;

assert_eq!(rust_type_to_pg_type("String"), "VARCHAR");
assert_eq!(rust_type_to_pg_type("i32"), "INTEGER");
assert_eq!(rust_type_to_pg_type("uuid::Uuid"), "UUID");
```

### `rust_type_to_postgres_value_variant(rust_type: &str) -> &'static str`

Returns the PostgresValue variant name for a given Rust type.

```rust
use type_mapping::rust_type_to_postgres_value_variant;

assert_eq!(rust_type_to_postgres_value_variant("String"), "Text");
assert_eq!(rust_type_to_postgres_value_variant("i32"), "Integer");
```

### `supports_direct_postgres_conversion(rust_type: &str) -> bool`

Checks if a type can be directly converted to PostgresValue without JSON serialization.

### `pg_type_size_hint(pg_type: &str) -> Option<usize>`

Provides size hints for PostgreSQL types for optimization purposes.

## Supported Types

### Basic Types
| Rust Type | PostgreSQL Type | PostgresValue Variant |
|-----------|-----------------|----------------------|
| `String` | `VARCHAR` | `Text` |
| `i8` | `SMALLINT` | `SmallInt` |
| `i16` | `SMALLINT` | `SmallInt` |
| `i32` | `INTEGER` | `Integer` |
| `i64` | `BIGINT` | `BigInt` |
| `u16` | `INTEGER` | `Integer` |
| `u32` | `BIGINT` | `Integer` |
| `u64` | `NUMERIC(20,0)` | `Decimal` |
| `f32` | `REAL` | `Float` |
| `f64` | `DOUBLE PRECISION` | `Float` |
| `bool` | `BOOLEAN` | `Boolean` |

### UUID Types
| Rust Type | PostgreSQL Type | PostgresValue Variant |
|-----------|-----------------|----------------------|
| `Uuid` / `uuid::Uuid` | `UUID` | `Uuid` |
| `Option<Uuid>` | `UUID` | `Uuid` |

### Timestamp Types
| Rust Type | PostgreSQL Type | PostgresValue Variant |
|-----------|-----------------|----------------------|
| `chrono::DateTime<chrono::Utc>` | `TIMESTAMP WITH TIME ZONE` | `Timestamp` |
| `chrono::NaiveDateTime` | `TIMESTAMP WITH TIME ZONE` | `Timestamp` |
| `chrono::Date<chrono::Utc>` | `DATE` | `Timestamp` |
| `chrono::NaiveDate` | `DATE` | `Timestamp` |
| `Option<chrono::DateTime<chrono::Utc>>` | `TIMESTAMP WITH TIME ZONE` | `Timestamp` |
| `Option<chrono::NaiveDateTime>` | `TIMESTAMP WITH TIME ZONE` | `Timestamp` |
| `Option<chrono::Date<chrono::Utc>>` | `DATE` | `Timestamp` |
| `Option<chrono::NaiveDate>` | `DATE` | `Timestamp` |

### JSON Types
| Rust Type | PostgreSQL Type | PostgresValue Variant |
|-----------|-----------------|----------------------|
| `serde_json::Value` / `Value` | `JSONB` | `Json` |
| `Option<serde_json::Value>` / `Option<Value>` | `JSONB` | `Json` |

### Array Types
| Rust Type | PostgreSQL Type | PostgresValue Variant |
|-----------|-----------------|----------------------|
| `Vec<String>` | `TEXT[]` | `Json` |

### Decimal Types
| Rust Type | PostgreSQL Type | PostgresValue Variant |
|-----------|-----------------|----------------------|
| `rust_decimal::Decimal` | `NUMERIC(28,10)` | `Decimal` |
| `bigdecimal::BigDecimal` | `NUMERIC` | `Decimal` |
| `Option<rust_decimal::Decimal>` | `NUMERIC(28,10)` | `Decimal` |
| `Option<bigdecimal::BigDecimal>` | `NUMERIC` | `Decimal` |

### Optional Basic Types
| Rust Type | PostgreSQL Type | PostgresValue Variant |
|-----------|-----------------|----------------------|
| `Option<String>` | `VARCHAR` | `Text` |
| `Option<i8>` | `SMALLINT` | `SmallInt` |
| `Option<i16>` | `SMALLINT` | `SmallInt` |
| `Option<i32>` | `INTEGER` | `Integer` |
| `Option<i64>` | `BIGINT` | `BigInt` |
| `Option<u16>` | `INTEGER` | `Integer` |
| `Option<u32>` | `BIGINT` | `Integer` |
| `Option<u64>` | `NUMERIC(20,0)` | `Decimal` |
| `Option<f32>` | `REAL` | `Float` |
| `Option<f64>` | `DOUBLE PRECISION` | `Float` |
| `Option<bool>` | `BOOLEAN` | `Boolean` |

## Query Parameter Binding Features

The type mapping system includes intelligent parameter binding that automatically:

- **RFC3339 Timestamp Parsing**: Automatically detects and parses RFC3339 formatted date strings to PostgreSQL timestamps
- **UUID String Parsing**: Automatically detects and parses UUID strings to PostgreSQL UUID type
- **Array Operations**: Supports PostgreSQL array overlap operations (`&&`) for tag filtering

## Array Query Operations

Special support for PostgreSQL array operations:

```rust
// Tag filtering with array overlap
QueryBuilder::new().filter_by_any_tag(vec!["tag1".to_string(), "tag2".to_string()])
// Generates: WHERE __tags__ && ARRAY[$1, $2]
```

## Architecture Benefits

- **Consistency**: Single source of truth for type mappings
- **Maintainability**: Changes to type mappings only need to be made in one place
- **Extensibility**: Easy to add support for new types
- **Documentation**: Clear documentation of supported type conversions