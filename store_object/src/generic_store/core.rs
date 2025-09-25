use crate::table_metadata::TableMetadata;
use crate::DbPool;
use std::sync::Arc;
use signal_system::SignalManager;
use cache_system::CacheManager;

/// Cache parameters for a store
#[derive(Debug, Clone)]
pub struct CacheParams {
    pub manager: Arc<CacheManager>,
    pub ttl: Option<u64>,      // Custom TTL for this store (None = use manager default)
    pub prefix: Option<String>, // Custom prefix for cache keys (None = use table name)
}

impl CacheParams {
    /// Create new cache parameters with manager
    pub fn new(manager: Arc<CacheManager>) -> Self {
        Self {
            manager,
            ttl: None,
            prefix: None,
        }
    }

    /// Set custom TTL for this store
    pub fn with_ttl(mut self, ttl: u64) -> Self {
        self.ttl = Some(ttl);
        self
    }

    /// Set custom prefix for cache keys
    pub fn with_prefix(mut self, prefix: String) -> Self {
        self.prefix = Some(prefix);
        self
    }
}

/// Generic database store that provides default implementations for all database operations
#[derive(Clone)]
pub struct GenericStore<T: TableMetadata> {
    pub(crate) db_pool: DbPool,
    pub(crate) signal_manager: Option<Arc<signal_system::SignalManager>>,
    pub(crate) cache_params: Option<CacheParams>,
    pub(crate) _phantom: std::marker::PhantomData<T>,
}

impl<T: TableMetadata> std::fmt::Debug for GenericStore<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GenericStore")
            .field("has_signals", &self.has_signals())
            .field("has_cache", &self.has_cache())
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

    /// Set cache parameters for this store
    pub fn set_cache(&mut self, cache_params: CacheParams) {
        self.cache_params = Some(cache_params);
    }

    /// Remove cache from this store
    pub fn remove_cache(&mut self) {
        self.cache_params = None;
    }

    /// Check if cache is set
    pub fn has_cache(&self) -> bool {
        self.cache_params.is_some()
    }

    /// Get effective cache TTL (custom or from cache manager config)
    pub(crate) fn get_cache_ttl(&self) -> u64 {
        self.cache_params
            .as_ref()
            .and_then(|cp| cp.ttl)
            .or_else(|| self.cache_params.as_ref().map(|cp| cp.manager.config().default_ttl))
            .unwrap_or(3600) // fallback to 1 hour
    }

    /// Get effective cache prefix (custom or table name)
    pub(crate) fn get_cache_prefix(&self) -> String {
        self.cache_params
            .as_ref()
            .and_then(|cp| cp.prefix.clone())
            .unwrap_or_else(|| T::table_name().to_string())
    }

    /// Get cache manager reference
    pub(crate) fn cache_manager(&self) -> Option<&Arc<CacheManager>> {
        self.cache_params.as_ref().map(|cp| &cp.manager)
    }

    pub(crate) fn emit_signal(&self, event: signal_system::DatabaseEvent) {
        if let Some(signal_manager) = &self.signal_manager {
            signal_manager.emit(event);
        }
    }
}