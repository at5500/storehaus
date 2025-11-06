//! Query builder utilities
//!
//! This module provides SQL query construction utilities.

use crate::query_builder::aggregation::SelectField;
use crate::query_builder::filter::{LogicalOperator, QueryCondition, QueryFilter, QueryOperator};
use crate::query_builder::grouping::GroupBy;
use crate::query_builder::join::{JoinClause, JoinCondition};
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

    /// Build SELECT clause from fields
    pub fn build_select_clause(fields: &[SelectField]) -> String {
        if fields.is_empty() {
            return "*".to_string();
        }

        let field_parts: Vec<String> = fields.iter().map(|field| Self::build_select_field(field)).collect();

        field_parts.join(", ")
    }

    fn build_select_field(field: &SelectField) -> String {
        match field {
            SelectField::All => "*".to_string(),
            SelectField::Field(name) => name.clone(),
            SelectField::FieldWithAlias { field, alias } => {
                format!("{} AS {}", field, alias)
            }
            SelectField::Aggregate {
                function,
                field,
                alias,
            } => {
                let func_name = function.to_sql();
                let field_part = if function.is_distinct() {
                    if let Some(f) = field {
                        format!("DISTINCT {}", f)
                    } else {
                        "*".to_string()
                    }
                } else {
                    field.as_deref().unwrap_or("*").to_string()
                };

                let aggregate = format!("{}({})", func_name, field_part);

                if let Some(alias) = alias {
                    format!("{} AS {}", aggregate, alias)
                } else {
                    aggregate
                }
            }
        }
    }

    /// Build JOIN clauses
    pub fn build_join_clause(joins: &[JoinClause]) -> String {
        if joins.is_empty() {
            return "".to_string();
        }

        joins
            .iter()
            .map(|join| {
                let join_type = join.join_type.to_sql();
                let table_part = if let Some(alias) = &join.alias {
                    format!("{} AS {}", join.table, alias)
                } else {
                    join.table.clone()
                };

                let condition_part = match &join.condition {
                    JoinCondition::On {
                        left_field,
                        right_field,
                    } => {
                        format!("ON {} = {}", left_field, right_field)
                    }
                    JoinCondition::Using(columns) => {
                        format!("USING ({})", columns.join(", "))
                    }
                };

                format!("{} {} {}", join_type, table_part, condition_part)
            })
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Build GROUP BY clause
    pub fn build_group_by_clause(group_by: Option<&GroupBy>) -> String {
        match group_by {
            Some(group) if !group.fields.is_empty() => {
                format!("GROUP BY {}", group.fields.join(", "))
            }
            _ => "".to_string(),
        }
    }

    /// Build HAVING clause
    pub fn build_having_clause(group_by: Option<&GroupBy>) -> (String, Vec<Value>) {
        match group_by {
            Some(group) if group.has_having() => {
                if let Some(having_conditions) = &group.having {
                    let mut values = Vec::new();
                    let mut param_counter = 1;

                    let conditions_sql = having_conditions
                        .iter()
                        .map(|condition| {
                            Self::build_condition_sql(condition, &mut values, &mut param_counter)
                        })
                        .collect::<Vec<_>>()
                        .join(" AND ");

                    (format!("HAVING {}", conditions_sql), values)
                } else {
                    ("".to_string(), Vec::new())
                }
            }
            _ => ("".to_string(), Vec::new()),
        }
    }
}
