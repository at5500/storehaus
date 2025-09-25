# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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

### Changed
- **BREAKING**: Simplified `create` method API - removed unnecessary `CreateResult` type
  - **Before**: `create()` returned `Result<(Model, CreateResult), Error>`
  - **After**: `create()` returns `Result<Model, Error>`
- Consolidated three separate examples into single comprehensive demo
- Updated Makefile commands for better clarity:
  - `make demo` - Run demo with PostgreSQL only
  - `make demo-with-cache` - Run demo with Redis cache support
  - `make demo-full` - Auto-start services and run full demo
- Enhanced README with updated usage examples and feature documentation
- Removed obsolete `version` attribute from docker-compose.yml

### Improved
- Better error handling and logging throughout the system
- Optimized cache key generation and TTL management
- Enhanced signal payloads with detailed event information
- Improved demo output with clearer progress indicators and test results

### Fixed
- Duplicate email constraint errors in batch operations
- Demo cleanup issues causing conflicts on repeated runs
- Cache invalidation after update and delete operations

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