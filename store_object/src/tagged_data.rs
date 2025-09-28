//! Tagged Data module
//!
//! This module provides tagged data functionality.

/// Wrapper for data with tags for create/update operations
#[derive(Debug, Clone)]
pub struct TaggedData<T> {
    pub data: T,
    pub tags: Vec<String>,
}

impl<T> TaggedData<T> {
    /// Create new tagged data
    pub fn new(data: T, tags: Vec<String>) -> Self {
        Self { data, tags }
    }

    /// Create tagged data with a single tag
    pub fn with_tag(data: T, tag: String) -> Self {
        Self::new(data, vec![tag])
    }

    /// Create tagged data without tags
    pub fn without_tags(data: T) -> Self {
        Self::new(data, vec![])
    }
}

impl<T> From<T> for TaggedData<T> {
    fn from(data: T) -> Self {
        Self::without_tags(data)
    }
}
