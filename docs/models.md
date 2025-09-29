# Model Definitions

This guide covers how to define data models in StoreHaus using derive macros and attributes.

## Model Structure and SQL Generation

StoreHaus provides automatic SQL generation and system field management for your data models:

```mermaid
classDiagram
    class User {
        +Uuid id
        +String name
        +String email
        +Option phone
        +Option age
        +DateTime \_\_created_at\_\_
        +DateTime \_\_updated_at\_\_
        +Vec \_\_tags\_\_
        +new(id, name, email, phone, age) User
        +table_name() String
        +create_fields() Vec
    }

    class Product {
        +Uuid id
        +String name
        +i32 price
        +Option description
        +DateTime \_\_created_at\_\_
        +DateTime \_\_updated_at\_\_
        +Vec \_\_tags\_\_
        +bool \_\_is_active\_\_
        +new(id, name, price, description) Product
        +table_name() String
    }

    class Log {
        +i32 id
        +String message
        +String level
        +DateTime \_\_created_at\_\_
        +DateTime \_\_updated_at\_\_
        +Vec \_\_tags\_\_
        +new(message, level) Log
        +table_name() String
    }

    class UsersTable {
        +UUID id
        +VARCHAR name
        +VARCHAR email
        +VARCHAR phone
        +INTEGER age
        +TIMESTAMP \_\_created_at\_\_
        +TIMESTAMP \_\_updated_at\_\_
        +TEXT[] \_\_tags\_\_
    }

    class ProductsTable {
        +UUID id
        +VARCHAR name
        +INTEGER price
        +VARCHAR description
        +BOOLEAN \_\_is_active\_\_
        +TIMESTAMP \_\_created_at\_\_
        +TIMESTAMP \_\_updated_at\_\_
        +TEXT[] \_\_tags\_\_
    }

    class LogsTable {
        +SERIAL id
        +VARCHAR message
        +VARCHAR level
        +TIMESTAMP \_\_created_at\_\_
        +TIMESTAMP \_\_updated_at\_\_
        +TEXT[] \_\_tags\_\_
    }

    User --> UsersTable : generates
    Product --> ProductsTable : generates
    Log --> LogsTable : generates
```

### Model Annotations
- **User Model**: `#[model]` with `#[soft_delete]` - Custom soft delete field
- **Product Model**: `#[model]` with `#[table(auto_soft_delete)]` - Automatic soft delete
- **Log Model**: `#[model]` with `#[table(auto_increment)]` - Auto-incrementing primary key

### Field Attributes Reference
- **`#[primary_key]`** - Primary key field
- **`#[field(create)]`** - Used in INSERT operations
- **`#[field(update)]`** - Used in UPDATE operations
- **`#[field(create, update)]`** - Used in both operations
- **`#[field(readonly)]`** - Never modified by StoreHaus
- **`#[soft_delete]`** - Soft delete field

### System Fields (Automatic)
- **`\_\_created_at\_\_`** - Creation timestamp
- **`\_\_updated_at\_\_`** - Last update timestamp
- **`\_\_tags\_\_`** - Operation tags array
- **`\_\_is_active\_\_`** - Soft delete flag (when using soft delete)

## Basic Model Definition

### Simple Model

```rust
use storehaus::prelude::*;

#[model]
#[table(name = "users")]
pub struct User {
    #[primary_key]
    pub id: Uuid,

    #[field(create, update)]
    pub name: String,

    #[field(create, update)]
    pub email: String,

    #[field(create, update)]
    pub phone: Option<String>,

    #[field(create, update)]
    pub age: Option<i32>,
}
```

### Model with Auto Soft Delete

```rust
#[model]
#[table(name = "products", auto_soft_delete)]
pub struct Product {
    #[primary_key]
    pub id: Uuid,

    #[field(create, update)]
    pub name: String,

    #[field(create, update)]
    pub price: i32,

    #[field(create, update)]
    pub description: Option<String>,
}
```

> **Note**: With `#[table(auto_soft_delete)]`, the `__is_active__` field is automatically added by the macro. You don't need to define it manually.

### Model with Custom Soft Delete Field

```rust
#[model]
#[table(name = "users")]
pub struct User {
    #[primary_key]
    pub id: Uuid,

    #[field(create, update)]
    pub name: String,

    #[field(create, update)]
    pub email: String,

    #[soft_delete]
    pub __is_active__: bool,
}
```

> **Note**: With `#[soft_delete]`, you manually define the soft delete field and have full control over its name and behavior.

### Model with Auto Increment

```rust
#[model]
#[table(name = "logs", auto_increment)]
pub struct Log {
    #[primary_key]
    pub id: i32,

    #[field(create, update)]
    pub message: String,

    #[field(create, update)]
    pub level: String,
}
```

> **Note**: With `#[table(auto_increment)]`, the primary key field will be automatically incremented by PostgreSQL. The primary key must be an integer type (`i32`, `i64`).

## Creating Model Instances

### Using Model::new() Method

The `#[model]` macro automatically generates a `new()` method that initializes system fields for you:

```rust
// The macro generates this for the User model:
impl User {
    pub fn new(
        id: Uuid,
        name: String,
        email: String,
        phone: Option<String>,
        age: Option<i32>
    ) -> Self {
        Self {
            id,
            name,
            email,
            phone,
            age,
            \_\_created_at\_\_: Utc::now(),
            \_\_updated_at\_\_: Utc::now(),
            \_\_tags\_\_: Vec::new(),
        }
    }
}

// Usage
let user = User::new(
    Uuid::new_v4(),
    "John Doe".to_string(),
    "john@example.com".to_string(),
    Some("+1-555-0123".to_string()),
    Some(30),
);
```

### For Auto Soft Delete Models

Models with `auto_soft_delete` get the `__is_active__` field initialized automatically to `true`:

```rust
// For the Product model with auto_soft_delete:
let product = Product::new(
    Uuid::new_v4(),
    "Laptop".to_string(),
    99999, // $999.99 in cents
    Some("Gaming laptop".to_string()),
);
// \_\_is_active\_\_ is automatically set to true
```

### For Custom Soft Delete Models

Models with `#[soft_delete]` field must include it in the constructor:

```rust
// For the User model with custom soft delete:
let user = User::new(
    Uuid::new_v4(),
    "John Doe".to_string(),
    "john@example.com".to_string(),
    true, // __is_active__
);
```

### For Auto Increment Models

Models with `auto_increment` don't include the primary key in the constructor:

```rust
// For the Log model with auto_increment:
let log = Log::new(
    "Application started successfully".to_string(),
    "INFO".to_string(),
);
// id is automatically generated by PostgreSQL
```

### Manual Instantiation (Not Recommended)

You can still create instances manually, but you must provide all system fields:

```rust
let user = User {
    id: Uuid::new_v4(),
    name: "John Doe".to_string(),
    email: "john@example.com".to_string(),
    phone: Some("+1-555-0123".to_string()),
    age: Some(30),
    \_\_created_at\_\_: Utc::now(),
    \_\_updated_at\_\_: Utc::now(),
    \_\_tags\_\_: Vec::new(),
};
```

**Note:** Always prefer `Model::new()` as it ensures system fields are properly initialized.

## Attributes Reference

### Model Attributes

#### `#[model]`
Basic model macro for database table generation and CRUD operations.

### Table Attributes

#### `#[table(name = "table_name")]`
Specifies the database table name for the model.

#### `#[table(name = "table_name", auto_soft_delete)]`
Enables automatic soft delete functionality. Adds an `__is_active__` field automatically.

```rust
#[table(name = "products", auto_soft_delete)]
```

#### `#[table(name = "table_name", auto_increment)]`
Enables auto-increment for the primary key field. Primary key must be an integer type.

```rust
#[table(name = "logs", auto_increment)]
```

You can combine multiple table options:

```rust
#[table(name = "comments", auto_soft_delete, auto_increment)]
```

### Field Attributes

#### `#[primary_key]`
Marks the field as the primary key. Required for every model.

```rust
#[primary_key]
pub id: Uuid,
```

#### `#[field(create, update)]`
Field can be set during create and update operations.

```rust
#[field(create, update)]
pub name: String,
```

#### `#[field(create)]`
Field can only be set during create operation (readonly after creation).

```rust
#[field(create)]
pub created_by: Uuid,
```

#### `#[field(update)]`
Field can only be updated, not set during creation.

```rust
#[field(update)]
pub last_modified_by: Option<Uuid>,
```

#### `#[soft_delete]`
Marks a boolean field as the soft delete flag. Gives you full control over the soft delete field name.

```rust
#[soft_delete]
pub __is_active__: bool,
```

#### `#[field(readonly)]`
Field is readonly and managed by the system (deprecated - use system fields instead).

```rust
#[field(readonly)]
pub computed_value: String,
```

## Supported Field Types

### Basic Types

```rust
pub name: String,              // VARCHAR
pub age: i32,                  // INTEGER
pub height: f64,               // DOUBLE PRECISION
pub enabled: bool,             // BOOLEAN
```

### Optional Types

```rust
pub middle_name: Option<String>,    // Nullable VARCHAR
pub score: Option<i32>,             // Nullable INTEGER
```

### UUID Types

```rust
use uuid::Uuid;

pub id: Uuid,                  // UUID primary key
pub parent_id: Option<Uuid>,   // Nullable UUID foreign key
```

### DateTime Types

```rust
use chrono::{DateTime, Utc};

pub birth_date: DateTime<Utc>, // TIMESTAMP WITH TIME ZONE
pub \_\_updated_at\_\_: DateTime<Utc>, // Automatic timestamp
```

### JSON Types

```rust
use serde_json::Value;

pub metadata: Value,           // JSONB
pub config: Option<Value>,     // Nullable JSONB
```

## System Fields

System fields are automatically managed by StoreHaus and should **not** be included in your model definition:

- `\_\_created_at\_\_` - Automatically set on creation
- `\_\_updated_at\_\_` - Automatically updated on modification
- `\_\_is_active\_\_` - Added when using `#[model(soft)]`, or custom field name when using `#[soft_delete]`
- `\_\_tags\_\_` - Array of operation tags

These fields are automatically added to your database table and are available in the generated SQL, but they don't appear in your Rust struct definition.

## Complete Example

```rust
use table_derive::model;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde_json::Value;

#[model(soft)]
#[table(name = "blog_posts")]
pub struct BlogPost {
    #[primary_key]
    pub id: Uuid,

    #[field(create, update)]
    pub title: String,

    #[field(create, update)]
    pub content: String,

    #[field(create, update)]
    pub author_id: Uuid,

    #[field(create)]
    pub slug: String,

    #[field(update)]
    pub published_at: Option<DateTime<Utc>>,

    #[field(create, update)]
    pub metadata: Option<Value>,

    #[field(create, update)]
    pub tags_array: Vec<String>,
}
```

This will generate a table with:
- Your defined fields: `id`, `title`, `content`, `author_id`, `slug`, `published_at`, `metadata`, `tags_array`
- System fields: `\_\_created_at\_\_`, `\_\_updated_at\_\_`, `\_\_is_active\_\_`, `\_\_tags\_\_`

## Generated Code

The `#[model]` macro automatically generates:

1. **TableMetadata trait implementation** - Provides table name, field information, and SQL generation
2. **Database derives** - `Debug`, `Clone`, `Serialize`, `Deserialize`, `FromRow`, `Type`
3. **System field management** - SQL for automatic timestamp and soft delete fields
4. **CRUD methods** - Create, update, and delete SQL generation

## Best Practices

### Naming Conventions

- Use `snake_case` for field names
- Use `PascalCase` for struct names
- Use descriptive table names in `#[table(name = "...")]`

### Field Organization

```rust
#[model(soft)]
#[table(name = "users")]
pub struct User {
    // Primary key first
    #[primary_key]
    pub id: Uuid,

    // Required fields
    #[field(create, update)]
    pub name: String,

    #[field(create, update)]
    pub email: String,

    // Optional fields
    #[field(create, update)]
    pub bio: Option<String>,

    // Create-only fields
    #[field(create)]
    pub created_by: Uuid,

    // Update-only fields
    #[field(update)]
    pub last_login: Option<DateTime<Utc>>,

    // Complex types last
    #[field(create, update)]
    pub preferences: Option<Value>,
}
```

### Error Handling

Common compilation errors and solutions:

#### Missing primary key
```
error: Model must have exactly one #[primary_key] field
```
Solution: Add `#[primary_key]` to exactly one field.

#### Invalid field attribute
```
error: Invalid field attribute combination
```
Solution: Use valid combinations like `create`, `update`, `create, update`, or `readonly`.

#### Unsupported type
```
error: Type doesn't implement required traits
```
Solution: Ensure your type implements `Debug`, `Clone`, `Serialize`, `Deserialize`, and `sqlx` traits.