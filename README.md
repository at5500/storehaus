# = StoreHaus =

A Rust database abstraction library with automatic code generation for PostgreSQL, featuring signals, caching, and advanced query capabilities.

## Architecture

- **`store_object/`** - Core library with database traits and generic store implementation
- **`table_derive/`** - Derive macro for automatic table metadata generation
- **`dispatcher/`** - Database connection manager and store registry
- **`signal_system/`** - Database event notifications and monitoring
- **`cache_system/`** - Redis-based caching layer for performance optimization

## Quick Start

### Prerequisites

- Rust 1.75+
- Docker and Docker Compose
- Make (optional, but recommended)
- Redis (optional, for caching features)

### Setup

1. **Clone and build the project:**
   ```bash
   git clone <repo>
   cd storehaus
   make setup
   ```

2. **Start the database:**
   ```bash
   make docker-up
   ```

3. **Run the demo:**
   ```bash
   make demo
   # Or with Redis cache support:
   make demo-with-cache
   # Or automatic setup + demo:
   make demo-full
   ```

### Manual Setup (without Make)

1. **Start services:**
   ```bash
   # PostgreSQL only
   docker-compose up -d postgres

   # PostgreSQL + Redis for caching
   docker-compose up -d postgres redis
   ```

2. **Build the project:**
   ```bash
   cargo build
   ```

3. **Run demo:**
   ```bash
   cd dispatcher
   cargo run --example demo
   ```

## Usage

### Define a Model

```rust
use table_derive::model;
use uuid::Uuid;

#[model]
#[table(name = "users")]
pub struct User {
    #[primary_key]
    pub id: Uuid,

    #[field(create, update)]
    pub name: String,

    #[field(create, update)]
    pub email: String,

    #[field(readonly)]
    pub created_at: chrono::DateTime<chrono::Utc>,

    #[field(readonly)]
    pub updated_at: chrono::DateTime<chrono::Utc>,

    #[soft_delete]
    pub is_active: bool,
}
```

### Use the Dispatcher

```rust
use dispatcher::{DatabaseConfig, Dispatcher};
use store_object::{generic_store::{GenericStore, CacheParams}, QueryBuilder, QueryFilter, SortOrder};
use signal_system::{DatabaseEvent, SignalManager};
use cache_system::{CacheConfig, CacheManager};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configure database
    let config = DatabaseConfig::new(
        "localhost".to_string(),
        5432,
        "storehaus".to_string(),
        "postgres".to_string(),
        "password".to_string(),
    ).with_max_connections(5);

    // Create dispatcher
    let mut dispatcher = Dispatcher::new(config).await?;

    // Setup signals (optional)
    let signal_manager = std::sync::Arc::new(SignalManager::new());
    signal_manager.add_callback(|event: &DatabaseEvent| {
        println!("Database event: {:?} on {}", event.event_type, event.table_name);
    });

    // Setup cache (optional)
    let cache_config = CacheConfig::new(
        "redis://localhost:6379".to_string(),
        1800, // TTL in seconds
        "myapp".to_string(),
    );
    let cache_manager = std::sync::Arc::new(CacheManager::new(cache_config)?);

    // Auto-migrate table
    dispatcher.auto_migrate::<User>(false).await?;

    // Create store with signals and cache
    let cache_params = CacheParams::new(cache_manager.clone())
        .with_ttl(900)                     // Cache TTL (15 min)
        .with_prefix("users".to_string()); // Cache prefix

    let user_store = GenericStore::<User>::new(
        dispatcher.pool().clone(),
        Some(signal_manager.clone()),       // Signals
        Some(cache_params),                 // Cache
    );

    dispatcher.register_store("users".to_string(), user_store)?;
    let user_store = dispatcher.get_store::<GenericStore<User>>("users")?;

    // CRUD Operations
    let user = User { /* ... */ };
    let created = user_store.create(user).await?;  // Signals emitted, cached
    let found = user_store.get_by_id(&created.id).await?; // Cache hit!
    let updated = user_store.update(&created.id, modified_user).await?; // Cache invalidated
    let deleted = user_store.delete(&created.id).await?;

    // Advanced Queries
    let active_users = user_store.find(
        QueryBuilder::new()
            .filter(QueryFilter::eq("is_active", json!(true)))
            .order_by("name", SortOrder::Asc)
            .limit(10)
    ).await?;

    // Batch operations
    let batch_updates = vec![(id1, user1), (id2, user2)];
    let updated_users = user_store.update_many(batch_updates).await?;

    Ok(())
}
```

## Development

### Available Commands

```bash
make help                # Show all available commands

# Database
make docker-up          # Start PostgreSQL
make docker-down        # Stop containers
make pgadmin-up         # Start PgAdmin web UI
make db-connect         # Connect to database via psql
make db-reset           # Reset database (WARNING: destroys data)

# Examples
make demo               # Run demo (PostgreSQL only)
make demo-with-cache    # Run demo with Redis cache
make demo-full          # Auto-start services and run demo

# Development
make build              # Build project
make test               # Run tests
make check              # Run format, lint, and tests
make format             # Format code
make lint               # Run linting

# Convenience
make dev                # Start development environment
make setup              # Initial setup for development
```

### Database Access

- **PostgreSQL**: `postgresql://postgres:password@localhost:5432/storehaus`
- **Redis**: `redis://localhost:6379`
- **PgAdmin**: http://localhost:5050 (admin@storehaus.local / admin)

### Project Structure

```
storehaus/
├── store_object/           # Core traits and generic store
│   ├── src/
│   │   ├── traits/        # StoreObject trait definitions
│   │   ├── generic_store/ # Generic store implementation
│   │   ├── query_builder/ # SQL query builder
│   │   └── table_metadata.rs # TableMetadata trait
│   └── Cargo.toml
├── table_derive/          # Derive macro for TableMetadata
│   ├── src/lib.rs
│   └── Cargo.toml
├── dispatcher/            # Connection manager and registry
│   ├── src/lib.rs
│   ├── examples/demo.rs   # Comprehensive demo
│   └── Cargo.toml
├── signal_system/         # Database event notifications
│   ├── src/lib.rs
│   └── Cargo.toml
├── cache_system/          # Redis caching layer
│   ├── src/lib.rs
│   └── Cargo.toml
├── docker-compose.yml     # PostgreSQL + Redis setup
├── init.sql              # Database initialization
├── Makefile              # Development commands
├── CHANGELOG.md          # Version history
└── README.md
```

## Features

### Current
- ✅ Generic store implementation
- ✅ Derive macro for table metadata
- ✅ Database connection management
- ✅ Store registration and retrieval
- ✅ Full CRUD operations
- ✅ Advanced query builder with filters, sorting, pagination
- ✅ Batch operations (create_many, update_many, delete_many)
- ✅ Signal system for database event monitoring
- ✅ Redis-based caching layer with performance optimization
- ✅ Auto-migration system
- ✅ Docker setup with PostgreSQL + Redis
- ✅ Comprehensive demo with all features

### Planned
- [ ] Relationship support (foreign keys, joins)
- [ ] Connection pooling optimization
- [ ] Multiple database support (MySQL, SQLite)
- [ ] Advanced caching strategies
- [ ] Database transaction management
- [ ] Custom field types and validators

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run tests: `make check`
5. Submit a pull request

## License

MIT License - see LICENSE file for details.