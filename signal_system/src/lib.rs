//! Signal system for database event handling
//!
//! This crate provides async signal handling and event processing
//! for database operations in the Storehaus ecosystem.

pub mod event;
pub mod manager;
pub mod prelude;
pub mod types;

pub use event::{DatabaseEvent, EventType};
pub use manager::{CallbackHandle, CallbackId, SignalConfig, SignalManager, SignalStats};
pub use types::{
    serialize_to_postgres_payload, serialize_to_postgres_record, EventCallback, PostgresValue,
    ToPostgresPayload,
};
