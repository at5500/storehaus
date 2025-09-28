//! Generic store implementations
//!
//! This module provides generic database store functionality.

use crate::table_metadata::TableMetadata;
use crate::DbPool;
use cache_system::{CacheManager, CacheParams};
use signal_system::SignalManager;
use std::sync::Arc;

/// Generic database store that provides default implementations for all database operations
#[derive(Clone)]
pub struct GenericStore<T: TableMetadata> {
    pub(crate) db_pool: DbPool,
    pub(crate) signal_manager: Option<Arc<SignalManager>>,
    pub(crate) cache_params: Option<CacheParams>,
    pub(crate) _phantom: std::marker::PhantomData<T>,
}

impl<T: TableMetadata> std::fmt::Debug for GenericStore<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GenericStore")
            .field("has_signals", &self.has_signals())
            .field("has_cache_manager", &self.has_cache_manager())
            .field("cache_params", &self.cache_params)
            .finish()
    }
}

impl<T: TableMetadata> GenericStore<T> {
    pub fn new(
        db_pool: DbPool,
        signal_manager: Option<Arc<SignalManager>>,
        cache_params: Option<CacheParams>,
    ) -> Self {
        Self {
            db_pool,
            signal_manager,
            cache_params,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Set signal manager for this store
    pub fn set_signal_manager(&mut self, signal_manager: Arc<SignalManager>) {
        self.signal_manager = Some(signal_manager);
    }

    /// Remove signal manager from this store
    pub fn remove_signal_manager(&mut self) {
        self.signal_manager = None;
    }

    /// Check if signal manager is set
    pub fn has_signals(&self) -> bool {
        self.signal_manager.is_some()
    }

    /// Set cache manager for this store
    pub fn set_cache_manager(&mut self, cache_params: CacheParams) {
        self.cache_params = Some(cache_params);
    }

    /// Remove cache manager from this store
    pub fn remove_cache_manager(&mut self) {
        self.cache_params = None;
    }

    /// Check if cache manager is set
    pub fn has_cache_manager(&self) -> bool {
        self.cache_params.is_some()
    }

    /// Get effective cache TTL
    pub(crate) fn get_cache_ttl(&self) -> u64 {
        self.cache_params.as_ref().map(|cp| cp.ttl).unwrap_or(3600) // fallback to 1 hour (will use CacheParams default)
    }

    /// Get effective cache prefix
    pub(crate) fn get_cache_prefix(&self) -> &str {
        self.cache_params
            .as_ref()
            .map(|cp| cp.prefix.as_str())
            .unwrap_or_else(|| T::table_name())
    }

    /// Get cache manager reference
    pub(crate) fn cache_manager(&self) -> Option<&Arc<CacheManager>> {
        self.cache_params.as_ref().map(|cp| &cp.manager)
    }

    pub(crate) async fn emit_signal(&self, event: signal_system::DatabaseEvent) {
        if let Some(signal_manager) = &self.signal_manager {
            signal_manager.emit(event).await;
        }
    }

    /// Create event with explicit tags
    pub(crate) fn create_event_with_explicit_tags(
        &self,
        event_type: signal_system::EventType,
        table_name: String,
        record: &T,
        record_id: Option<String>,
        tags: Vec<String>,
    ) -> signal_system::DatabaseEvent {
        let mut event = signal_system::DatabaseEvent::new(event_type, table_name);

        if let Some(id) = record_id {
            event = event.with_record_id(id);
        }

        // Add record data as typed PostgresValue payload
        let payload = signal_system::serialize_to_postgres_payload(record);
        for (key, value) in payload {
            event.add_payload(key, value);
        }

        // Add explicit tags to event
        if !tags.is_empty() {
            event.add_tags(tags);
            // Also add tags to payload for completeness
            event.add_payload(
                "__tags__".to_string(),
                signal_system::PostgresValue::Json(serde_json::Value::Array(
                    event
                        .tags
                        .iter()
                        .map(|t| serde_json::Value::String(t.clone()))
                        .collect(),
                )),
            );
        }

        event
    }

    /// Extract tags from payload and create event with tags (legacy method)
    pub(crate) fn create_event_with_tags(
        &self,
        event_type: signal_system::EventType,
        table_name: String,
        record: &T,
        record_id: Option<String>,
    ) -> signal_system::DatabaseEvent {
        // For backward compatibility - no tags
        self.create_event_with_explicit_tags(event_type, table_name, record, record_id, Vec::new())
    }
}
