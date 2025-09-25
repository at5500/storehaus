use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::types::PostgresValue;

/// Database event type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventType {
    Create,
    Update,
    Delete,
}

/// Database event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseEvent {
    /// Unique event ID
    pub id: Uuid,
    /// Event type
    pub event_type: EventType,
    /// Table name
    pub table_name: String,
    /// Record ID (if available)
    pub record_id: Option<String>,
    /// Additional data
    pub payload: HashMap<String, PostgresValue>,
    /// Event timestamp (UTC)
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl DatabaseEvent {
    pub fn new(event_type: EventType, table_name: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            event_type,
            table_name,
            record_id: None,
            payload: HashMap::new(),
            timestamp: chrono::Utc::now(),
        }
    }

    pub fn with_record_id(mut self, record_id: String) -> Self {
        self.record_id = Some(record_id);
        self
    }

    pub fn with_payload(mut self, key: String, value: PostgresValue) -> Self {
        self.payload.insert(key, value);
        self
    }

    pub fn add_payload(&mut self, key: String, value: PostgresValue) {
        self.payload.insert(key, value);
    }
}