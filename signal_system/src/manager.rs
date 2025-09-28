//! Improved signal manager with proper resource management and cleanup

use crate::event::DatabaseEvent;
use crate::types::{EventCallback, EventProcessingError};
use std::collections::HashMap;
use std::future::Future;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::timeout;
use tracing::{error, info, warn};

/// Unique identifier for callbacks
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CallbackId(u64);

/// Handle for managing callback lifecycle
/// IMPORTANT: You must call unsubscribe() to properly remove the callback
pub struct CallbackHandle {
    id: CallbackId,
    manager: Arc<SignalManager>,
}

impl CallbackHandle {
    /// Remove this callback from the manager
    /// This must be called explicitly for proper cleanup
    pub async fn unsubscribe(self) {
        self.manager.remove_callback(self.id).await;
    }

    /// Get the callback ID
    pub fn id(&self) -> CallbackId {
        self.id
    }
}

// Re-export centralized config
pub use config::SignalConfig;

/// Callback metadata for resource management
struct CallbackMeta {
    callback: EventCallback,
    consecutive_failures: u32,
    total_executions: u64,
    total_failures: u64,
    created_at: std::time::Instant,
}

/// Improved signal manager with resource cleanup
pub struct SignalManager {
    // Callbacks stored with IDs for removal
    callbacks: Arc<RwLock<HashMap<CallbackId, CallbackMeta>>>,
    // ID generator
    next_id: AtomicU64,
    // Configuration
    config: SignalConfig,
    // Error handler
    error_handler: Arc<RwLock<Option<Box<dyn Fn(EventProcessingError) + Send + Sync>>>>,
    // Cleanup task handle
    cleanup_handle: Arc<RwLock<Option<tokio::task::JoinHandle<()>>>>,
}

impl std::fmt::Debug for SignalManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SignalManager")
            .field("callback_count", &"<async>")
            .field("config", &self.config)
            .finish()
    }
}

impl SignalManager {
    pub fn new(config: SignalConfig) -> Arc<Self> {
        let manager = Arc::new(Self {
            callbacks: Arc::new(RwLock::new(HashMap::new())),
            next_id: AtomicU64::new(0),
            config,
            error_handler: Arc::new(RwLock::new(None)),
            cleanup_handle: Arc::new(RwLock::new(None)),
        });

        // Start cleanup task
        let cleanup_manager = Arc::clone(&manager);
        let cleanup_task = tokio::spawn(async move {
            cleanup_manager.cleanup_loop().await;
        });

        // Store cleanup handle
        let manager_clone = Arc::clone(&manager);
        tokio::spawn(async move {
            let mut handle = manager_clone.cleanup_handle.write().await;
            *handle = Some(cleanup_task);
        });

        manager
    }

    /// Add callback and return a handle for lifecycle management
    /// Returns error if max_callbacks limit is reached (when configured)
    pub async fn add_callback<F, Fut>(
        self: &Arc<Self>,
        callback: F,
    ) -> Result<CallbackHandle, String>
    where
        F: Fn(DatabaseEvent) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = anyhow::Result<()>> + Send + 'static,
    {
        let mut callbacks = self.callbacks.write().await;

        // Check callback limit only if configured
        if callbacks.len() >= self.config.max_callbacks {
            return Err(format!(
                "Maximum callback limit ({}) reached. Remove unused callbacks or increase the limit.",
                self.config.max_callbacks
            ));
        }

        let id = CallbackId(self.next_id.fetch_add(1, Ordering::SeqCst));

        let event_callback: EventCallback = Arc::new(move |event| Box::pin(callback(event)));

        callbacks.insert(
            id,
            CallbackMeta {
                callback: event_callback,
                consecutive_failures: 0,
                total_executions: 0,
                total_failures: 0,
                created_at: std::time::Instant::now(),
            },
        );

        Ok(CallbackHandle {
            id,
            manager: Arc::clone(self),
        })
    }

    /// Remove a specific callback
    pub async fn remove_callback(&self, id: CallbackId) -> bool {
        let mut callbacks = self.callbacks.write().await;
        callbacks.remove(&id).is_some()
    }

    /// Emit event with timeout and resource management
    pub async fn emit(&self, event: DatabaseEvent) {
        // First, collect all callbacks and prepare them for execution
        let callback_futures = {
            let mut callbacks = self.callbacks.write().await;
            let mut futures = Vec::new();

            for (id, meta) in callbacks.iter_mut() {
                meta.total_executions += 1;
                let future = (meta.callback)(event.clone());
                let timeout_future = timeout(
                    Duration::from_secs(self.config.callback_timeout_seconds),
                    future,
                );
                futures.push((*id, timeout_future));
            }

            futures
        };

        // Execute all callbacks concurrently (lock is released)
        let mut results = Vec::new();
        for (id, timeout_future) in callback_futures {
            let result = timeout_future.await;
            results.push((id, result));
        }

        // Process results and update metadata
        let mut callbacks = self.callbacks.write().await;
        let mut ids_to_remove = Vec::new();

        for (id, result) in results {
            if let Some(meta) = callbacks.get_mut(&id) {
                match result {
                    Ok(Ok(())) => {
                        // Success - reset consecutive failures
                        meta.consecutive_failures = 0;
                    }
                    Ok(Err(error)) => {
                        // Callback returned error
                        meta.total_failures += 1;
                        meta.consecutive_failures += 1;

                        self.handle_callback_error(EventProcessingError {
                            callback_index: id.0 as usize,
                            error,
                        })
                        .await;

                        // Check if should remove
                        if self.config.remove_failing_callbacks
                            && meta.consecutive_failures >= self.config.max_consecutive_failures
                        {
                            ids_to_remove.push(id);
                            warn!(
                                callback_id = id.0,
                                consecutive_failures = meta.consecutive_failures,
                                "Removing callback after consecutive failures"
                            );
                        }
                    }
                    Err(_elapsed) => {
                        // Timeout occurred
                        meta.total_failures += 1;
                        meta.consecutive_failures += 1;

                        let error = anyhow::anyhow!(
                            "Callback {} timed out after {}s",
                            id.0,
                            self.config.callback_timeout_seconds
                        );

                        self.handle_callback_error(EventProcessingError {
                            callback_index: id.0 as usize,
                            error,
                        })
                        .await;

                        // Check if should remove
                        if self.config.remove_failing_callbacks
                            && meta.consecutive_failures >= self.config.max_consecutive_failures
                        {
                            ids_to_remove.push(id);
                            warn!(
                                callback_id = id.0,
                                consecutive_failures = meta.consecutive_failures,
                                "Removing callback after timeouts"
                            );
                        }
                    }
                }
            }
        }

        // Remove failing callbacks
        for id in ids_to_remove {
            callbacks.remove(&id);
        }
    }

    /// Set error handler for failed callbacks
    pub async fn set_error_handler<F>(&self, handler: F)
    where
        F: Fn(EventProcessingError) + Send + Sync + 'static,
    {
        let mut error_handler = self.error_handler.write().await;
        *error_handler = Some(Box::new(handler));
    }

    /// Handle callback errors
    async fn handle_callback_error(&self, error: EventProcessingError) {
        let error_handler = self.error_handler.read().await;
        if let Some(handler) = error_handler.as_ref() {
            handler(error);
        } else {
            error!(
                callback_index = error.callback_index,
                error = %error.error,
                "Signal callback error"
            );
        }
    }

    /// Cleanup loop to remove old or unused callbacks
    async fn cleanup_loop(self: Arc<Self>) {
        let mut interval =
            tokio::time::interval(Duration::from_secs(self.config.cleanup_interval_seconds));

        loop {
            interval.tick().await;

            let mut callbacks = self.callbacks.write().await;
            let now = std::time::Instant::now();
            let mut ids_to_remove = Vec::new();

            for (id, meta) in callbacks.iter() {
                // Remove callbacks older than 24 hours with no recent activity
                if now.duration_since(meta.created_at) > Duration::from_secs(86400) && meta.total_executions == 0 {
                    ids_to_remove.push(*id);
                    info!(
                        callback_id = id.0,
                        age_hours = 24,
                        "Removing inactive callback (never executed)"
                    );
                }
            }

            for id in ids_to_remove {
                callbacks.remove(&id);
            }
        }
    }

    /// Get statistics about callbacks
    pub async fn get_stats(&self) -> SignalStats {
        let callbacks = self.callbacks.read().await;

        let total_callbacks = callbacks.len();
        let total_executions: u64 = callbacks.values().map(|m| m.total_executions).sum();
        let total_failures: u64 = callbacks.values().map(|m| m.total_failures).sum();
        let failing_callbacks = callbacks
            .values()
            .filter(|m| m.consecutive_failures > 0)
            .count();

        SignalStats {
            total_callbacks,
            total_executions,
            total_failures,
            failing_callbacks,
            callback_limit: self.config.max_callbacks,
        }
    }

    /// Clear all callbacks
    pub async fn clear_callbacks(&self) {
        let mut callbacks = self.callbacks.write().await;
        callbacks.clear();
    }

    /// Get number of registered callbacks
    pub async fn callback_count(&self) -> usize {
        let callbacks = self.callbacks.read().await;
        callbacks.len()
    }

    /// Shutdown and cleanup all resources
    pub async fn shutdown(&self) {
        // Clear all callbacks
        self.clear_callbacks().await;

        // Cancel cleanup task
        let mut handle = self.cleanup_handle.write().await;
        if let Some(task) = handle.take() {
            task.abort();
        }
    }
}

/// Statistics about signal processing
#[derive(Debug, Clone)]
pub struct SignalStats {
    pub total_callbacks: usize,
    pub total_executions: u64,
    pub total_failures: u64,
    pub failing_callbacks: usize,
    pub callback_limit: usize,
}

impl SignalStats {
    /// Check if we're near the callback limit
    pub fn is_near_limit(&self) -> bool {
        self.total_callbacks as f64 / self.callback_limit as f64 > 0.9
    }
}
