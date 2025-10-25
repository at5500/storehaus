# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Comprehensive type mapping example (`all_types_demo.rs`) demonstrating all SQLx-compatible types
- Public re-exports of internal crates (`store_object`, `table_derive`, `cache_system`, `signal_system`, `type_mapping`) for macro compatibility
- Documentation about unsigned integer limitations and PostgreSQL type system constraints

### Fixed
- **CRITICAL**: Fixed incorrect PostgreSQL type mapping for `Option<DateTime<Utc>>` and other complex types
  - Previously all fields (except primary key and system fields) were hardcoded as `VARCHAR`
  - Now correctly maps to appropriate PostgreSQL types:
    - `Option<DateTime<Utc>>` → `TIMESTAMP WITH TIME ZONE` (was `VARCHAR`)
    - `Option<i32>` → `INTEGER` (was `VARCHAR`)
    - All other Optional types now correctly mapped
- Type string normalization to handle `quote!()` macro whitespace output
  - Handles types like `Option < DateTime < Utc > >` correctly
- `generate_table_fields()` now uses actual field types from `get_field_types()` instead of fallback

## [Previous - Unreleased]

### Added
- Redis-based caching system with performance optimization
- Signal system for database event monitoring and notifications
- Advanced query builder with filters, sorting, and pagination
- Batch operations (update_many, delete_many) with transaction support
- Auto-migration system for automatic table creation
- Comprehensive demo example showcasing all features
- Docker Compose setup with PostgreSQL and Redis
- Cache performance testing and comparison
- Graceful fallback when Redis is unavailable
- `#[model]` attribute macro for simplified model definitions
- `CacheParams` struct for better cache configuration management
- Modern ASCII architecture diagrams with visual icons and emojis
- Configuration system support in documentation
- Comprehensive type mapping documentation with examples
- Complete `type_mapping` crate for Rust to PostgreSQL type conversion
- Optional type support (Option<T>) for all basic types
- RFC3339 timestamp parsing and automatic conversion
- UUID string parsing in parameter binding
- PostgreSQL array operations with `&&` overlap operator
- JSONB type mapping for `serde_json::Value`
- Comprehensive examples collection with learning path
- Multiple documentation files (caching, signals, configuration, etc.)
- Configuration system with TOML, environment variables, and validation
- Prelude modules for all crates for easier imports
- System fields management (__created_at__, __updated_at__, __tags__)
- Soft delete support with automatic __is_active__ field
- Tagged data operations for categorization and tracking
- Validation system for data integrity
- Error handling improvements with detailed error types

### Changed
- **BREAKING**: Simplified `create` method API - removed unnecessary `CreateResult` type
  - **Before**: `create()` returned `Result<(Model, CreateResult), Error>`
  - **After**: `create()` returns `Result<Model, Error>`
- **BREAKING**: SignalManager architecture refactored for better flexibility
  - SignalManager now created externally like CacheManager
  - Removed enable/disable methods - SignalManager always active when provided
  - Removed SignalManager from StoreHaus - now passed directly to stores
- **BREAKING**: Renamed `Dispatcher` to `StoreHaus` throughout the codebase
  - Updated all struct names, method calls, and documentation
  - Improved brand consistency and API clarity
- Enhanced documentation structure and visual presentation
  - Updated ASCII architecture diagrams with modern icons and layout
  - Improved visual consistency across all documentation files
  - Fixed rectangular borders and alignment in diagrams
  - Added emoji icons for better visual organization
  - Reorganized component responsibilities and descriptions
- Major codebase restructuring and improvements
  - Moved from dispatcher-based to storehaus-centric architecture
  - Reorganized crate structure for better modularity
  - Updated all imports and exports to use prelude modules
  - Improved error messages and debugging information
- Enhanced type system and query capabilities
  - Expanded type mapping to cover all common Rust types
  - Added intelligent parameter binding with automatic type detection
  - Improved query builder with more operators and filters
  - Better handling of complex data types (JSONB, Arrays, UUIDs)
- **BREAKING**: GenericStore constructor now accepts `CacheParams` instead of separate cache parameters
  - **Before**: `GenericStore::new(pool, signals, cache_manager, ttl, prefix)`
  - **After**: `GenericStore::new(pool, signals, cache_params)`
- Consolidated three separate examples into single comprehensive demo
- Updated Makefile commands for better clarity:
  - `make demo` - Run demo with PostgreSQL only
  - `make demo-with-cache` - Run demo with Redis cache support
  - `make demo-full` - Auto-start services and run full demo
- Enhanced README with updated usage examples and feature documentation
- Removed obsolete `version` attribute from docker-compose.yml
- Updated all examples to use `#[model]` macro instead of explicit derives

### Improved
- Better error handling and logging throughout the system
- Optimized cache key generation and TTL management
- Enhanced signal payloads with detailed event information
- Improved demo output with clearer progress indicators and test results

### Fixed
- Removed unimplemented Transaction Support from documentation and diagrams
- Corrected feature descriptions to match actual implementation
- Fixed ragged edges in ASCII architecture diagrams
- Removed all remaining "dispatcher" references from documentation
- Fixed type mapping issues for complex PostgreSQL types
  - JSONB fields now correctly mapped instead of VARCHAR
  - Optional types (Option<T>) properly handled in SQL generation
  - UUID parameters correctly bound in queries
  - RFC3339 timestamps automatically parsed and converted
- Resolved SQL parameter binding errors
  - Fixed "operator does not exist" errors for UUID comparisons
  - Corrected timestamp comparison issues
  - Implemented PostgreSQL array overlap operations
- Fixed query builder SQL generation
  - Resolved parameter spacing issues causing syntax errors
  - Improved WHERE clause construction
  - Better handling of complex filter conditions
- Clippy warnings and code quality improvements
  - Removed unnecessary borrowing and references
  - Fixed collapsible match patterns
  - Eliminated unused imports and variables
  - Improved code documentation and comments
- Duplicate email constraint errors in batch operations
- Demo cleanup issues causing conflicts on repeated runs
- Cache invalidation after update and delete operations
- Misleading documentation about advanced caching strategies
  - Removed references to unimplemented "caching strategies" from examples
  - Updated documentation to accurately reflect TTL-only caching implementation
  - Corrected ASCII diagrams to show "Expire" instead of "LRU"
  - Fixed component descriptions to match actual cache capabilities

### Removed
- Old separate example files (basic_example.rs, signals_usage.rs, cache_usage.rs)
- Unused `CreateResult` associated type from StoreObject trait
- Redundant example commands from Makefile

## [Previous Versions]

### [0.1.0] - Initial Release
- Basic store object implementation
- Table metadata derive macro
- Database connection management
- Simple CRUD operations
- Docker setup with PostgreSQL
- Basic example usage