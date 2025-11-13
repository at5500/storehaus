# Database Transactions

StoreHaus provides comprehensive support for database transactions, allowing you to execute multiple operations atomically. This ensures data consistency and integrity across complex operations.

## Table of Contents

- [Overview](#overview)
- [Basic Usage](#basic-usage)
- [API Reference](#api-reference)
- [Advanced Patterns](#advanced-patterns)
- [Best Practices](#best-practices)
- [Error Handling](#error-handling)

## Overview

Transactions in StoreHaus guarantee that a series of database operations either all succeed or all fail together. This is critical for maintaining data consistency, especially in financial applications, inventory management, or any scenario requiring atomic operations.

### Key Features

- **Atomic Operations** - All or nothing execution
- **No Direct SQL Required** - Uses QueryBuilder for type safety
- **Flexible Executor Pattern** - Works with Pool or Transaction
- **Integration with StoreHaus Features** - Supports increment/decrement, filters, and all query operations
- **Proper Error Propagation** - Clear rollback on failures

### When to Use Transactions

Use transactions when you need to:
- Transfer money between accounts
- Update related records that must stay consistent
- Perform multiple dependent operations
- Ensure data integrity across operations
- Rollback on partial failures

## Basic Usage

### Simple Transaction

```rust
use storehaus::prelude::*;
use serde_json::json;

// Begin transaction from pool
let mut tx = pool.begin().await?;

// Perform operations within transaction
let query = QueryBuilder::new()
    .filter(QueryFilter::eq("wallet_id", json!(wallet_id)))
    .update(UpdateSet::new().increment("balance", json!(100)));

store.update_where_with_executor(&mut tx, query, None).await?;

// Commit transaction
tx.commit().await?;
```

### Transfer Between Accounts

A classic example demonstrating atomic transfers:

```rust
use storehaus::prelude::*;
use serde_json::json;
use rust_decimal::Decimal;
use uuid::Uuid;

async fn transfer_funds(
    store: &GenericStore<Wallet>,
    pool: &PgPool,
    from_wallet: Uuid,
    to_wallet: Uuid,
    amount: Decimal,
) -> Result<(), Box<dyn std::error::Error>> {
    // Start transaction
    let mut tx = pool.begin().await?;

    // Debit from source wallet
    let debit_query = QueryBuilder::new()
        .filter(QueryFilter::eq("wallet_id", json!(from_wallet)))
        .update(UpdateSet::new().decrement("balance", json!(amount)));

    let debited = store
        .update_where_with_executor(&mut tx, debit_query, None)
        .await?;

    // Verify sufficient balance
    if debited.is_empty() {
        tx.rollback().await?;
        return Err("Insufficient balance or wallet not found".into());
    }

    // Credit to destination wallet
    let credit_query = QueryBuilder::new()
        .filter(QueryFilter::eq("wallet_id", json!(to_wallet)))
        .update(UpdateSet::new().increment("balance", json!(amount)));

    let credited = store
        .update_where_with_executor(&mut tx, credit_query, None)
        .await?;

    // Verify destination exists
    if credited.is_empty() {
        tx.rollback().await?;
        return Err("Destination wallet not found".into());
    }

    // Commit the transaction
    tx.commit().await?;

    Ok(())
}
```

## API Reference

### `update_where_with_executor`

Execute an update query within a transaction context.

```rust
pub async fn update_where_with_executor<'e, E>(
    &self,
    executor: E,
    query: QueryBuilder,
    data: Option<T>,
) -> Result<Vec<T>, StorehausError>
where
    E: sqlx::Executor<'e, Database = sqlx::Postgres>
```

**Parameters:**
- `executor` - Any sqlx executor (Pool or Transaction)
- `query` - QueryBuilder with filters and update operations
- `data` - Optional model data (not needed when using UpdateSet)

**Returns:** Vector of updated records

**Example:**
```rust
let query = QueryBuilder::new()
    .filter(QueryFilter::eq("id", json!(id)))
    .update(UpdateSet::new().increment("count", json!(1)));

let updated = store.update_where_with_executor(&mut tx, query, None).await?;
```

### Transaction Methods

#### `begin()`
```rust
let mut tx = pool.begin().await?;
```
Start a new database transaction.

#### `commit()`
```rust
tx.commit().await?;
```
Commit all operations in the transaction.

#### `rollback()`
```rust
tx.rollback().await?;
```
Rollback all operations and restore previous state.

## Advanced Patterns

### Multiple Model Updates

```rust
use storehaus::prelude::*;
use serde_json::json;

async fn update_inventory_and_orders(
    product_store: &GenericStore<Product>,
    order_store: &GenericStore<Order>,
    pool: &PgPool,
    product_id: Uuid,
    order_id: Uuid,
    quantity: i32,
) -> Result<(), StorehausError> {
    let mut tx = pool.begin().await?;

    // Decrease inventory
    let inv_query = QueryBuilder::new()
        .filter(QueryFilter::eq("product_id", json!(product_id)))
        .update(UpdateSet::new().decrement("stock", json!(quantity)));

    product_store.update_where_with_executor(&mut tx, inv_query, None).await?;

    // Mark order as fulfilled
    let order_query = QueryBuilder::new()
        .filter(QueryFilter::eq("order_id", json!(order_id)))
        .update(UpdateSet::new().set("status", json!("fulfilled")));

    order_store.update_where_with_executor(&mut tx, order_query, None).await?;

    tx.commit().await?;
    Ok(())
}
```

### Conditional Updates

```rust
async fn conditional_transfer(
    store: &GenericStore<Wallet>,
    pool: &PgPool,
    wallet_id: Uuid,
    min_balance: Decimal,
    withdraw_amount: Decimal,
) -> Result<Option<Wallet>, StorehausError> {
    let mut tx = pool.begin().await?;

    // Update only if balance >= min_balance + withdraw_amount
    let query = QueryBuilder::new()
        .filter(QueryFilter::eq("wallet_id", json!(wallet_id)))
        .filter(QueryFilter::gte("balance", json!(min_balance + withdraw_amount)))
        .update(UpdateSet::new().decrement("balance", json!(withdraw_amount)));

    let result = store.update_where_with_executor(&mut tx, query, None).await?;

    if result.is_empty() {
        // Condition not met, rollback
        tx.rollback().await?;
        return Ok(None);
    }

    tx.commit().await?;
    Ok(result.into_iter().next())
}
```

### Nested Transaction Logic

```rust
async fn process_payment_with_fees(
    wallet_store: &GenericStore<Wallet>,
    transaction_store: &GenericStore<Transaction>,
    pool: &PgPool,
    payer_id: Uuid,
    payee_id: Uuid,
    amount: Decimal,
    fee: Decimal,
) -> Result<(Wallet, Wallet, Transaction), StorehausError> {
    let mut tx = pool.begin().await?;

    // Debit payer (amount + fee)
    let debit_query = QueryBuilder::new()
        .filter(QueryFilter::eq("wallet_id", json!(payer_id)))
        .update(UpdateSet::new().decrement("balance", json!(amount + fee)));

    let mut payer = wallet_store
        .update_where_with_executor(&mut tx, debit_query, None)
        .await?
        .into_iter()
        .next()
        .ok_or_else(|| StorehausError::NotFound("Payer wallet not found".to_string()))?;

    // Credit payee (just amount, fee goes to platform)
    let credit_query = QueryBuilder::new()
        .filter(QueryFilter::eq("wallet_id", json!(payee_id)))
        .update(UpdateSet::new().increment("balance", json!(amount)));

    let mut payee = wallet_store
        .update_where_with_executor(&mut tx, credit_query, None)
        .await?
        .into_iter()
        .next()
        .ok_or_else(|| StorehausError::NotFound("Payee wallet not found".to_string()))?;

    // Create transaction record
    let transaction = Transaction::new(
        Uuid::new_v4(),
        payer_id,
        payee_id,
        amount,
        fee,
        "completed".to_string(),
    );

    // Execute insert using the transaction
    let created_tx = transaction.execute_create_tx(&mut tx).await?;

    // Commit everything atomically
    tx.commit().await?;

    Ok((payer, payee, created_tx))
}
```

## Best Practices

### 1. Always Handle Errors

Always handle transaction errors and rollback explicitly:

```rust
let mut tx = pool.begin().await?;

match perform_operations(&mut tx).await {
    Ok(_) => tx.commit().await?,
    Err(e) => {
        tx.rollback().await?;
        return Err(e);
    }
}
```

### 2. Keep Transactions Short

Minimize transaction duration to avoid blocking other operations:

```rust
// ❌ Bad: Long-running operation in transaction
let mut tx = pool.begin().await?;
let data = fetch_from_external_api().await?; // This could take seconds!
store.update_where_with_executor(&mut tx, query, None).await?;
tx.commit().await?;

// ✅ Good: Prepare data first, then transact
let data = fetch_from_external_api().await?;
let mut tx = pool.begin().await?;
store.update_where_with_executor(&mut tx, query, None).await?;
tx.commit().await?;
```

### 3. Use Specific Filters

Always use specific WHERE conditions to avoid accidental updates:

```rust
// ❌ Bad: Could update multiple records unintentionally
let query = QueryBuilder::new()
    .filter(QueryFilter::eq("status", json!("pending")))
    .update(UpdateSet::new().set("status", json!("processed")));

// ✅ Good: Specific ID-based update
let query = QueryBuilder::new()
    .filter(QueryFilter::eq("order_id", json!(specific_order_id)))
    .update(UpdateSet::new().set("status", json!("processed")));
```

### 4. Verify Results

Check that operations affected the expected number of records:

```rust
let updated = store.update_where_with_executor(&mut tx, query, None).await?;

if updated.len() != 1 {
    tx.rollback().await?;
    return Err("Expected to update exactly 1 record".into());
}
```

### 5. Use Optimistic Locking

For concurrent updates, use version numbers or timestamps:

```rust
let query = QueryBuilder::new()
    .filter(QueryFilter::eq("id", json!(id)))
    .filter(QueryFilter::eq("version", json!(expected_version))) // Optimistic lock
    .update(UpdateSet::new()
        .set("data", json!(new_data))
        .increment("version", json!(1))
    );

let updated = store.update_where_with_executor(&mut tx, query, None).await?;

if updated.is_empty() {
    tx.rollback().await?;
    return Err("Concurrent modification detected".into());
}
```

## Error Handling

### Common Transaction Errors

#### 1. Deadlocks

```rust
use tokio::time::{sleep, Duration};

async fn transfer_with_retry(
    store: &GenericStore<Wallet>,
    pool: &PgPool,
    from: Uuid,
    to: Uuid,
    amount: Decimal,
    max_retries: u32,
) -> Result<(), StorehausError> {
    let mut attempt = 0;

    loop {
        match perform_transfer(store, pool, from, to, amount).await {
            Ok(_) => return Ok(()),
            Err(e) if is_deadlock_error(&e) && attempt < max_retries => {
                attempt += 1;
                sleep(Duration::from_millis(100 * attempt as u64)).await;
                continue;
            }
            Err(e) => return Err(e),
        }
    }
}

fn is_deadlock_error(e: &StorehausError) -> bool {
    // Check if error is a deadlock
    matches!(e, StorehausError::DatabaseError(msg) if msg.contains("deadlock"))
}
```

#### 2. Constraint Violations

```rust
async fn safe_update(
    store: &GenericStore<Account>,
    pool: &PgPool,
    account_id: Uuid,
    new_balance: Decimal,
) -> Result<Account, StorehausError> {
    let mut tx = pool.begin().await?;

    let query = QueryBuilder::new()
        .filter(QueryFilter::eq("id", json!(account_id)))
        .filter(QueryFilter::gte("balance", json!(Decimal::ZERO))) // Check constraint
        .update(UpdateSet::new().set("balance", json!(new_balance)));

    let updated = store.update_where_with_executor(&mut tx, query, None).await?;

    if updated.is_empty() {
        tx.rollback().await?;
        return Err(StorehausError::ValidationError(
            "Update would violate balance constraint".to_string()
        ));
    }

    tx.commit().await?;
    Ok(updated.into_iter().next().unwrap())
}
```

#### 3. Timeout Handling

```rust
use tokio::time::timeout;
use std::time::Duration;

async fn transfer_with_timeout(
    store: &GenericStore<Wallet>,
    pool: &PgPool,
    from: Uuid,
    to: Uuid,
    amount: Decimal,
) -> Result<(), Box<dyn std::error::Error>> {
    let result = timeout(
        Duration::from_secs(5),
        perform_transfer(store, pool, from, to, amount)
    ).await;

    match result {
        Ok(Ok(())) => Ok(()),
        Ok(Err(e)) => Err(e.into()),
        Err(_) => Err("Transaction timeout".into()),
    }
}
```

## Performance Considerations

### Transaction Scope

Keep transactions as small as possible:

```rust
// ❌ Bad: Large transaction scope
let mut tx = pool.begin().await?;
let users = fetch_all_users(&mut tx).await?;
for user in users {
    update_user(&mut tx, user).await?; // Long-running loop
}
tx.commit().await?;

// ✅ Good: Batch operations
let users = fetch_all_users(pool).await?; // Outside transaction
let user_ids: Vec<Uuid> = users.iter().map(|u| u.id).collect();

let mut tx = pool.begin().await?;
let query = QueryBuilder::new()
    .filter(QueryFilter::r#in("id", json!(user_ids)))
    .update(UpdateSet::new().set("updated", json!(true)));
store.update_where_with_executor(&mut tx, query, None).await?;
tx.commit().await?;
```

### Batch Size

For bulk operations, use reasonable batch sizes:

```rust
const BATCH_SIZE: usize = 100;

async fn bulk_update(
    store: &GenericStore<Record>,
    pool: &PgPool,
    records: Vec<Uuid>,
) -> Result<(), StorehausError> {
    for chunk in records.chunks(BATCH_SIZE) {
        let mut tx = pool.begin().await?;

        let query = QueryBuilder::new()
            .filter(QueryFilter::r#in("id", json!(chunk)))
            .update(UpdateSet::new().set("processed", json!(true)));

        store.update_where_with_executor(&mut tx, query, None).await?;
        tx.commit().await?;
    }

    Ok(())
}
```

## Integration with Other Features

### Signals and Events

Note that when using `update_where_with_executor`, signals are **not automatically emitted** (to avoid emitting before commit). Handle signals after transaction commits:

```rust
let mut tx = pool.begin().await?;

let query = QueryBuilder::new()
    .filter(QueryFilter::eq("id", json!(id)))
    .update(UpdateSet::new().increment("balance", json!(amount)));

let updated = store.update_where_with_executor(&mut tx, query, None).await?;

tx.commit().await?;

// Emit signals AFTER successful commit
if let Some(signal_manager) = signal_manager {
    for record in &updated {
        let event = create_balance_updated_event(record);
        signal_manager.emit(event).await;
    }
}
```

### Cache Invalidation

Similarly, invalidate cache after transaction commits:

```rust
let mut tx = pool.begin().await?;

let updated = store.update_where_with_executor(&mut tx, query, None).await?;

tx.commit().await?;

// Invalidate cache AFTER successful commit
if let Some(cache) = cache_manager {
    for record in &updated {
        cache.invalidate(&record.id).await?;
    }
}
```

## Summary

StoreHaus transactions provide:

- ✅ **Type-safe atomic operations** without raw SQL
- ✅ **QueryBuilder integration** for consistent API
- ✅ **Flexible executor pattern** for composability
- ✅ **Proper error handling** with rollback support
- ✅ **Integration ready** for signals and caching

For more information:
- [Query Builder Documentation](query-builder-joins-aggregations.md)
- [Error Handling Guide](error-handling.md)
- [Models and Updates](models.md)