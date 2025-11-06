# Debug Logging Configuration

This document explains how to enable debug and trace logging in the StoreHaus project.

## Overview

By default, debug and trace logging is **disabled** to reduce noise and improve performance in production environments. Debug and trace logs are only compiled into the binary when explicitly enabled using the `debug-logging` feature flag.

## How It Works

The project uses conditional compilation with Rust's `cfg` feature flags to include or exclude debug logging code at compile time:

- **Without `debug-logging`**: Debug and trace log statements are completely removed from the compiled binary (zero runtime cost)
- **With `debug-logging`**: Debug and trace logs are compiled in and will appear based on your tracing configuration

## Log Levels

The following log levels are always available:
- `error!` - Always compiled in
- `warn!` - Always compiled in
- `info!` - Always compiled in

The following are conditional (require `debug-logging` feature):
- `debug_log!` - Only when `debug-logging` is enabled
- `trace_log!` - Only when `debug-logging` is enabled

## Enabling Debug Logging

### For Development

To build with debug logging enabled:

```bash
cargo build --features debug-logging
```

To run examples with debug logging:

```bash
cargo run --example demo --features debug-logging
```

To run tests with debug logging:

```bash
cargo test --features debug-logging
```

### For Specific Crates

You can enable debug logging for specific crates:

```bash
cargo build -p storehaus --features debug-logging
cargo build -p signal-system --features debug-logging
```

### For All Workspace Members

To build the entire workspace with debug logging:

```bash
cargo build --workspace --features debug-logging
```

## Setting Log Level at Runtime

Even with `debug-logging` compiled in, you still need to configure the log level at runtime using environment variables:

```bash
# Show all debug logs
RUST_LOG=debug cargo run --example demo --features debug-logging

# Show debug logs only for specific crates
RUST_LOG=storehaus=debug,signal_system=debug cargo run --example demo --features debug-logging

# Show trace logs (most verbose)
RUST_LOG=trace cargo run --example demo --features debug-logging
```

## Example: Database Migration

### Without Debug Logging (Production)

```bash
cargo run --example demo
```

Output will only show `info`, `warn`, and `error` logs:
```
INFO Database connected successfully
INFO User store registered
INFO Created 5 users
```

### With Debug Logging (Development)

```bash
RUST_LOG=debug cargo run --example demo --features debug-logging
```

Output will include detailed debug information:
```
INFO Database connected successfully
DEBUG Creating table with SQL: CREATE TABLE IF NOT EXISTS users (...)
DEBUG Creating index with SQL: CREATE INDEX IF NOT EXISTS idx_users_email ON users(email)
INFO User store registered
DEBUG Executing query: INSERT INTO users ...
INFO Created 5 users
```

## Adding Debug Logs to Your Code

When adding new debug or trace logs, use the conditional macros:

```rust
use storehaus::{debug_log, trace_log};

// This will only be compiled when debug-logging is enabled
debug_log!(table = %table_name, "Creating table");
trace_log!(sql = ?create_sql, "SQL statement details");

// These are always available
info!("Database migration completed");
warn!("Connection pool at 80% capacity");
error!(error = %e, "Failed to execute query");
```

## Performance Considerations

- **Without `debug-logging`**: Zero runtime overhead - debug statements are not even compiled into the binary
- **With `debug-logging`**: Minimal overhead when logs are filtered out by log level
- **Production**: Always build without `debug-logging` for best performance

## CI/CD Configuration

In your CI/CD pipeline, you may want to run tests with debug logging to catch issues:

```yaml
# Example GitHub Actions
- name: Run tests with debug logging
  run: cargo test --workspace --features debug-logging
  env:
    RUST_LOG: debug
```

For production builds, **do not** include the feature flag:

```yaml
- name: Build for production
  run: cargo build --release
```

## Troubleshooting

### "Debug logs not appearing"

1. Make sure you compiled with `--features debug-logging`
2. Check that `RUST_LOG` environment variable is set correctly
3. Verify tracing subscriber is initialized in your binary

### "Too much logging output"

Filter by crate or module:
```bash
# Only debug logs from storehaus
RUST_LOG=storehaus=debug,info

# Only specific module
RUST_LOG=storehaus::migration=debug,info
```

## Summary

| Scenario | Command | Output |
|----------|---------|--------|
| Production | `cargo build --release` | info/warn/error only |
| Development (debug) | `RUST_LOG=debug cargo run --features debug-logging` | + debug logs |
| Development (trace) | `RUST_LOG=trace cargo run --features debug-logging` | + debug + trace logs |
| Testing | `cargo test --features debug-logging` | All test logs |
