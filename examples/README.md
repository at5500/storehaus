# StoreHaus Examples

This directory contains examples demonstrating various features of the StoreHaus library, organized by complexity and topic.

## 🚀 Getting Started Examples

### [01_basic_usage.rs](./01_basic_usage.rs)
**Basic CRUD operations with models**
- Creating simple models with `#[model]` macro
- Using the new `Model::new()` method for clean instantiation
- Basic create, read, update, delete operations
- Working with StoreHaus coordination

```bash
cargo run --example 01_basic_usage 
```

### [02_model_definition.rs](./02_model_definition.rs)
**Advanced model definition techniques**
- Field attributes (`#[primary_key]`, `#[field(create, update)]`)
- Different data types and nullable fields
- Model relationships and foreign keys
- Soft delete models with `auto_soft_delete`

```bash
cargo run --example 02_model_definition 
```

## 🏷️ Tags and Organization

### [tags_demo.rs](./tags_demo.rs)
**Advanced tagging system**
- Creating and managing tags
- Tag-based queries and filtering
- Operation categorization and tracking

```bash
cargo run --example tags_demo 
```

## ⚡ Signals and Events

### [signals_basic.rs](./signals_basic.rs)
**Introduction to the signal system**
- Setting up signal callbacks
- Handling database events (Create, Update, Delete)
- Basic error handling in callbacks

```bash
cargo run --example signals_basic 
```

## 🗄️ Caching

### [caching_basic.rs](./caching_basic.rs)
**Introduction to caching**
- Setting up Redis cache
- Basic cache configuration
- Cache hits vs misses
- Cache invalidation

```bash
cargo run --example caching_basic 
```

## 🏪 Real-World Applications

### [ecommerce_demo.rs](./ecommerce_demo.rs)
**Complete e-commerce system**
- Users, products, orders, inventory
- Complex business logic
- Integration of all StoreHaus features
- Real-world performance scenarios

```bash
cargo run --example ecommerce_demo 
```

### [blog_system.rs](./blog_system.rs)
**Blog system with posts and comments**
- Content management
- User permissions
- Comment threading
- Tag-based categorization

```bash
cargo run --example blog_system 
```

## 🎯 Quick Demo

### [demo.rs](./demo.rs)
**Main demonstration script**
- Quick overview of core features
- Good starting point for new users

```bash
cargo run --example demo 
```

## 📋 Requirements

All examples require:
- PostgreSQL database running on `localhost:5432`
- Database named `storehaus`
- User `postgres` with password `password`
- Redis server on `localhost:6379` (for caching examples)

### Quick Setup with Docker

```bash
# Start PostgreSQL and Redis
docker-compose up -d

# Or manually:
docker run -d --name postgres -e POSTGRES_DB=storehaus -e POSTGRES_PASSWORD=password -p 5432:5432 postgres:15
docker run -d --name redis -p 6379:6379 redis:7-alpine
```

## 🎓 Learning Path

**Recommended order for learning:**

1. **Start Here**: `demo.rs` - Overview of the system
2. **Basics**: `01_basic_usage.rs` - Learn CRUD operations
3. **Models**: `02_model_definition.rs` - Advanced model features
4. **Organization**: `tags_demo.rs` - Tagging and categorization
5. **Events**: `signals_basic.rs` - Event handling and callbacks
6. **Performance**: `caching_basic.rs` - Redis caching optimization
7. **Integration**: `ecommerce_demo.rs` - Complete e-commerce system
8. **Real-world**: `blog_system.rs` - Content management application

## 🛠️ Troubleshooting

**Common issues:**

- **Database connection failed**: Ensure PostgreSQL is running and accessible
- **Redis connection failed**: Install and start Redis server
- **Compilation errors**: Run `cargo clean` and rebuild
- **Permission denied**: Check database user permissions

**Getting help:**
- Check the main [README.md](../README.md) for setup instructions
- Review individual example files for detailed comments
- Open an issue if you encounter problems