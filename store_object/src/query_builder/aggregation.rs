/// Represents SQL aggregate functions
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AggregateFunction {
    /// COUNT(*) or COUNT(field)
    Count,
    /// SUM(field)
    Sum,
    /// AVG(field)
    Avg,
    /// MIN(field)
    Min,
    /// MAX(field)
    Max,
    /// COUNT(DISTINCT field)
    CountDistinct,
}

impl AggregateFunction {
    /// Convert aggregate function to SQL string
    pub fn to_sql(&self) -> &'static str {
        match self {
            AggregateFunction::Count => "COUNT",
            AggregateFunction::Sum => "SUM",
            AggregateFunction::Avg => "AVG",
            AggregateFunction::Min => "MIN",
            AggregateFunction::Max => "MAX",
            AggregateFunction::CountDistinct => "COUNT",
        }
    }

    /// Check if this is a DISTINCT aggregate
    pub fn is_distinct(&self) -> bool {
        matches!(self, AggregateFunction::CountDistinct)
    }
}

/// Represents a field selection in a SELECT clause
#[derive(Debug, Clone, PartialEq)]
pub enum SelectField {
    /// Select all fields: SELECT *
    All,
    /// Select specific field: SELECT field_name
    Field(String),
    /// Select field with alias: SELECT field_name AS alias
    FieldWithAlias {
        field: String,
        alias: String,
    },
    /// Select aggregate function: SELECT COUNT(field)
    Aggregate {
        function: AggregateFunction,
        field: Option<String>, // None for COUNT(*)
        alias: Option<String>,
    },
}

impl SelectField {
    /// Create a simple field selection
    pub fn field(field: impl Into<String>) -> Self {
        SelectField::Field(field.into())
    }

    /// Create a field with alias
    pub fn field_as(field: impl Into<String>, alias: impl Into<String>) -> Self {
        SelectField::FieldWithAlias {
            field: field.into(),
            alias: alias.into(),
        }
    }

    /// Create COUNT(*) aggregate
    pub fn count_all() -> Self {
        SelectField::Aggregate {
            function: AggregateFunction::Count,
            field: None,
            alias: None,
        }
    }

    /// Create COUNT(field) aggregate
    pub fn count(field: impl Into<String>) -> Self {
        SelectField::Aggregate {
            function: AggregateFunction::Count,
            field: Some(field.into()),
            alias: None,
        }
    }

    /// Create COUNT(DISTINCT field) aggregate
    pub fn count_distinct(field: impl Into<String>) -> Self {
        SelectField::Aggregate {
            function: AggregateFunction::CountDistinct,
            field: Some(field.into()),
            alias: None,
        }
    }

    /// Create SUM(field) aggregate
    pub fn sum(field: impl Into<String>) -> Self {
        SelectField::Aggregate {
            function: AggregateFunction::Sum,
            field: Some(field.into()),
            alias: None,
        }
    }

    /// Create AVG(field) aggregate
    pub fn avg(field: impl Into<String>) -> Self {
        SelectField::Aggregate {
            function: AggregateFunction::Avg,
            field: Some(field.into()),
            alias: None,
        }
    }

    /// Create MIN(field) aggregate
    pub fn min(field: impl Into<String>) -> Self {
        SelectField::Aggregate {
            function: AggregateFunction::Min,
            field: Some(field.into()),
            alias: None,
        }
    }

    /// Create MAX(field) aggregate
    pub fn max(field: impl Into<String>) -> Self {
        SelectField::Aggregate {
            function: AggregateFunction::Max,
            field: Some(field.into()),
            alias: None,
        }
    }

    /// Add an alias to this select field
    pub fn with_alias(self, alias: impl Into<String>) -> Self {
        match self {
            SelectField::Field(field) => SelectField::FieldWithAlias {
                field,
                alias: alias.into(),
            },
            SelectField::Aggregate {
                function,
                field,
                alias: _,
            } => SelectField::Aggregate {
                function,
                field,
                alias: Some(alias.into()),
            },
            other => other,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aggregate_function_to_sql() {
        assert_eq!(AggregateFunction::Count.to_sql(), "COUNT");
        assert_eq!(AggregateFunction::Sum.to_sql(), "SUM");
        assert_eq!(AggregateFunction::Avg.to_sql(), "AVG");
        assert_eq!(AggregateFunction::Min.to_sql(), "MIN");
        assert_eq!(AggregateFunction::Max.to_sql(), "MAX");
    }

    #[test]
    fn test_aggregate_function_is_distinct() {
        assert!(!AggregateFunction::Count.is_distinct());
        assert!(AggregateFunction::CountDistinct.is_distinct());
        assert!(!AggregateFunction::Sum.is_distinct());
    }

    #[test]
    fn test_select_field_simple() {
        let field = SelectField::field("name");
        assert_eq!(field, SelectField::Field("name".to_string()));
    }

    #[test]
    fn test_select_field_with_alias() {
        let field = SelectField::field_as("user_name", "name");
        assert_eq!(
            field,
            SelectField::FieldWithAlias {
                field: "user_name".to_string(),
                alias: "name".to_string(),
            }
        );
    }

    #[test]
    fn test_select_field_count_all() {
        let field = SelectField::count_all();
        match field {
            SelectField::Aggregate {
                function,
                field,
                alias,
            } => {
                assert_eq!(function, AggregateFunction::Count);
                assert_eq!(field, None);
                assert_eq!(alias, None);
            }
            _ => panic!("Expected Aggregate variant"),
        }
    }

    #[test]
    fn test_select_field_count() {
        let field = SelectField::count("id");
        match field {
            SelectField::Aggregate {
                function,
                field,
                alias,
            } => {
                assert_eq!(function, AggregateFunction::Count);
                assert_eq!(field, Some("id".to_string()));
                assert_eq!(alias, None);
            }
            _ => panic!("Expected Aggregate variant"),
        }
    }

    #[test]
    fn test_select_field_count_distinct() {
        let field = SelectField::count_distinct("user_id");
        match field {
            SelectField::Aggregate {
                function,
                field,
                alias,
            } => {
                assert_eq!(function, AggregateFunction::CountDistinct);
                assert_eq!(field, Some("user_id".to_string()));
            }
            _ => panic!("Expected Aggregate variant"),
        }
    }

    #[test]
    fn test_select_field_aggregates() {
        let sum_field = SelectField::sum("amount");
        let avg_field = SelectField::avg("price");
        let min_field = SelectField::min("age");
        let max_field = SelectField::max("score");

        // Verify all are Aggregate variants
        assert!(matches!(sum_field, SelectField::Aggregate { .. }));
        assert!(matches!(avg_field, SelectField::Aggregate { .. }));
        assert!(matches!(min_field, SelectField::Aggregate { .. }));
        assert!(matches!(max_field, SelectField::Aggregate { .. }));
    }

    #[test]
    fn test_select_field_with_alias_chaining() {
        let field = SelectField::field("user_name").with_alias("name");
        assert_eq!(
            field,
            SelectField::FieldWithAlias {
                field: "user_name".to_string(),
                alias: "name".to_string(),
            }
        );

        let aggregate = SelectField::count("id").with_alias("total");
        match aggregate {
            SelectField::Aggregate {
                function,
                field,
                alias,
            } => {
                assert_eq!(function, AggregateFunction::Count);
                assert_eq!(field, Some("id".to_string()));
                assert_eq!(alias, Some("total".to_string()));
            }
            _ => panic!("Expected Aggregate variant"),
        }
    }
}