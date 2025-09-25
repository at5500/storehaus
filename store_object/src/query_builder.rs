use serde_json::Value;

/// Query condition operators
#[derive(Debug, Clone)]
pub enum QueryOperator {
    Eq,      // =
    Ne,      // !=
    Gt,      // >
    Gte,     // >=
    Lt,      // <
    Lte,     // <=
    Like,    // LIKE
    ILike,   // ILIKE (case insensitive)
    In,      // IN
    NotIn,   // NOT IN
    IsNull,  // IS NULL
    IsNotNull, // IS NOT NULL
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

    /// Create an AND group
    pub fn and(filters: Vec<QueryFilter>) -> Self {
        Self::Group {
            operator: LogicalOperator::And,
            filters,
        }
    }

    /// Create an OR group
    pub fn or(filters: Vec<QueryFilter>) -> Self {
        Self::Group {
            operator: LogicalOperator::Or,
            filters,
        }
    }

    /// Convenience methods for common operations
    pub fn eq(field: &str, value: Value) -> Self {
        Self::condition(field, QueryOperator::Eq, Some(value))
    }

    pub fn ne(field: &str, value: Value) -> Self {
        Self::condition(field, QueryOperator::Ne, Some(value))
    }

    pub fn gt(field: &str, value: Value) -> Self {
        Self::condition(field, QueryOperator::Gt, Some(value))
    }

    pub fn gte(field: &str, value: Value) -> Self {
        Self::condition(field, QueryOperator::Gte, Some(value))
    }

    pub fn lt(field: &str, value: Value) -> Self {
        Self::condition(field, QueryOperator::Lt, Some(value))
    }

    pub fn lte(field: &str, value: Value) -> Self {
        Self::condition(field, QueryOperator::Lte, Some(value))
    }

    pub fn like(field: &str, pattern: &str) -> Self {
        Self::condition(field, QueryOperator::Like, Some(Value::String(pattern.to_string())))
    }

    pub fn ilike(field: &str, pattern: &str) -> Self {
        Self::condition(field, QueryOperator::ILike, Some(Value::String(pattern.to_string())))
    }

    pub fn in_values(field: &str, values: Vec<Value>) -> Self {
        Self::condition(field, QueryOperator::In, Some(Value::Array(values)))
    }

    pub fn not_in_values(field: &str, values: Vec<Value>) -> Self {
        Self::condition(field, QueryOperator::NotIn, Some(Value::Array(values)))
    }

    pub fn is_null(field: &str) -> Self {
        Self::condition(field, QueryOperator::IsNull, None)
    }

    pub fn is_not_null(field: &str) -> Self {
        Self::condition(field, QueryOperator::IsNotNull, None)
    }
}

/// Query builder for constructing SQL WHERE clauses
pub struct QueryBuilder {
    conditions: Vec<QueryFilter>,
    order_by: Vec<(String, SortOrder)>,
    limit: Option<i64>,
    offset: Option<i64>,
}

#[derive(Debug, Clone)]
pub enum SortOrder {
    Asc,
    Desc,
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

    /// Build the WHERE clause and parameter values
    pub fn build_where_clause(&self) -> (String, Vec<Value>) {
        if self.conditions.is_empty() {
            return (String::new(), Vec::new());
        }

        let mut params = Vec::new();
        let mut param_counter = 1;

        let conditions_sql = self.conditions
            .iter()
            .map(|filter| self.build_filter_sql(filter, &mut params, &mut param_counter))
            .collect::<Vec<_>>()
            .join(" AND ");

        let where_clause = if conditions_sql.is_empty() {
            String::new()
        } else {
            format!(" WHERE {}", conditions_sql)
        };

        (where_clause, params)
    }

    /// Build ORDER BY clause
    pub fn build_order_clause(&self) -> String {
        if self.order_by.is_empty() {
            return String::new();
        }

        let order_parts: Vec<String> = self.order_by
            .iter()
            .map(|(field, order)| {
                let order_str = match order {
                    SortOrder::Asc => "ASC",
                    SortOrder::Desc => "DESC",
                };
                format!("{} {}", field, order_str)
            })
            .collect();

        format!(" ORDER BY {}", order_parts.join(", "))
    }

    /// Build LIMIT and OFFSET clause
    pub fn build_limit_clause(&self) -> String {
        let mut clause = String::new();

        if let Some(limit) = self.limit {
            clause.push_str(&format!(" LIMIT {}", limit));
        }

        if let Some(offset) = self.offset {
            clause.push_str(&format!(" OFFSET {}", offset));
        }

        clause
    }

    /// Build complete query parts
    pub fn build(&self) -> (String, String, String, Vec<Value>) {
        let (where_clause, params) = self.build_where_clause();
        let order_clause = self.build_order_clause();
        let limit_clause = self.build_limit_clause();

        (where_clause, order_clause, limit_clause, params)
    }

    fn build_filter_sql(&self, filter: &QueryFilter, params: &mut Vec<Value>, param_counter: &mut i32) -> String {
        match filter {
            QueryFilter::Condition(condition) => {
                self.build_condition_sql(condition, params, param_counter)
            }
            QueryFilter::Group { operator, filters } => {
                if filters.is_empty() {
                    return String::new();
                }

                let operator_str = match operator {
                    LogicalOperator::And => " AND ",
                    LogicalOperator::Or => " OR ",
                };

                let conditions: Vec<String> = filters
                    .iter()
                    .map(|f| self.build_filter_sql(f, params, param_counter))
                    .filter(|s| !s.is_empty())
                    .collect();

                if conditions.is_empty() {
                    String::new()
                } else if conditions.len() == 1 {
                    conditions[0].clone()
                } else {
                    format!("({})", conditions.join(operator_str))
                }
            }
        }
    }

    fn build_condition_sql(&self, condition: &QueryCondition, params: &mut Vec<Value>, param_counter: &mut i32) -> String {
        let field = &condition.field;

        match &condition.operator {
            QueryOperator::Eq => {
                if let Some(value) = &condition.value {
                    let placeholder = format!("${}", param_counter);
                    params.push(value.clone());
                    *param_counter += 1;
                    format!("{} = {}", field, placeholder)
                } else {
                    format!("{} IS NULL", field)
                }
            }
            QueryOperator::Ne => {
                if let Some(value) = &condition.value {
                    let placeholder = format!("${}", param_counter);
                    params.push(value.clone());
                    *param_counter += 1;
                    format!("{} != {}", field, placeholder)
                } else {
                    format!("{} IS NOT NULL", field)
                }
            }
            QueryOperator::Gt => {
                if let Some(value) = &condition.value {
                    let placeholder = format!("${}", param_counter);
                    params.push(value.clone());
                    *param_counter += 1;
                    format!("{} > {}", field, placeholder)
                } else {
                    String::new()
                }
            }
            QueryOperator::Gte => {
                if let Some(value) = &condition.value {
                    let placeholder = format!("${}", param_counter);
                    params.push(value.clone());
                    *param_counter += 1;
                    format!("{} >= {}", field, placeholder)
                } else {
                    String::new()
                }
            }
            QueryOperator::Lt => {
                if let Some(value) = &condition.value {
                    let placeholder = format!("${}", param_counter);
                    params.push(value.clone());
                    *param_counter += 1;
                    format!("{} < {}", field, placeholder)
                } else {
                    String::new()
                }
            }
            QueryOperator::Lte => {
                if let Some(value) = &condition.value {
                    let placeholder = format!("${}", param_counter);
                    params.push(value.clone());
                    *param_counter += 1;
                    format!("{} <= {}", field, placeholder)
                } else {
                    String::new()
                }
            }
            QueryOperator::Like => {
                if let Some(value) = &condition.value {
                    let placeholder = format!("${}", param_counter);
                    params.push(value.clone());
                    *param_counter += 1;
                    format!("{} LIKE {}", field, placeholder)
                } else {
                    String::new()
                }
            }
            QueryOperator::ILike => {
                if let Some(value) = &condition.value {
                    let placeholder = format!("${}", param_counter);
                    params.push(value.clone());
                    *param_counter += 1;
                    format!("{} ILIKE {}", field, placeholder)
                } else {
                    String::new()
                }
            }
            QueryOperator::In => {
                if let Some(Value::Array(values)) = &condition.value {
                    if values.is_empty() {
                        return "FALSE".to_string(); // IN () is invalid, use FALSE
                    }
                    let placeholders: Vec<String> = values
                        .iter()
                        .map(|_| {
                            let placeholder = format!("${}", param_counter);
                            *param_counter += 1;
                            placeholder
                        })
                        .collect();
                    params.extend(values.clone());
                    format!("{} IN ({})", field, placeholders.join(", "))
                } else {
                    String::new()
                }
            }
            QueryOperator::NotIn => {
                if let Some(Value::Array(values)) = &condition.value {
                    if values.is_empty() {
                        return "TRUE".to_string(); // NOT IN () should be TRUE
                    }
                    let placeholders: Vec<String> = values
                        .iter()
                        .map(|_| {
                            let placeholder = format!("${}", param_counter);
                            *param_counter += 1;
                            placeholder
                        })
                        .collect();
                    params.extend(values.clone());
                    format!("{} NOT IN ({})", field, placeholders.join(", "))
                } else {
                    String::new()
                }
            }
            QueryOperator::IsNull => format!("{} IS NULL", field),
            QueryOperator::IsNotNull => format!("{} IS NOT NULL", field),
        }
    }
}

impl Default for QueryBuilder {
    fn default() -> Self {
        Self::new()
    }
}