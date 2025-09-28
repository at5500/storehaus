//! Id Type module
//!
//! This module provides id type functionality.

use std::fmt::{self, Display, Write};
use uuid::Uuid;

/// Universal ID type that can handle both numeric IDs and UUIDs efficiently
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum UniversalId {
    /// Numeric ID (i32, i64, etc.)
    Numeric(i64),
    /// UUID ID
    Uuid(Uuid),
    /// String-based ID
    String(String),
}

impl UniversalId {
    /// Efficiently convert ID to string without format! allocations
    pub fn to_string_fast(&self) -> String {
        match self {
            UniversalId::Numeric(n) => {
                let mut buffer = String::with_capacity(20);
                let _ = write!(buffer, "{}", n);
                buffer
            }
            UniversalId::Uuid(uuid) => uuid.to_string(),
            UniversalId::String(s) => s.clone(),
        }
    }

    /// Get a pre-sized buffer for string conversion
    pub fn to_string_with_capacity(&self) -> String {
        match self {
            UniversalId::Numeric(_) => {
                let mut buffer = String::with_capacity(20);
                let _ = write!(buffer, "{}", self);
                buffer
            }
            UniversalId::Uuid(uuid) => uuid.to_string(),
            UniversalId::String(s) => s.clone(),
        }
    }
}

impl Display for UniversalId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UniversalId::Numeric(n) => write!(f, "{}", n),
            UniversalId::Uuid(uuid) => write!(f, "{}", uuid),
            UniversalId::String(s) => write!(f, "{}", s),
        }
    }
}

impl From<i32> for UniversalId {
    fn from(id: i32) -> Self {
        UniversalId::Numeric(id as i64)
    }
}

impl From<i64> for UniversalId {
    fn from(id: i64) -> Self {
        UniversalId::Numeric(id)
    }
}

impl From<u32> for UniversalId {
    fn from(id: u32) -> Self {
        UniversalId::Numeric(id as i64)
    }
}

impl From<u64> for UniversalId {
    fn from(id: u64) -> Self {
        // Note: This may truncate for very large u64 values
        UniversalId::Numeric(id as i64)
    }
}

impl From<Uuid> for UniversalId {
    fn from(id: Uuid) -> Self {
        UniversalId::Uuid(id)
    }
}

impl From<String> for UniversalId {
    fn from(id: String) -> Self {
        UniversalId::String(id)
    }
}

impl From<&str> for UniversalId {
    fn from(id: &str) -> Self {
        UniversalId::String(id.to_string())
    }
}

/// Trait for types that can provide a universal ID
pub trait HasUniversalId {
    fn universal_id(&self) -> UniversalId;
}

// Blanket implementations for common numeric types
impl HasUniversalId for i32 {
    fn universal_id(&self) -> UniversalId {
        UniversalId::Numeric(*self as i64)
    }
}

impl HasUniversalId for i64 {
    fn universal_id(&self) -> UniversalId {
        UniversalId::Numeric(*self)
    }
}

impl HasUniversalId for u32 {
    fn universal_id(&self) -> UniversalId {
        UniversalId::Numeric(*self as i64)
    }
}

impl HasUniversalId for Uuid {
    fn universal_id(&self) -> UniversalId {
        UniversalId::Uuid(*self)
    }
}

impl HasUniversalId for String {
    fn universal_id(&self) -> UniversalId {
        UniversalId::String(self.clone())
    }
}

impl HasUniversalId for &str {
    fn universal_id(&self) -> UniversalId {
        UniversalId::String(self.to_string())
    }
}
