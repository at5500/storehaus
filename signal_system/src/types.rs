//! Type definitions for signal system
//!
//! This module contains PostgreSQL value types and serialization
//! utilities for the signal system.

use crate::event::DatabaseEvent;
use futures::future::BoxFuture;
use std::sync::Arc;

// Re-export from type-mapping for convenience
pub use type_mapping::{
    serialize_to_postgres_payload, serialize_to_postgres_record, PostgresValue, ToPostgresPayload,
};

/// Async event callback type that returns a Result
pub type EventCallback =
    Arc<dyn Fn(DatabaseEvent) -> BoxFuture<'static, anyhow::Result<()>> + Send + Sync>;

/// Event processing error
#[derive(Debug)]
pub struct EventProcessingError {
    pub callback_index: usize,
    pub error: anyhow::Error,
}
