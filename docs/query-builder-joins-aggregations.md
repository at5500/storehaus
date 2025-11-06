# QueryBuilder: JOINs and Aggregations

StoreHaus QueryBuilder now supports JOIN operations and aggregation functions for building complex SQL queries.

## Table of Contents

- [JOIN Operations](#join-operations)
- [SELECT and Aggregation Functions](#select-and-aggregation-functions)
- [GROUP BY](#group-by)
- [HAVING](#having)
- [Complete Example](#complete-example)

## JOIN Operations

QueryBuilder supports all major JOIN types:

### JOIN Types

```rust
use storehaus::prelude::*;
// Or alternatively:
// use store_object::query_builder::{JoinType, JoinClause, QueryBuilder};

// INNER JOIN
let query = QueryBuilder::new()
    .join(JoinClause::new_on(
        JoinType::Inner,
        "orders",
        "users.id",
        "orders.user_id",
    ));

// LEFT JOIN
let query = QueryBuilder::new()
    .join(JoinClause::new_on(
        JoinType::Left,
        "profiles",
        "users.id",
        "profiles.user_id",
    ));

// RIGHT JOIN
let query = QueryBuilder::new()
    .join(JoinClause::new_on(
        JoinType::Right,
        "products",
        "orders.product_id",
        "products.id",
    ));

// FULL OUTER JOIN
let query = QueryBuilder::new()
    .join(JoinClause::new_on(
        JoinType::Full,
        "addresses",
        "users.id",
        "addresses.user_id",
    ));

// CROSS JOIN
let query = QueryBuilder::new()
    .join(JoinClause::new_on(
        JoinType::Cross,
        "categories",
        "products.id",
        "categories.id",
    ));
```

### JOIN with Aliases

```rust
let query = QueryBuilder::new()
    .join(
        JoinClause::new_on(
            JoinType::Left,
            "orders",
            "users.id",
            "o.user_id",
        )
        .with_alias("o")
    );
```

### JOIN with USING

```rust
let query = QueryBuilder::new()
    .join(JoinClause::new_using(
        JoinType::Inner,
        "profiles",
        vec!["user_id".to_string()],
    ));
```

### Multiple JOINs

```rust
let query = QueryBuilder::new()
    .join(JoinClause::new_on(
        JoinType::Inner,
        "orders",
        "users.id",
        "orders.user_id",
    ))
    .join(JoinClause::new_on(
        JoinType::Left,
        "order_items",
        "orders.id",
        "order_items.order_id",
    ));
```

## SELECT and Aggregation Functions

### Selecting Specific Fields

```rust
// All query builder types are available via prelude
use storehaus::prelude::*;

let query = QueryBuilder::new()
    .select_fields(vec![
        SelectField::field("users.name"),
        SelectField::field("users.email"),
        SelectField::field("orders.total_amount"),
    ]);

// Generates: SELECT users.name, users.email, orders.total_amount FROM ...
```

### Fields with Aliases

```rust
let query = QueryBuilder::new()
    .select(SelectField::field_as("user_name", "name"))
    .select(SelectField::field_as("user_email", "email"));

// Generates: SELECT user_name AS name, user_email AS email FROM ...
```

### Aggregation Functions

```rust

// COUNT(*)
let query = QueryBuilder::new()
    .select(SelectField::count_all().with_alias("total"));

// COUNT(field)
let query = QueryBuilder::new()
    .select(SelectField::count("id").with_alias("user_count"));

// COUNT(DISTINCT field)
let query = QueryBuilder::new()
    .select(SelectField::count_distinct("user_id").with_alias("unique_users"));

// SUM
let query = QueryBuilder::new()
    .select(SelectField::sum("amount").with_alias("total_amount"));

// AVG
let query = QueryBuilder::new()
    .select(SelectField::avg("price").with_alias("average_price"));

// MIN
let query = QueryBuilder::new()
    .select(SelectField::min("created_at").with_alias("first_order"));

// MAX
let query = QueryBuilder::new()
    .select(SelectField::max("updated_at").with_alias("last_update"));
```

## GROUP BY

### Grouping by Single Field

```rust
// GroupBy is also available via prelude
let query = QueryBuilder::new()
    .select_fields(vec![
        SelectField::field("status"),
        SelectField::count_all().with_alias("count"),
    ])
    .group_by(GroupBy::single("status"));

// Generates: SELECT status, COUNT(*) AS count FROM orders GROUP BY status
```

### Grouping by Multiple Fields

```rust
let query = QueryBuilder::new()
    .select_fields(vec![
        SelectField::field("category"),
        SelectField::field("status"),
        SelectField::count_all().with_alias("count"),
    ])
    .group_by(GroupBy::new(vec![
        "category".to_string(),
        "status".to_string(),
    ]));

// Generates:
// SELECT category, status, COUNT(*) AS count
// FROM products
// GROUP BY category, status
```

## HAVING

HAVING is used to filter aggregated results after GROUP BY.

### Simple HAVING Conditions

```rust
use serde_json::json;

let query = QueryBuilder::new()
    .select_fields(vec![
        SelectField::field("category"),
        SelectField::count_all().with_alias("product_count"),
    ])
    .group_by(
        GroupBy::single("category")
            .having(QueryFilter::gt("COUNT(*)", json!(10)))
    );

// Generates:
// SELECT category, COUNT(*) AS product_count
// FROM products
// GROUP BY category
// HAVING COUNT(*) > $1
```

### Multiple HAVING Conditions

```rust
let query = QueryBuilder::new()
    .select_fields(vec![
        SelectField::field("user_id"),
        SelectField::count("order_id").with_alias("order_count"),
        SelectField::sum("total_amount").with_alias("total_spent"),
    ])
    .group_by(
        GroupBy::single("user_id")
            .having(QueryFilter::gt("COUNT(order_id)", json!(5)))
            .having(QueryFilter::gte("SUM(total_amount)", json!(1000.0)))
    );

// Generates:
// SELECT user_id, COUNT(order_id) AS order_count, SUM(total_amount) AS total_spent
// FROM orders
// GROUP BY user_id
// HAVING COUNT(order_id) > $1 AND SUM(total_amount) >= $2
```

## Complete Example

### Complex Query with JOIN, Aggregation, and Filtering

```rust
use storehaus::prelude::*;
use serde_json::json;

let query = QueryBuilder::new()
    // SELECT clause
    .select_fields(vec![
        SelectField::field("users.name"),
        SelectField::field("users.email"),
        SelectField::count("orders.id").with_alias("order_count"),
        SelectField::sum("orders.total_amount").with_alias("total_spent"),
        SelectField::avg("orders.total_amount").with_alias("avg_order_value"),
    ])
    // JOIN clause
    .join(JoinClause::new_on(
        JoinType::Inner,
        "orders",
        "users.id",
        "orders.user_id",
    ))
    // WHERE clause - filtering before aggregation
    .filter(QueryFilter::eq("orders.status", json!("completed")))
    .filter(QueryFilter::gte(
        "orders.created_at",
        json!("2024-01-01"),
    ))
    // GROUP BY clause
    .group_by(
        GroupBy::new(vec!["users.name".to_string(), "users.email".to_string()])
            // HAVING clause - filtering after aggregation
            .having(QueryFilter::gt("COUNT(orders.id)", json!(3)))
            .having(QueryFilter::gte("SUM(orders.total_amount)", json!(500.0)))
    )
    // ORDER BY clause
    .order_by("total_spent", SortOrder::Desc)
    // LIMIT clause
    .limit(10);

// Generates:
// SELECT
//   users.name,
//   users.email,
//   COUNT(orders.id) AS order_count,
//   SUM(orders.total_amount) AS total_spent,
//   AVG(orders.total_amount) AS avg_order_value
// FROM users
// INNER JOIN orders ON users.id = orders.user_id
// WHERE orders.status = $1 AND orders.created_at >= $2
// GROUP BY users.name, users.email
// HAVING COUNT(orders.id) > $3 AND SUM(orders.total_amount) >= $4
// ORDER BY total_spent DESC
// LIMIT 10
```

### Using build_full() to Get All Query Parts

```rust
let (
    select_clause,      // "users.name, COUNT(orders.id) AS order_count"
    join_clause,        // "INNER JOIN orders ON users.id = orders.user_id"
    where_clause,       // "WHERE orders.status = $1"
    group_by_clause,    // "GROUP BY users.name"
    having_clause,      // "HAVING COUNT(orders.id) > $2"
    order_clause,       // "ORDER BY order_count DESC"
    limit_clause,       // "LIMIT 10"
    where_values,       // [String("completed")]
    having_values,      // [Number(3)]
) = query.build_full();

// Assemble the complete SQL query
let sql = format!(
    "SELECT {} FROM users {} {} {} {} {} {}",
    select_clause,
    join_clause,
    where_clause,
    group_by_clause,
    having_clause,
    order_clause,
    limit_clause
);
```

### Backward Compatibility

For simple queries without JOINs and aggregations, you can continue using the old `build()` method:

```rust
let query = QueryBuilder::new()
    .filter(QueryFilter::eq("status", json!("active")))
    .order_by("name", SortOrder::Asc)
    .limit(20);

let (where_clause, order_clause, limit_clause, values) = query.build();
// Works as before!
```

## Practical Examples

### Example 1: Order Statistics by User

```rust
let query = QueryBuilder::new()
    .select_fields(vec![
        SelectField::field("users.name"),
        SelectField::count("orders.id").with_alias("total_orders"),
        SelectField::sum("orders.total_amount").with_alias("revenue"),
    ])
    .join(JoinClause::new_on(
        JoinType::Left,
        "orders",
        "users.id",
        "orders.user_id",
    ))
    .group_by(GroupBy::single("users.name"))
    .order_by("revenue", SortOrder::Desc);
```

### Example 2: Top Categories by Product Count

```rust
let query = QueryBuilder::new()
    .select_fields(vec![
        SelectField::field("category"),
        SelectField::count("id").with_alias("product_count"),
        SelectField::avg("price").with_alias("avg_price"),
    ])
    .filter(QueryFilter::eq("is_active", json!(true)))
    .group_by(
        GroupBy::single("category")
            .having(QueryFilter::gte("COUNT(id)", json!(5)))
    )
    .order_by("product_count", SortOrder::Desc)
    .limit(10);
```

### Example 3: Sales Analysis with 3-Table JOIN

```rust
let query = QueryBuilder::new()
    .select_fields(vec![
        SelectField::field("products.name"),
        SelectField::field("categories.name").with_alias("category"),
        SelectField::sum("order_items.quantity").with_alias("total_sold"),
        SelectField::sum("order_items.price * order_items.quantity")
            .with_alias("total_revenue"),
    ])
    .join(JoinClause::new_on(
        JoinType::Inner,
        "categories",
        "products.category_id",
        "categories.id",
    ))
    .join(JoinClause::new_on(
        JoinType::Inner,
        "order_items",
        "products.id",
        "order_items.product_id",
    ))
    .group_by(GroupBy::new(vec![
        "products.name".to_string(),
        "categories.name".to_string(),
    ]))
    .order_by("total_revenue", SortOrder::Desc)
    .limit(20);
```

## Running the Example

A complete working example is available in `examples/join_and_aggregation_demo.rs`:

```bash
# Make sure PostgreSQL is running
docker-compose up -d

# Run the example
cargo run --example join_and_aggregation_demo
```

## API Reference

### JoinType

- `JoinType::Inner` - INNER JOIN
- `JoinType::Left` - LEFT JOIN
- `JoinType::Right` - RIGHT JOIN
- `JoinType::Full` - FULL OUTER JOIN
- `JoinType::Cross` - CROSS JOIN

### SelectField

- `SelectField::All` - SELECT *
- `SelectField::field(name)` - SELECT field_name
- `SelectField::field_as(field, alias)` - SELECT field AS alias
- `SelectField::count_all()` - COUNT(*)
- `SelectField::count(field)` - COUNT(field)
- `SelectField::count_distinct(field)` - COUNT(DISTINCT field)
- `SelectField::sum(field)` - SUM(field)
- `SelectField::avg(field)` - AVG(field)
- `SelectField::min(field)` - MIN(field)
- `SelectField::max(field)` - MAX(field)

### GroupBy

- `GroupBy::single(field)` - GROUP BY single field
- `GroupBy::new(fields)` - GROUP BY multiple fields
- `.having(filter)` - Add HAVING condition
- `.with_having(filters)` - Set multiple HAVING conditions

### QueryBuilder New Methods

- `.select(field)` - Add field to SELECT
- `.select_fields(fields)` - Add multiple fields to SELECT
- `.join(join_clause)` - Add JOIN
- `.joins(join_clauses)` - Add multiple JOINs
- `.group_by(group_by)` - Set GROUP BY
- `.build_full()` - Get all query parts including SELECT, JOIN, GROUP BY, HAVING

## Notes

1. **Backward Compatibility**: The `build()` method continues to work for existing code
2. **Parameterization**: All values in WHERE and HAVING use parameterized queries (SQL injection protection)
3. **Flexibility**: You can combine JOINs, aggregations, filtering, and sorting in any way
4. **PostgreSQL**: Implementation is optimized for PostgreSQL

## Next Steps

- Explore the [complete example](../examples/join_and_aggregation_demo.rs)
- Check out the [QueryBuilder documentation](./models.md)
- Try creating your own complex queries!