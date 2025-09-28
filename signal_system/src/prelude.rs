//! Convenience re-exports for common signal-system usage

// Core signal system components
pub use crate::event::{DatabaseEvent, EventType};
pub use crate::manager::{CallbackHandle, CallbackId, SignalConfig, SignalManager, SignalStats};
pub use crate::types::{
    serialize_to_postgres_payload, serialize_to_postgres_record, EventCallback, PostgresValue,
    ToPostgresPayload,
};

// Common external dependencies
pub use async_trait::async_trait;
pub use serde::{Deserialize, Serialize};
pub use serde_json;
pub use tokio;
