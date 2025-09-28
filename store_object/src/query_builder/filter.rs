//! Query builder utilities
//!
//! This module provides SQL query construction utilities.

use serde_json::Value;

/// Query condition operators
#[derive(Debug, Clone)]
pub enum QueryOperator {
    Eq,        // =
    Ne,        // !=
    Gt,        // >
    Gte,       // >=
    Lt,        // <
    Lte,       // <=
    Like,      // LIKE
    ILike,     // ILIKE (case insensitive)
    In,        // IN
    NotIn,     // NOT IN
    IsNull,    // IS NULL
    IsNotNull, // IS NOT NULL
    ArrayOverlap, // && (PostgreSQL array overlap)
}

/// Single condition in WHERE clause
#[derive(Debug, Clone)]
pub struct QueryCondition {
    pub field: String,
    pub operator: QueryOperator,
    pub value: Option<Value>, // None for IS NULL/IS NOT NULL
}

/// Logical operators for combining conditions
#[derive(Debug, Clone)]
pub enum LogicalOperator {
    And,
    Or,
}

/// Query filter that can be nested
#[derive(Debug, Clone)]
pub enum QueryFilter {
    Condition(QueryCondition),
    Group {
        operator: LogicalOperator,
        filters: Vec<QueryFilter>,
    },
}

impl QueryFilter {
    /// Create a simple condition
    pub fn condition(field: &str, operator: QueryOperator, value: Option<Value>) -> Self {
        Self::Condition(QueryCondition {
            field: field.to_string(),
            operator,
            value,
        })
    }

    /// Create AND group
    pub fn and(filters: Vec<QueryFilter>) -> Self {
        Self::Group {
            operator: LogicalOperator::And,
            filters,
        }
    }

    /// Create OR group
    pub fn or(filters: Vec<QueryFilter>) -> Self {
        Self::Group {
            operator: LogicalOperator::Or,
            filters,
        }
    }

    /// Equal condition
    pub fn eq(field: &str, value: Value) -> Self {
        Self::condition(field, QueryOperator::Eq, Some(value))
    }

    /// Not equal condition
    pub fn ne(field: &str, value: Value) -> Self {
        Self::condition(field, QueryOperator::Ne, Some(value))
    }

    /// Greater than condition
    pub fn gt(field: &str, value: Value) -> Self {
        Self::condition(field, QueryOperator::Gt, Some(value))
    }

    /// Greater than or equal condition
    pub fn gte(field: &str, value: Value) -> Self {
        Self::condition(field, QueryOperator::Gte, Some(value))
    }

    /// Less than condition
    pub fn lt(field: &str, value: Value) -> Self {
        Self::condition(field, QueryOperator::Lt, Some(value))
    }

    /// Less than or equal condition
    pub fn lte(field: &str, value: Value) -> Self {
        Self::condition(field, QueryOperator::Lte, Some(value))
    }

    /// LIKE condition
    pub fn like(field: &str, pattern: &str) -> Self {
        Self::condition(
            field,
            QueryOperator::Like,
            Some(Value::String(pattern.to_string())),
        )
    }

    /// ILIKE condition (case insensitive)
    pub fn ilike(field: &str, pattern: &str) -> Self {
        Self::condition(
            field,
            QueryOperator::ILike,
            Some(Value::String(pattern.to_string())),
        )
    }

    /// IN condition
    pub fn in_values(field: &str, values: Vec<Value>) -> Self {
        Self::condition(field, QueryOperator::In, Some(Value::Array(values)))
    }

    /// NOT IN condition
    pub fn not_in_values(field: &str, values: Vec<Value>) -> Self {
        Self::condition(field, QueryOperator::NotIn, Some(Value::Array(values)))
    }

    /// IS NULL condition
    pub fn is_null(field: &str) -> Self {
        Self::condition(field, QueryOperator::IsNull, None)
    }

    /// IS NOT NULL condition
    pub fn is_not_null(field: &str) -> Self {
        Self::condition(field, QueryOperator::IsNotNull, None)
    }

    /// Filter by records that have any of the specified tags
    pub fn has_any_tag(tags: Vec<String>) -> Self {
        let tag_values: Vec<Value> = tags.into_iter().map(Value::String).collect();
        Self::condition(
            "__tags__",
            QueryOperator::ArrayOverlap,
            Some(Value::Array(tag_values)),
        )
    }

    /// Filter by records that have all of the specified tags
    pub fn has_all_tags(tags: Vec<String>) -> Self {
        let conditions: Vec<QueryFilter> = tags
            .into_iter()
            .map(|tag| {
                Self::condition(
                    "__tags__",
                    QueryOperator::Like,
                    Some(Value::String(format!("%{}%", tag))),
                )
            })
            .collect();

        Self::and(conditions)
    }

    /// Filter by records that have a specific tag
    pub fn has_tag(tag: String) -> Self {
        Self::has_any_tag(vec![tag])
    }
}
