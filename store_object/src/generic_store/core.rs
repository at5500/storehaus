use crate::table_metadata::TableMetadata;
use crate::DbPool;
use std::sync::Arc;
use signal_system::SignalManager;
use cache_system::CacheManager;

/// Generic database store that provides default implementations for all database operations
#[derive(Clone)]
pub struct GenericStore<T: TableMetadata> {
    pub(crate) db_pool: DbPool,
    pub(crate) signal_manager: Option<Arc<signal_system::SignalManager>>,
    pub(crate) cache_manager: Option<Arc<cache_system::CacheManager>>,
    pub(crate) cache_ttl: Option<u64>, // Custom TTL for this store
    pub(crate) cache_prefix: Option<String>, // Custom prefix for cache keys
    pub(crate) _phantom: std::marker::PhantomData<T>,
}

impl<T: TableMetadata> std::fmt::Debug for GenericStore<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GenericStore")
            .field("has_signals", &self.has_signals())
            .field("has_cache", &self.has_cache())
            .field("cache_ttl", &self.cache_ttl)
            .field("cache_prefix", &self.cache_prefix)
            .finish()
    }
}

impl<T: TableMetadata> GenericStore<T> {
    pub fn new(
        db_pool: DbPool,
        signal_manager: Option<Arc<SignalManager>>,
        cache_manager: Option<Arc<CacheManager>>,
        cache_ttl: Option<u64>,
        cache_prefix: Option<String>,
    ) -> Self {
        Self {
            db_pool,
            signal_manager,
            cache_manager,
            cache_ttl,
            cache_prefix,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Set signal manager for this store
    pub fn set_signal_manager(&mut self, signal_manager: Arc<signal_system::SignalManager>) {
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
    pub fn set_cache_manager(&mut self, cache_manager: Arc<cache_system::CacheManager>, ttl: Option<u64>, prefix: Option<String>) {
        self.cache_manager = Some(cache_manager);
        self.cache_ttl = ttl;
        self.cache_prefix = prefix;
    }

    /// Remove cache manager from this store
    pub fn remove_cache_manager(&mut self) {
        self.cache_manager = None;
        self.cache_ttl = None;
        self.cache_prefix = None;
    }

    /// Check if cache manager is set
    pub fn has_cache(&self) -> bool {
        self.cache_manager.is_some()
    }

    /// Get effective cache TTL (custom or from cache manager config)
    pub(crate) fn get_cache_ttl(&self) -> u64 {
        self.cache_ttl
            .or_else(|| self.cache_manager.as_ref().map(|cm| cm.config().default_ttl))
            .unwrap_or(3600) // fallback to 1 hour
    }

    /// Get effective cache prefix (custom or table name)
    pub(crate) fn get_cache_prefix(&self) -> String {
        self.cache_prefix
            .clone()
            .unwrap_or_else(|| T::table_name().to_string())
    }

    pub(crate) fn emit_signal(&self, event: signal_system::DatabaseEvent) {
        if let Some(signal_manager) = &self.signal_manager {
            signal_manager.emit(event);
        }
    }
}