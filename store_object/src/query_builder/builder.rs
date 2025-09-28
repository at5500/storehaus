//! Query builder utilities
//!
//! This module provides SQL query construction utilities.

use crate::query_builder::filter::QueryFilter;
use crate::query_builder::ordering::SortOrder;
use crate::query_builder::sql_generation::SqlGenerator;
use serde_json::Value;

/// Query builder for constructing complex database queries
#[derive(Debug, Clone)]
pub struct QueryBuilder {
    pub(crate) conditions: Vec<QueryFilter>,
    pub(crate) order_by: Vec<(String, SortOrder)>,
    pub(crate) limit: Option<i64>,
    pub(crate) offset: Option<i64>,
}

impl QueryBuilder {
    pub fn new() -> Self {
        Self {
            conditions: Vec::new(),
            order_by: Vec::new(),
            limit: None,
            offset: None,
        }
    }

    /// Add a filter condition
    pub fn filter(mut self, filter: QueryFilter) -> Self {
        self.conditions.push(filter);
        self
    }

    /// Add multiple filters (combined with AND)
    pub fn filters(mut self, filters: Vec<QueryFilter>) -> Self {
        self.conditions.extend(filters);
        self
    }

    /// Add ordering
    pub fn order_by(mut self, field: &str, order: SortOrder) -> Self {
        self.order_by.push((field.to_string(), order));
        self
    }

    /// Add limit
    pub fn limit(mut self, limit: i64) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Add offset
    pub fn offset(mut self, offset: i64) -> Self {
        self.offset = Some(offset);
        self
    }

    /// Filter by records that have any of the specified tags
    pub fn filter_by_any_tag(self, tags: Vec<String>) -> Self {
        self.filter(QueryFilter::has_any_tag(tags))
    }

    /// Filter by records that have all of the specified tags
    pub fn filter_by_all_tags(self, tags: Vec<String>) -> Self {
        self.filter(QueryFilter::has_all_tags(tags))
    }

    /// Filter by records that have a specific tag
    pub fn filter_by_tag(self, tag: String) -> Self {
        self.filter(QueryFilter::has_tag(tag))
    }

    /// Build WHERE clause
    pub fn build_where_clause(&self) -> (String, Vec<Value>) {
        SqlGenerator::build_where_clause(&self.conditions)
    }

    /// Build ORDER BY clause
    pub fn build_order_clause(&self) -> String {
        SqlGenerator::build_order_clause(&self.order_by)
    }

    /// Build LIMIT/OFFSET clause
    pub fn build_limit_clause(&self) -> String {
        SqlGenerator::build_limit_clause(self.limit, self.offset)
    }

    /// Build complete query parts (WHERE, ORDER BY, LIMIT, Values)
    pub fn build(&self) -> (String, String, String, Vec<Value>) {
        let (where_clause, values) = self.build_where_clause();
        let order_clause = self.build_order_clause();
        let limit_clause = self.build_limit_clause();

        (where_clause, order_clause, limit_clause, values)
    }
}

impl Default for QueryBuilder {
    fn default() -> Self {
        Self::new()
    }
}
