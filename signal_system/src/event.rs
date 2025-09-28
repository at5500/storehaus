//! Database event types and definitions
//!
//! This module defines the structure of database events
//! that flow through the signal system.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
    /// Unique event ID (auto-increment when stored in database)
    pub id: Option<i64>,
    /// Event type
    pub event_type: EventType,
    /// Table name
    pub table_name: String,
    /// Record ID (if available)
    pub record_id: Option<String>,
    /// Tags for grouping related operations
    pub tags: Vec<String>,
    /// Additional data
    pub payload: HashMap<String, PostgresValue>,
    /// Event timestamp (UTC)
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl DatabaseEvent {
    pub fn new(event_type: EventType, table_name: String) -> Self {
        Self {
            id: None, // Will be set when stored in database
            event_type,
            table_name,
            record_id: None,
            tags: Vec::new(),
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

    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.add_tags(tags);
        self
    }

    pub fn add_payload(&mut self, key: String, value: PostgresValue) {
        self.payload.insert(key, value);
    }

    pub fn add_tags(&mut self, tags: Vec<String>) {
        for tag in tags {
            if !self.tags.contains(&tag) {
                self.tags.push(tag);
            }
        }
    }

    /// Get tags as PostgresValue for including in payload
    ///
    /// Converts the tags Vec<String> to PostgresValue::Json containing a JSON array.
    /// This ensures consistent representation across the system and works with
    /// the unified type mapping from the type-mapping crate.
    ///
    /// Example: `["tag1", "tag2"]` -> `PostgresValue::Json(["tag1", "tag2"])`
    pub fn tags_as_postgres_value(&self) -> PostgresValue {
        PostgresValue::Json(serde_json::Value::Array(
            self.tags
                .iter()
                .map(|t| serde_json::Value::String(t.clone()))
                .collect(),
        ))
    }
}
