use std::sync::atomic::AtomicBool;

use crate::event::DatabaseEvent;
use crate::types::EventCallback;

/// Signal manager for database event notifications
pub struct SignalManager {
    callbacks: std::sync::RwLock<Vec<EventCallback>>,
    enabled: AtomicBool,
}

impl std::fmt::Debug for SignalManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SignalManager")
            .field("callback_count", &self.callback_count())
            .field("enabled", &self.is_enabled())
            .finish()
    }
}

impl SignalManager {
    pub fn new() -> Self {
        Self {
            callbacks: std::sync::RwLock::new(Vec::new()),
            enabled: AtomicBool::new(false),
        }
    }

    /// Enable signal system
    pub fn enable(&self) {
        self.enabled.store(true, std::sync::atomic::Ordering::Relaxed);
    }

    /// Disable signal system
    pub fn disable(&self) {
        self.enabled.store(false, std::sync::atomic::Ordering::Relaxed);
    }

    /// Check if system is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled.load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Add event callback
    pub fn add_callback<F>(&self, callback: F)
    where
        F: Fn(&DatabaseEvent) + Send + Sync + 'static,
    {
        if let Ok(mut callbacks) = self.callbacks.write() {
            callbacks.push(Box::new(callback));
        }
    }

    /// Emit event to all subscribers
    pub fn emit(&self, event: DatabaseEvent) {
        if !self.is_enabled() {
            return;
        }

        if let Ok(callbacks) = self.callbacks.read() {
            for callback in callbacks.iter() {
                callback(&event);
            }
        }
    }

    /// Clear all callbacks
    pub fn clear_callbacks(&self) {
        if let Ok(mut callbacks) = self.callbacks.write() {
            callbacks.clear();
        }
    }

    /// Get number of registered callbacks
    pub fn callback_count(&self) -> usize {
        self.callbacks.read().map(|c| c.len()).unwrap_or(0)
    }
}

impl Default for SignalManager {
    fn default() -> Self {
        Self::new()
    }
}