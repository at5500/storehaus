use serde::{Serialize, Deserialize};
use uuid::Uuid;

use crate::event::DatabaseEvent;

/// PostgreSQL data types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PostgresValue {
    Text(String),
    Integer(i32),
    BigInt(i64),
    SmallInt(i16),
    Boolean(bool),
    Uuid(Uuid),
    Timestamp(chrono::DateTime<chrono::Utc>),
    Decimal(String), // Store as string to preserve precision
    Json(serde_json::Value),
    Record(std::collections::HashMap<String, PostgresValue>), // Associative array for full records
    Null,
}

/// Event callback type
pub type EventCallback = Box<dyn Fn(&DatabaseEvent) + Send + Sync>;