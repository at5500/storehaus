//! Query builder utilities
//!
//! This module provides SQL query construction utilities.

/// Pagination configuration
#[derive(Debug, Clone)]
pub struct Pagination {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

impl Default for Pagination {
    fn default() -> Self {
        Self::new()
    }
}

impl Pagination {
    pub fn new() -> Self {
        Self {
            limit: None,
            offset: None,
        }
    }

    pub fn with_limit(mut self, limit: i64) -> Self {
        self.limit = Some(limit);
        self
    }

    pub fn with_offset(mut self, offset: i64) -> Self {
        self.offset = Some(offset);
        self
    }

    pub fn to_sql(&self) -> String {
        let mut clauses = Vec::new();

        if let Some(limit) = self.limit {
            clauses.push(format!("LIMIT {}", limit));
        }

        if let Some(offset) = self.offset {
            clauses.push(format!("OFFSET {}", offset));
        }

        clauses.join(" ")
    }
}
