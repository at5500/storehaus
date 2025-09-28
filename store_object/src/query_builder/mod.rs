//! Query builder utilities
//!
//! This module provides SQL query construction utilities.

pub mod builder;
pub mod filter;
pub mod ordering;
pub mod pagination;
pub mod sql_generation;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod integration_tests;

// Re-export main types for backward compatibility
pub use builder::QueryBuilder;
pub use filter::{QueryFilter, QueryOperator};
pub use ordering::SortOrder;
