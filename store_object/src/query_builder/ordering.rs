//! Query builder utilities
//!
//! This module provides SQL query construction utilities.

#[derive(Debug, Clone)]
pub enum SortOrder {
    Asc,
    Desc,
}

impl SortOrder {
    pub fn to_sql(&self) -> &'static str {
        match self {
            SortOrder::Asc => "ASC",
            SortOrder::Desc => "DESC",
        }
    }
}
