# System Fields

StoreHaus automatically manages several system fields that are added to every table but are not part of your model definition. These fields provide essential metadata about your records.

## Overview

System fields use the `__field_name__` naming convention to distinguish them from user-defined fields. They are automatically:

- Added to database tables during migration
- Populated during CRUD operations
- Included in signal events
- Available in queries and filters

## Available System Fields

### `__created_at__`

- **Type**: `TIMESTAMP WITH TIME ZONE`
- **Description**: Records when the row was first created
- **Behavior**:
  - Automatically set during `create()` operations
  - Never modified after creation
  - Always set to current UTC timestamp

```sql
-- Automatically added to all tables
__created_at__ TIMESTAMP WITH TIME ZONE DEFAULT NOW()
```

### `__updated_at__`

- **Type**: `TIMESTAMP WITH TIME ZONE`
- **Description**: Records when the row was last modified
- **Behavior**:
  - Set to current UTC timestamp during `create()`
  - Updated to current UTC timestamp during `update()` operations
  - Can be used to track last modification time

```sql
-- Automatically added to all tables
__updated_at__ TIMESTAMP WITH TIME ZONE DEFAULT NOW()
```

### Soft Delete Field

- **Type**: `BOOLEAN`
- **Description**: Configurable soft delete flag
- **Options**:
  - `#[model(soft)]` - Automatically adds `__is_active__` field
  - `#[soft_delete]` - Use custom field name on any boolean field
- **Behavior**:
  - Set to `true` by default on creation
  - Set to `false` during `delete()` operations (soft delete)
  - Records are filtered out of normal queries when `false`

```sql
-- Default for #[model(soft)]
__is_active__ BOOLEAN DEFAULT true

-- Custom field example
enabled BOOLEAN DEFAULT true  -- when using #[soft_delete] on 'enabled' field
```

### `__tags__`

- **Type**: `TEXT[]` (PostgreSQL array)
- **Description**: Array of operation tags for grouping and tracking
- **Behavior**:
  - Empty array `{}` by default
  - Populated when operations include tag parameters
  - Indexed with GIN for efficient searching
  - Available in signal events

```sql
-- Automatically added to all tables with GIN index
__tags__ TEXT[] DEFAULT '{}',
CREATE INDEX IF NOT EXISTS idx_tablename_tags ON tablename USING GIN(__tags__)
```

## Usage in Operations

### Basic CRUD with System Fields

```rust
use uuid::Uuid;
use store_object::{QueryBuilder, QueryFilter};
use serde_json::json;

// Create operation - system fields are automatically populated
let user = User {
    id: Uuid::new_v4(),
    name: "John Doe".to_string(),
    email: "john@example.com".to_string(),
};

// __created_at__ and __updated_at__ are set automatically
// __tags__ is set to provided tags or empty array
let created = user_store.create(
    user,
    Some(vec!["registration".to_string(), "new-user".to_string()])
).await?;
```

### Querying System Fields

```rust
// Find recently created users (last 24 hours)
let recent_users = user_store.find(
    QueryBuilder::new()
        .filter(QueryFilter::gte("__created_at__", json!("2024-01-01T00:00:00Z")))
        .order_by("__created_at__", SortOrder::Desc)
).await?;

// Find users by tags
let tagged_users = user_store.find(
    QueryBuilder::new()
        .filter_by_any_tag(vec!["premium".to_string(), "vip".to_string()])
).await?;

// Find active users (for soft delete models)
let active_users = user_store.find(
    QueryBuilder::new()
        .filter(QueryFilter::eq("__is_active__", json!(true)))
).await?;
```

### Update Operations

```rust
// Update user - __updated_at__ is automatically refreshed
let updated = user_store.update(
    &user_id,
    modified_user,
    Some(vec!["profile-update".to_string()])
).await?;

// __updated_at__ now contains the current timestamp
// __tags__ contains ["profile-update"]
```

### Soft Delete

```rust
// For models with soft delete field
#[model]
#[table(name = "users")]
pub struct User {
    #[primary_key]
    pub id: Uuid,

    #[field(create, update)]
    pub name: String,

    #[soft_delete]
    pub __is_active__: bool,
}

// Delete operation sets __is_active__ = false
let was_deleted = user_store.delete(&user_id).await?;

// Record still exists in database but __is_active__ = false
// Future queries will automatically filter out inactive records
```

## Database Schema

When you run migrations, system fields are automatically added to your tables:

```sql
CREATE TABLE users (
    id UUID PRIMARY KEY,
    name VARCHAR NOT NULL,
    email VARCHAR NOT NULL,

    -- System fields (automatically added)
    __created_at__ TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    __updated_at__ TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    __is_active__ BOOLEAN DEFAULT true,  -- Only for soft delete models
    __tags__ TEXT[] DEFAULT '{}'
);

-- Automatic indexes
CREATE INDEX IF NOT EXISTS idx_users_created_at ON users(__created_at__);
CREATE INDEX IF NOT EXISTS idx_users_updated_at ON users(__updated_at__);
CREATE INDEX IF NOT EXISTS idx_users_tags ON users USING GIN(__tags__);
```

## Signal Integration

System fields are included in all signal events:

```rust
// Set up signal monitoring
signal_manager.add_callback(|event: &DatabaseEvent| {
    println!("Event: {:?}", event.event_type);
    println!("Table: {}", event.table_name);
    println!("Tags: {:?}", event.tags);

    // Access system field values from payload
    if let Some(created_at) = event.payload.get("__created_at__") {
        println!("Created at: {:?}", user.__created_at__);
    }

    if let Some(tags) = event.payload.get("__tags__") {
        println!("Operation tags: {:?}", tags);
    }
});
```

## Query Builder Extensions

Special methods for system fields:

### Tag Filtering

```rust
use store_object::QueryBuilder;

// Find records with any of the specified tags
let results = user_store.find(
    QueryBuilder::new()
        .filter_by_any_tag(vec!["premium".to_string(), "vip".to_string()])
).await?;

// Find records with all specified tags
let results = user_store.find(
    QueryBuilder::new()
        .filter_by_all_tags(vec!["verified".to_string(), "active".to_string()])
).await?;
```

### Date Range Queries

```rust
use chrono::{Utc, Duration};

// Find records created in the last week
let week_ago = Utc::now() - Duration::weeks(1);
let recent = user_store.find(
    QueryBuilder::new()
        .filter(QueryFilter::gte("__created_at__", json!(week_ago)))
).await?;

// Find records updated today
let today = Utc::now().date_naive();
let today_updates = user_store.find(
    QueryBuilder::new()
        .filter(QueryFilter::gte("__updated_at__", json!(format!("{}T00:00:00Z", today))))
).await?;
```

## Best Practices

### Don't Include in Model Definition

❌ **Wrong** - Don't define system fields in your struct:
```rust
#[model]
#[table(name = "users")]
pub struct User {
    #[primary_key]
    pub id: Uuid,
    pub name: String,

    // DON'T DO THIS - system fields are automatic
    pub __created_at__: DateTime<Utc>,
    pub __updated_at__: DateTime<Utc>,
}
```

✅ **Correct** - Let the system manage them:
```rust
#[model]
#[table(name = "users")]
pub struct User {
    #[primary_key]
    pub id: Uuid,

    #[field(create, update)]
    pub name: String,

    // System fields are automatically added
}
```

### Leverage System Fields for Business Logic

```rust
// Audit trail using system fields
async fn get_user_activity_log(user_id: &Uuid) -> Result<Vec<ActivityLog>, StorehausError> {
    // Query operations by user ID and sort by timestamp
    user_store.find(
        QueryBuilder::new()
            .filter(QueryFilter::eq("user_id", json!(user_id)))
            .order_by("__updated_at__", SortOrder::Desc)
            .limit(100)
    ).await
}

// Data retention using timestamps
async fn cleanup_old_records() -> Result<Vec<Uuid>, StorehausError> {
    let cutoff = Utc::now() - Duration::days(365);

    user_store.delete_where(
        QueryBuilder::new()
            .filter(QueryFilter::lt("__updated_at__", json!(cutoff)))
    ).await
}
```

### Tag Strategy

```rust
// Use hierarchical tags for organization
let tags = vec![
    "user:registration".to_string(),
    "source:web".to_string(),
    "campaign:holiday2024".to_string()
];

user_store.create(user, Some(tags)).await?;

// Query by tag prefixes for reporting
let web_registrations = user_store.find(
    QueryBuilder::new()
        .filter_by_any_tag(vec!["source:web".to_string()])
).await?;
```