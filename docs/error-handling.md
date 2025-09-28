# Error Handling Best Practices

StoreHaus follows Rust's idiomatic error handling patterns. This document outlines best practices for handling errors in production applications.

## Core Principles

### ✅ DO: Use Result<T, E> for Fallible Operations

```rust
// Good: Return Result for operations that can fail
async fn get_user_by_id(store: &GenericStore<User>, id: Uuid) -> Result<User, StorehausError> {
    store.find_by_id(id).await
}

// Good: Handle validation errors properly
fn create_validated_name(name: &str) -> Result<ValidatedTableName, ValidationError> {
    ValidatedTableName::new(name)
}
```

### ❌ DON'T: Use unwrap() or expect() in Production Code

```rust
// Bad: Can panic in production
let user = store.find_by_id(id).await.unwrap();

// Bad: Can panic even with descriptive message
let user = store.find_by_id(id).await.expect("User should exist");
```

### ✅ DO: Use Pattern Matching for Error Handling

```rust
// Good: Handle each case explicitly
match store.find_by_id(id).await {
    Ok(user) => {
        println!("Found user: {}", user.name);
        Ok(user)
    }
    Err(StorehausError::NotFound { .. }) => {
        println!("User not found, creating default");
        create_default_user().await
    }
    Err(e) => {
        tracing::error!(error = %e, "Database error");
        Err(e)
    }
}
```

### ✅ DO: Use ? Operator for Error Propagation

```rust
// Good: Propagate errors up the call stack
async fn update_user_email(store: &GenericStore<User>, id: Uuid, email: String) -> Result<User, StorehausError> {
    let mut user = store.find_by_id(id).await?;  // Propagate error
    user.email = email;
    store.update(&user).await  // Propagate error
}
```

## Safe Alternatives to unwrap()

### Option Handling

```rust
// Instead of: record_id.unwrap()
// Use pattern matching:
if let Some(record_id) = record_id {
    cleanup_user_data(record_id);
}

// Or provide a default:
let record_id = record_id.unwrap_or_default();

// Or convert to Result:
let record_id = record_id.ok_or_else(|| {
    StorehausError::InvalidConfiguration {
        message: "Record ID is required".to_string(),
    }
})?;
```

### Result Handling

```rust
// Instead of: operation().unwrap()
// Use match:
match operation() {
    Ok(value) => {
        // Handle success
        process_value(value);
    }
    Err(e) => {
        // Handle error appropriately
        tracing::error!(error = %e, "Operation failed");
        return Err(e);
    }
}

// Or use ? for propagation:
let value = operation()?;
```

## StoreHaus Error Types

### ValidationError

```rust
use store_object::{ValidatedTableName, ValidationError};

// Good: Handle validation errors
match ValidatedTableName::new("user-table") {
    Ok(name) => println!("Valid name: {}", name),
    Err(ValidationError::InvalidCharacters(name)) => {
        tracing::error!("Invalid characters in table name: {}", name);
    }
    Err(ValidationError::TooLong { name, length, max_length }) => {
        tracing::error!("Table name '{}' is too long: {} chars (max {})", name, length, max_length);
    }
    Err(e) => {
        tracing::error!("Validation error: {}", e);
    }
}
```

### StorehausError

```rust
use store_object::StorehausError;

// Good: Handle specific error types
match store.create(&user).await {
    Ok(created_user) => Ok(created_user),
    Err(StorehausError::ValidationError { field, reason, .. }) => {
        tracing::error!("Validation failed for field '{}': {}", field, reason);
        Err(StorehausError::validation("user", &field, &reason))
    }
    Err(StorehausError::DatabaseOperation { operation, .. }) => {
        tracing::error!("Database operation '{}' failed", operation);
        // Maybe retry or fallback
        retry_operation().await
    }
    Err(e) if e.is_transient() => {
        tracing::error!("Transient error, retrying: {}", e);
        // Implement retry logic
        retry_with_backoff(|| store.create(&user)).await
    }
    Err(e) => {
        tracing::error!("Unrecoverable error: {}", e);
        Err(e)
    }
}
```

## Error Context and Logging

### Add Context to Errors

```rust
// Good: Add context when propagating errors
async fn process_user_order(user_id: Uuid, order_data: OrderData) -> Result<Order, StorehausError> {
    let user = store.find_by_id(user_id).await
        .map_err(|e| {
            tracing::error!("Failed to find user {} for order processing: {}", user_id, e);
            e
        })?;

    let order = create_order(&user, order_data).await
        .map_err(|e| {
            tracing::error!("Failed to create order for user {}: {}", user.id, e);
            e
        })?;

    Ok(order)
}
```

### Structured Logging

```rust
// Good: Use structured logging with error context
async fn handle_database_error(error: &StorehausError) {
    let context = error.context();

    log::error!(
        "Database operation failed: {} (table: {}, operation: {})",
        error,
        context.get("table").unwrap_or("unknown"),
        context.get("operation").unwrap_or("unknown")
    );

    // Send to monitoring system
    if error.is_transient() {
        metrics::counter!("database_transient_errors").increment(1);
    } else {
        metrics::counter!("database_permanent_errors").increment(1);
        // Alert on permanent errors
        alert_system::send_alert(&format!("Database error: {}", error)).await;
    }
}
```

## Testing Error Conditions

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_validation_errors() {
        // Test invalid table names
        assert!(ValidatedTableName::new("SELECT").is_err());
        assert!(ValidatedTableName::new("123invalid").is_err());
        assert!(ValidatedTableName::new("").is_err());

        // Test specific error types
        match ValidatedTableName::new("SELECT") {
            Err(ValidationError::ReservedKeyword(name)) => {
                assert_eq!(name, "SELECT");
            }
            _ => panic!("Expected ReservedKeyword error"),
        }
    }

    #[tokio::test]
    async fn test_database_error_handling() {
        let store = create_test_store().await;

        // Test not found error
        match store.find_by_id(Uuid::new_v4()).await {
            Err(StorehausError::NotFound { resource, identifier }) => {
                assert_eq!(resource, "User");
                // Test passed
            }
            _ => panic!("Expected NotFound error"),
        }
    }
}
```

## Proc Macro Error Handling

When writing derive macros, use `syn::Error` for compile-time errors:

```rust
// Good: Structured compile-time errors
pub fn validate_table_name_syn(name: &str, span: proc_macro2::Span) -> syn::Result<()> {
    ValidatedTableName::new(name)
        .map_err(|e| syn::Error::new(span, format!("Invalid table name '{}': {}", name, e)))
}

// Good: Handle parsing errors gracefully
pub fn parse_table_attributes(attrs: &[Attribute]) -> syn::Result<TableAttributes> {
    // ... parsing logic ...

    let table_name = name.ok_or_else(|| {
        syn::Error::new_spanned(attr, "table attribute requires a name parameter")
    })?;

    validate_table_name_syn(&table_name, attr.span())?;

    Ok(TableAttributes { name: table_name })
}
```

## Summary

- **Never use `unwrap()` or `expect()` in production code**
- **Always handle errors explicitly with `Result<T, E>`**
- **Use the `?` operator for error propagation**
- **Provide meaningful error messages and context**
- **Test error conditions thoroughly**
- **Use structured logging for error analysis**
- **Handle transient vs permanent errors differently**

By following these practices, your StoreHaus applications will be more robust, maintainable, and production-ready.