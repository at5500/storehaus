//! Query builder utilities
//!
//! This module provides SQL query construction utilities.

use crate::query_builder::aggregation::SelectField;
use crate::query_builder::filter::QueryFilter;
use crate::query_builder::grouping::GroupBy;
use crate::query_builder::join::JoinClause;
use crate::query_builder::ordering::SortOrder;
use crate::query_builder::sql_generation::SqlGenerator;
use crate::query_builder::update::UpdateSet;
use serde_json::Value;

/// Query builder for constructing complex database queries
#[derive(Debug, Clone)]
pub struct QueryBuilder {
    pub(crate) select_fields: Vec<SelectField>,
    pub(crate) joins: Vec<JoinClause>,
    pub(crate) conditions: Vec<QueryFilter>,
    pub(crate) group_by: Option<GroupBy>,
    pub(crate) order_by: Vec<(String, SortOrder)>,
    pub(crate) limit: Option<i64>,
    pub(crate) offset: Option<i64>,
    pub(crate) updates: Option<UpdateSet>,
}

impl QueryBuilder {
    pub fn new() -> Self {
        Self {
            select_fields: Vec::new(),
            joins: Vec::new(),
            conditions: Vec::new(),
            group_by: None,
            order_by: Vec::new(),
            limit: None,
            offset: None,
            updates: None,
        }
    }

    /// Add a select field
    pub fn select(mut self, field: SelectField) -> Self {
        self.select_fields.push(field);
        self
    }

    /// Add multiple select fields
    pub fn select_fields(mut self, fields: Vec<SelectField>) -> Self {
        self.select_fields.extend(fields);
        self
    }

    /// Add a JOIN clause
    pub fn join(mut self, join: JoinClause) -> Self {
        self.joins.push(join);
        self
    }

    /// Add multiple JOIN clauses
    pub fn joins(mut self, joins: Vec<JoinClause>) -> Self {
        self.joins.extend(joins);
        self
    }

    /// Set GROUP BY clause
    pub fn group_by(mut self, group_by: GroupBy) -> Self {
        self.group_by = Some(group_by);
        self
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

    /// Set update operations for UPDATE queries
    /// This allows specifying atomic operations like increment/decrement
    ///
    /// Example:
    /// ```ignore
    /// let query = QueryBuilder::new()
    ///     .filter(QueryFilter::eq("wallet_id", json!(id)))
    ///     .update(UpdateSet::new().increment("balance", json!(100)));
    /// ```
    pub fn update(mut self, updates: UpdateSet) -> Self {
        self.updates = Some(updates);
        self
    }

    /// Check if this query has update operations defined
    pub fn has_updates(&self) -> bool {
        self.updates.as_ref().map(|u| !u.is_empty()).unwrap_or(false)
    }

    /// Get the update operations
    pub fn get_updates(&self) -> Option<&UpdateSet> {
        self.updates.as_ref()
    }

    /// Build SELECT clause
    pub fn build_select_clause(&self) -> String {
        SqlGenerator::build_select_clause(&self.select_fields)
    }

    /// Build JOIN clauses
    pub fn build_join_clause(&self) -> String {
        SqlGenerator::build_join_clause(&self.joins)
    }

    /// Build WHERE clause
    pub fn build_where_clause(&self) -> (String, Vec<Value>) {
        SqlGenerator::build_where_clause(&self.conditions)
    }

    /// Build GROUP BY clause
    pub fn build_group_by_clause(&self) -> String {
        SqlGenerator::build_group_by_clause(self.group_by.as_ref())
    }

    /// Build HAVING clause
    pub fn build_having_clause(&self) -> (String, Vec<Value>) {
        SqlGenerator::build_having_clause(self.group_by.as_ref())
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
    /// Returns: (where_clause, order_clause, limit_clause, values)
    ///
    /// Note: This method is kept for backward compatibility with existing code.
    /// For queries with JOINs or aggregations, use `build_full()` instead.
    pub fn build(&self) -> (String, String, String, Vec<Value>) {
        let (where_clause, values) = self.build_where_clause();
        let order_clause = self.build_order_clause();
        let limit_clause = self.build_limit_clause();

        (where_clause, order_clause, limit_clause, values)
    }

    /// Build complete query with all clauses including SELECT, JOIN, GROUP BY, and HAVING
    /// Returns: (select_clause, join_clause, where_clause, group_by_clause, having_clause, order_clause, limit_clause, where_values, having_values)
    pub fn build_full(&self) -> (String, String, String, String, String, String, String, Vec<Value>, Vec<Value>) {
        let select_clause = self.build_select_clause();
        let join_clause = self.build_join_clause();
        let (where_clause, where_values) = self.build_where_clause();
        let group_by_clause = self.build_group_by_clause();
        let (having_clause, having_values) = self.build_having_clause();
        let order_clause = self.build_order_clause();
        let limit_clause = self.build_limit_clause();

        (
            select_clause,
            join_clause,
            where_clause,
            group_by_clause,
            having_clause,
            order_clause,
            limit_clause,
            where_values,
            having_values,
        )
    }
}

impl Default for QueryBuilder {
    fn default() -> Self {
        Self::new()
    }
}
