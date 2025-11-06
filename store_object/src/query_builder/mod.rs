//! Query builder utilities
//!
//! This module provides SQL query construction utilities.

pub mod aggregation;
pub mod builder;
pub mod filter;
pub mod grouping;
pub mod join;
pub mod ordering;
pub mod pagination;
pub mod sql_generation;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod integration_tests;

// Re-export main types for backward compatibility
pub use aggregation::{AggregateFunction, SelectField};
pub use builder::QueryBuilder;
pub use filter::{QueryFilter, QueryOperator};
pub use grouping::GroupBy;
pub use join::{JoinClause, JoinCondition, JoinType};
pub use ordering::SortOrder;
