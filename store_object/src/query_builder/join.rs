/// Represents the type of SQL JOIN operation
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JoinType {
    /// INNER JOIN - returns records that have matching values in both tables
    Inner,
    /// LEFT JOIN - returns all records from the left table and matched records from the right table
    Left,
    /// RIGHT JOIN - returns all records from the right table and matched records from the left table
    Right,
    /// FULL OUTER JOIN - returns all records when there is a match in either left or right table
    Full,
    /// CROSS JOIN - returns Cartesian product of both tables
    Cross,
}

impl JoinType {
    /// Convert JoinType to SQL string
    pub fn to_sql(&self) -> &'static str {
        match self {
            JoinType::Inner => "INNER JOIN",
            JoinType::Left => "LEFT JOIN",
            JoinType::Right => "RIGHT JOIN",
            JoinType::Full => "FULL OUTER JOIN",
            JoinType::Cross => "CROSS JOIN",
        }
    }
}

/// Represents a condition for joining tables
#[derive(Debug, Clone, PartialEq)]
pub enum JoinCondition {
    /// Join on a condition (e.g., ON table1.id = table2.user_id)
    On {
        left_field: String,
        right_field: String,
    },
    /// Join using common column names (e.g., USING (id, name))
    Using(Vec<String>),
}

/// Represents a complete JOIN clause
#[derive(Debug, Clone, PartialEq)]
pub struct JoinClause {
    /// Type of join (INNER, LEFT, etc.)
    pub join_type: JoinType,
    /// Table to join with
    pub table: String,
    /// Optional table alias
    pub alias: Option<String>,
    /// Join condition (ON or USING)
    pub condition: JoinCondition,
}

impl JoinClause {
    /// Create a new JOIN clause with ON condition
    pub fn new_on(
        join_type: JoinType,
        table: impl Into<String>,
        left_field: impl Into<String>,
        right_field: impl Into<String>,
    ) -> Self {
        Self {
            join_type,
            table: table.into(),
            alias: None,
            condition: JoinCondition::On {
                left_field: left_field.into(),
                right_field: right_field.into(),
            },
        }
    }

    /// Create a new JOIN clause with USING condition
    pub fn new_using(
        join_type: JoinType,
        table: impl Into<String>,
        columns: Vec<String>,
    ) -> Self {
        Self {
            join_type,
            table: table.into(),
            alias: None,
            condition: JoinCondition::Using(columns),
        }
    }

    /// Add an alias for the joined table
    pub fn with_alias(mut self, alias: impl Into<String>) -> Self {
        self.alias = Some(alias.into());
        self
    }

    /// Get the table reference (alias if present, otherwise table name)
    pub fn table_ref(&self) -> &str {
        self.alias.as_ref().unwrap_or(&self.table)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_join_type_to_sql() {
        assert_eq!(JoinType::Inner.to_sql(), "INNER JOIN");
        assert_eq!(JoinType::Left.to_sql(), "LEFT JOIN");
        assert_eq!(JoinType::Right.to_sql(), "RIGHT JOIN");
        assert_eq!(JoinType::Full.to_sql(), "FULL OUTER JOIN");
        assert_eq!(JoinType::Cross.to_sql(), "CROSS JOIN");
    }

    #[test]
    fn test_join_clause_new_on() {
        let join = JoinClause::new_on(
            JoinType::Inner,
            "orders",
            "users.id",
            "orders.user_id",
        );

        assert_eq!(join.join_type, JoinType::Inner);
        assert_eq!(join.table, "orders");
        assert_eq!(join.alias, None);
        assert_eq!(
            join.condition,
            JoinCondition::On {
                left_field: "users.id".to_string(),
                right_field: "orders.user_id".to_string(),
            }
        );
    }

    #[test]
    fn test_join_clause_new_using() {
        let join = JoinClause::new_using(
            JoinType::Left,
            "profiles",
            vec!["user_id".to_string()],
        );

        assert_eq!(join.join_type, JoinType::Left);
        assert_eq!(join.table, "profiles");
        assert_eq!(
            join.condition,
            JoinCondition::Using(vec!["user_id".to_string()])
        );
    }

    #[test]
    fn test_join_clause_with_alias() {
        let join = JoinClause::new_on(
            JoinType::Left,
            "orders",
            "users.id",
            "o.user_id",
        )
        .with_alias("o");

        assert_eq!(join.alias, Some("o".to_string()));
        assert_eq!(join.table_ref(), "o");
    }

    #[test]
    fn test_join_clause_table_ref_without_alias() {
        let join = JoinClause::new_on(
            JoinType::Inner,
            "orders",
            "users.id",
            "orders.user_id",
        );

        assert_eq!(join.table_ref(), "orders");
    }
}
