//! Query builder utilities
//!
//! This module provides SQL query construction utilities.

use crate::query_builder::filter::{LogicalOperator, QueryCondition, QueryFilter, QueryOperator};
use crate::query_builder::ordering::SortOrder;
use serde_json::Value;

pub struct SqlGenerator;

impl SqlGenerator {
    /// Build WHERE clause from conditions
    pub fn build_where_clause(conditions: &[QueryFilter]) -> (String, Vec<Value>) {
        if conditions.is_empty() {
            return ("".to_string(), Vec::new());
        }

        let mut values = Vec::new();
        let mut param_counter = 1;

        let conditions_sql = conditions
            .iter()
            .map(|condition| Self::build_condition_sql(condition, &mut values, &mut param_counter))
            .collect::<Vec<_>>()
            .join(" AND ");

        if conditions_sql.is_empty() {
            ("".to_string(), values)
        } else {
            (format!("WHERE {}", conditions_sql), values)
        }
    }

    fn build_condition_sql(
        filter: &QueryFilter,
        values: &mut Vec<Value>,
        param_counter: &mut i32,
    ) -> String {
        match filter {
            QueryFilter::Condition(condition) => {
                Self::build_single_condition_sql(condition, values, param_counter)
            }
            QueryFilter::Group { operator, filters } => {
                let operator_str = match operator {
                    LogicalOperator::And => " AND ",
                    LogicalOperator::Or => " OR ",
                };

                let group_conditions = filters
                    .iter()
                    .map(|f| Self::build_condition_sql(f, values, param_counter))
                    .collect::<Vec<_>>()
                    .join(operator_str);

                format!("({})", group_conditions)
            }
        }
    }

    fn build_single_condition_sql(
        condition: &QueryCondition,
        values: &mut Vec<Value>,
        param_counter: &mut i32,
    ) -> String {
        let field = &condition.field;

        match &condition.operator {
            QueryOperator::Eq => {
                if let Some(value) = &condition.value {
                    values.push(value.clone());
                    let param = format!("${}", param_counter);
                    *param_counter += 1;
                    format!("{} = {}", field, param)
                } else {
                    format!("{} IS NULL", field)
                }
            }
            QueryOperator::Ne => {
                if let Some(value) = &condition.value {
                    values.push(value.clone());
                    let param = format!("${}", param_counter);
                    *param_counter += 1;
                    format!("{} != {}", field, param)
                } else {
                    format!("{} IS NOT NULL", field)
                }
            }
            QueryOperator::Gt => {
                if let Some(value) = &condition.value {
                    values.push(value.clone());
                    let param = format!("${}", param_counter);
                    *param_counter += 1;
                    format!("{} > {}", field, param)
                } else {
                    "1=0".to_string() // Invalid condition
                }
            }
            QueryOperator::Gte => {
                if let Some(value) = &condition.value {
                    values.push(value.clone());
                    let param = format!("${}", param_counter);
                    *param_counter += 1;
                    format!("{} >= {}", field, param)
                } else {
                    "1=0".to_string()
                }
            }
            QueryOperator::Lt => {
                if let Some(value) = &condition.value {
                    values.push(value.clone());
                    let param = format!("${}", param_counter);
                    *param_counter += 1;
                    format!("{} < {}", field, param)
                } else {
                    "1=0".to_string()
                }
            }
            QueryOperator::Lte => {
                if let Some(value) = &condition.value {
                    values.push(value.clone());
                    let param = format!("${}", param_counter);
                    *param_counter += 1;
                    format!("{} <= {}", field, param)
                } else {
                    "1=0".to_string()
                }
            }
            QueryOperator::Like => {
                if let Some(value) = &condition.value {
                    values.push(value.clone());
                    let param = format!("${}", param_counter);
                    *param_counter += 1;
                    format!("{} LIKE {}", field, param)
                } else {
                    "1=0".to_string()
                }
            }
            QueryOperator::ILike => {
                if let Some(value) = &condition.value {
                    values.push(value.clone());
                    let param = format!("${}", param_counter);
                    *param_counter += 1;
                    format!("{} ILIKE {}", field, param)
                } else {
                    "1=0".to_string()
                }
            }
            QueryOperator::In => {
                if let Some(Value::Array(array_values)) = &condition.value {
                    if array_values.is_empty() {
                        return "1=0".to_string(); // Empty IN clause
                    }

                    let placeholders: Vec<String> = array_values
                        .iter()
                        .map(|_| {
                            let param = format!("${}", param_counter);
                            *param_counter += 1;
                            param
                        })
                        .collect();

                    values.extend(array_values.clone());
                    format!("{} IN ({})", field, placeholders.join(", "))
                } else {
                    "1=0".to_string()
                }
            }
            QueryOperator::NotIn => {
                if let Some(Value::Array(array_values)) = &condition.value {
                    if array_values.is_empty() {
                        return "1=1".to_string(); // Empty NOT IN clause
                    }

                    let placeholders: Vec<String> = array_values
                        .iter()
                        .map(|_| {
                            let param = format!("${}", param_counter);
                            *param_counter += 1;
                            param
                        })
                        .collect();

                    values.extend(array_values.clone());
                    format!("{} NOT IN ({})", field, placeholders.join(", "))
                } else {
                    "1=1".to_string()
                }
            }
            QueryOperator::IsNull => format!("{} IS NULL", field),
            QueryOperator::IsNotNull => format!("{} IS NOT NULL", field),
            QueryOperator::ArrayOverlap => {
                if let Some(Value::Array(array_values)) = &condition.value {
                    if array_values.is_empty() {
                        return "1=0".to_string(); // Empty array overlap
                    }

                    let placeholders: Vec<String> = array_values
                        .iter()
                        .map(|_| {
                            let param = format!("${}", param_counter);
                            *param_counter += 1;
                            param
                        })
                        .collect();

                    values.extend(array_values.clone());
                    format!("{} && ARRAY[{}]", field, placeholders.join(", "))
                } else {
                    "1=0".to_string()
                }
            },
        }
    }

    /// Build ORDER BY clause
    pub fn build_order_clause(order_by: &[(String, SortOrder)]) -> String {
        if order_by.is_empty() {
            return "".to_string();
        }

        let order_items: Vec<String> = order_by
            .iter()
            .map(|(field, order)| format!("{} {}", field, order.to_sql()))
            .collect();

        format!("ORDER BY {}", order_items.join(", "))
    }

    /// Build LIMIT/OFFSET clause
    pub fn build_limit_clause(limit: Option<i64>, offset: Option<i64>) -> String {
        let mut clauses = Vec::new();

        if let Some(limit) = limit {
            clauses.push(format!("LIMIT {}", limit));
        }

        if let Some(offset) = offset {
            clauses.push(format!("OFFSET {}", offset));
        }

        clauses.join(" ")
    }
}
