//! Query builder utilities
//!
//! This module provides SQL query construction utilities.

#[cfg(test)]
mod tests {
    use crate::query_builder::{QueryBuilder, QueryFilter, QueryOperator, SortOrder};
    use serde_json::json;

    // ========================================
    // QueryFilter Edge Cases
    // ========================================

    #[test]
    fn test_query_filter_empty_values() {
        // Test empty string value
        let filter = QueryFilter::eq("name", json!(""));
        assert!(matches!(filter, QueryFilter::Condition(_)));

        // Test null value handling
        let filter = QueryFilter::is_null("optional_field");
        assert!(matches!(filter, QueryFilter::Condition(_)));

        // Test not null value handling
        let filter = QueryFilter::is_not_null("required_field");
        assert!(matches!(filter, QueryFilter::Condition(_)));
    }

    #[test]
    fn test_query_filter_empty_arrays() {
        // Test empty IN clause
        let filter = QueryFilter::in_values("status", vec![]);
        assert!(matches!(filter, QueryFilter::Condition(_)));

        // Test empty NOT IN clause
        let filter = QueryFilter::not_in_values("type", vec![]);
        assert!(matches!(filter, QueryFilter::Condition(_)));
    }

    #[test]
    fn test_query_filter_special_characters() {
        // Test SQL injection patterns
        let filter = QueryFilter::eq("name", json!("'; DROP TABLE users; --"));
        assert!(matches!(filter, QueryFilter::Condition(_)));

        // Test Unicode characters
        let filter = QueryFilter::like("description", "测试数据");
        assert!(matches!(filter, QueryFilter::Condition(_)));

        // Test control characters
        let filter = QueryFilter::eq("data", json!("test\n\r\t"));
        assert!(matches!(filter, QueryFilter::Condition(_)));
    }

    #[test]
    fn test_query_filter_numeric_edge_cases() {
        // Test very large numbers
        let filter = QueryFilter::gt("amount", json!(i64::MAX));
        assert!(matches!(filter, QueryFilter::Condition(_)));

        // Test very small numbers
        let filter = QueryFilter::lt("balance", json!(i64::MIN));
        assert!(matches!(filter, QueryFilter::Condition(_)));

        // Test floating point edge cases
        let filter = QueryFilter::eq("price", json!(f64::NAN));
        assert!(matches!(filter, QueryFilter::Condition(_)));

        let filter = QueryFilter::eq("discount", json!(f64::INFINITY));
        assert!(matches!(filter, QueryFilter::Condition(_)));
    }

    #[test]
    fn test_query_filter_nested_groups() {
        // Test deeply nested AND/OR groups
        let inner_filter = QueryFilter::or(vec![
            QueryFilter::eq("status", json!("active")),
            QueryFilter::eq("status", json!("pending")),
        ]);

        let outer_filter = QueryFilter::and(vec![
            inner_filter,
            QueryFilter::gt("__created_at__", json!("2024-01-01")),
        ]);

        assert!(matches!(outer_filter, QueryFilter::Group { .. }));
    }

    #[test]
    fn test_query_filter_empty_groups() {
        // Test empty AND group
        let filter = QueryFilter::and(vec![]);
        assert!(matches!(filter, QueryFilter::Group { .. }));

        // Test empty OR group
        let filter = QueryFilter::or(vec![]);
        assert!(matches!(filter, QueryFilter::Group { .. }));
    }

    #[test]
    fn test_tag_filters_edge_cases() {
        // Test empty tag list
        let filter = QueryFilter::has_any_tag(vec![]);
        assert!(matches!(filter, QueryFilter::Condition(_)));

        // Test duplicate tags
        let filter = QueryFilter::has_all_tags(vec![
            "tag1".to_string(),
            "tag1".to_string(),
            "tag2".to_string(),
        ]);
        assert!(matches!(filter, QueryFilter::Group { .. }));

        // Test very long tag names
        let long_tag = "a".repeat(1000);
        let filter = QueryFilter::has_tag(long_tag);
        assert!(matches!(filter, QueryFilter::Condition(_)));
    }

    // ========================================
    // SQL Generation Edge Cases
    // ========================================

    #[test]
    fn test_sql_generation_empty_conditions() {
        use crate::query_builder::sql_generation::SqlGenerator;

        let (where_clause, values) = SqlGenerator::build_where_clause(&[]);
        assert_eq!(where_clause, "");
        assert!(values.is_empty());
    }

    #[test]
    fn test_sql_generation_empty_arrays() {
        use crate::query_builder::sql_generation::SqlGenerator;

        // Test empty IN clause generation
        let filter = QueryFilter::in_values("status", vec![]);
        let (where_clause, values) = SqlGenerator::build_where_clause(&[filter]);

        assert!(where_clause.contains("1=0")); // Empty IN should become false condition
        assert!(values.is_empty());

        // Test empty NOT IN clause generation
        let filter = QueryFilter::not_in_values("type", vec![]);
        let (where_clause, values) = SqlGenerator::build_where_clause(&[filter]);

        assert!(where_clause.contains("1=1")); // Empty NOT IN should become true condition
        assert!(values.is_empty());
    }

    #[test]
    fn test_sql_generation_null_conditions() {
        use crate::query_builder::sql_generation::SqlGenerator;

        // Test IS NULL generation
        let filter = QueryFilter::is_null("deleted_at");
        let (where_clause, values) = SqlGenerator::build_where_clause(&[filter]);

        assert!(where_clause.contains("IS NULL"));
        assert!(values.is_empty());

        // Test IS NOT NULL generation
        let filter = QueryFilter::is_not_null("__created_at__");
        let (where_clause, values) = SqlGenerator::build_where_clause(&[filter]);

        assert!(where_clause.contains("IS NOT NULL"));
        assert!(values.is_empty());
    }

    #[test]
    fn test_sql_generation_invalid_operator_value_combinations() {
        use crate::query_builder::sql_generation::SqlGenerator;

        // Test GT without value (should generate invalid condition)
        let filter = QueryFilter::condition("amount", QueryOperator::Gt, None);
        let (where_clause, values) = SqlGenerator::build_where_clause(&[filter]);

        assert!(where_clause.contains("1=0")); // Invalid condition
        assert!(values.is_empty());

        // Test IN with non-array value
        let filter =
            QueryFilter::condition("status", QueryOperator::In, Some(json!("not_an_array")));
        let (where_clause, values) = SqlGenerator::build_where_clause(&[filter]);

        assert!(where_clause.contains("1=0")); // Invalid condition
        assert!(values.is_empty());
    }

    #[test]
    fn test_sql_generation_complex_nested_groups() {
        use crate::query_builder::sql_generation::SqlGenerator;

        // Test deeply nested groups
        let complex_filter = QueryFilter::and(vec![
            QueryFilter::or(vec![
                QueryFilter::eq("status", json!("active")),
                QueryFilter::eq("status", json!("pending")),
            ]),
            QueryFilter::and(vec![
                QueryFilter::gt("amount", json!(100)),
                QueryFilter::lt("amount", json!(1000)),
            ]),
        ]);

        let (where_clause, values) = SqlGenerator::build_where_clause(&[complex_filter]);

        assert!(where_clause.contains("("));
        assert!(where_clause.contains(")"));
        assert!(where_clause.contains("AND"));
        assert!(where_clause.contains("OR"));
        assert_eq!(values.len(), 4); // 4 parameter values
    }

    #[test]
    fn test_sql_generation_parameter_numbering() {
        use crate::query_builder::sql_generation::SqlGenerator;

        // Test that parameters are numbered correctly
        let filters = vec![
            QueryFilter::eq("name", json!("test1")),
            QueryFilter::eq("email", json!("test2")),
            QueryFilter::gt("age", json!(25)),
        ];

        let (where_clause, values) = SqlGenerator::build_where_clause(&filters);

        assert!(where_clause.contains("$1"));
        assert!(where_clause.contains("$2"));
        assert!(where_clause.contains("$3"));
        assert_eq!(values.len(), 3);
    }

    // ========================================
    // QueryBuilder Edge Cases
    // ========================================

    #[test]
    fn test_query_builder_empty_state() {
        let builder = QueryBuilder::new();

        let (where_clause, order_clause, limit_clause, values) = builder.build();

        assert_eq!(where_clause, "");
        assert_eq!(order_clause, "");
        assert_eq!(limit_clause, "");
        assert!(values.is_empty());
    }

    #[test]
    fn test_query_builder_extreme_limits() {
        let builder = QueryBuilder::new().limit(i64::MAX).offset(i64::MAX);

        let (_, _, limit_clause, _) = builder.build();

        assert!(limit_clause.contains(&format!("LIMIT {}", i64::MAX)));
        assert!(limit_clause.contains(&format!("OFFSET {}", i64::MAX)));
    }

    #[test]
    fn test_query_builder_zero_and_negative_limits() {
        // Test zero limit
        let builder = QueryBuilder::new().limit(0);
        let (_, _, limit_clause, _) = builder.build();
        assert!(limit_clause.contains("LIMIT 0"));

        // Test negative offset (PostgreSQL allows this)
        let builder = QueryBuilder::new().offset(-1);
        let (_, _, limit_clause, _) = builder.build();
        assert!(limit_clause.contains("OFFSET -1"));
    }

    #[test]
    fn test_query_builder_many_order_by_clauses() {
        let mut builder = QueryBuilder::new();

        // Add many order by clauses
        for i in 0..100 {
            builder = builder.order_by(&format!("field_{}", i), SortOrder::Asc);
        }

        let (_, order_clause, _, _) = builder.build();

        assert!(order_clause.contains("ORDER BY"));
        assert_eq!(order_clause.matches(",").count(), 99); // 100 fields = 99 commas
    }

    #[test]
    fn test_query_builder_many_filters() {
        let mut builder = QueryBuilder::new();

        // Add many filters
        for i in 0..100 {
            builder = builder.filter(QueryFilter::eq(&format!("field_{}", i), json!(i)));
        }

        let (where_clause, _, _, values) = builder.build();

        assert!(where_clause.contains("WHERE"));
        assert_eq!(where_clause.matches("AND").count(), 99); // 100 conditions = 99 ANDs
        assert_eq!(values.len(), 100);
    }

    #[test]
    fn test_query_builder_method_chaining_order() {
        // Test that method chaining order doesn't matter
        let builder1 = QueryBuilder::new()
            .filter(QueryFilter::eq("name", json!("test")))
            .order_by("__created_at__", SortOrder::Desc)
            .limit(10)
            .offset(5);

        let builder2 = QueryBuilder::new()
            .limit(10)
            .filter(QueryFilter::eq("name", json!("test")))
            .offset(5)
            .order_by("__created_at__", SortOrder::Desc);

        let result1 = builder1.build();
        let result2 = builder2.build();

        assert_eq!(result1.0, result2.0); // where clause
        assert_eq!(result1.1, result2.1); // order clause
        assert_eq!(result1.2, result2.2); // limit clause
        assert_eq!(result1.3, result2.3); // values
    }

    #[test]
    fn test_query_builder_tag_filters() {
        let builder = QueryBuilder::new()
            .filter_by_any_tag(vec!["tag1".to_string(), "tag2".to_string()])
            .filter_by_all_tags(vec!["required1".to_string(), "required2".to_string()])
            .filter_by_tag("single_tag".to_string());

        let (where_clause, _, _, values) = builder.build();

        assert!(where_clause.contains("__tags__"));
        assert!(!values.is_empty());
    }

    // ========================================
    // SortOrder Edge Cases
    // ========================================

    #[test]
    fn test_sort_order_sql_conversion() {
        use crate::query_builder::ordering::SortOrder;

        assert_eq!(SortOrder::Asc.to_sql(), "ASC");
        assert_eq!(SortOrder::Desc.to_sql(), "DESC");
    }

    #[test]
    fn test_order_clause_generation() {
        use crate::query_builder::sql_generation::SqlGenerator;

        // Test empty order
        let order_clause = SqlGenerator::build_order_clause(&[]);
        assert_eq!(order_clause, "");

        // Test single order
        let orders = vec![("name".to_string(), SortOrder::Asc)];
        let order_clause = SqlGenerator::build_order_clause(&orders);
        assert_eq!(order_clause, "ORDER BY name ASC");

        // Test multiple orders
        let orders = vec![
            ("priority".to_string(), SortOrder::Desc),
            ("__created_at__".to_string(), SortOrder::Asc),
            ("name".to_string(), SortOrder::Asc),
        ];
        let order_clause = SqlGenerator::build_order_clause(&orders);
        assert_eq!(
            order_clause,
            "ORDER BY priority DESC, __created_at__ ASC, name ASC"
        );
    }

    // ========================================
    // Pagination Edge Cases
    // ========================================

    #[test]
    fn test_pagination_edge_cases() {
        use crate::query_builder::pagination::Pagination;

        // Test empty pagination
        let pagination = Pagination::new();
        assert_eq!(pagination.to_sql(), "");

        // Test only limit
        let pagination = Pagination::new().with_limit(10);
        assert_eq!(pagination.to_sql(), "LIMIT 10");

        // Test only offset
        let pagination = Pagination::new().with_offset(5);
        assert_eq!(pagination.to_sql(), "OFFSET 5");

        // Test both limit and offset
        let pagination = Pagination::new().with_limit(10).with_offset(5);
        assert_eq!(pagination.to_sql(), "LIMIT 10 OFFSET 5");
    }

    #[test]
    fn test_limit_clause_generation() {
        use crate::query_builder::sql_generation::SqlGenerator;

        // Test no limits
        let clause = SqlGenerator::build_limit_clause(None, None);
        assert_eq!(clause, "");

        // Test only limit
        let clause = SqlGenerator::build_limit_clause(Some(10), None);
        assert_eq!(clause, "LIMIT 10");

        // Test only offset
        let clause = SqlGenerator::build_limit_clause(None, Some(5));
        assert_eq!(clause, "OFFSET 5");

        // Test both
        let clause = SqlGenerator::build_limit_clause(Some(10), Some(5));
        assert_eq!(clause, "LIMIT 10 OFFSET 5");
    }

    // ========================================
    // Integration Edge Cases
    // ========================================

    #[test]
    fn test_full_query_building_edge_cases() {
        // Test extremely complex query
        let builder = QueryBuilder::new()
            .filter(QueryFilter::and(vec![
                QueryFilter::or(vec![
                    QueryFilter::eq("status", json!("active")),
                    QueryFilter::eq("status", json!("pending")),
                ]),
                QueryFilter::gt("amount", json!(0)),
                QueryFilter::is_not_null("user_id"),
            ]))
            .filter(QueryFilter::not_in_values(
                "category",
                vec![json!("spam"), json!("deleted")],
            ))
            .filter_by_any_tag(vec!["premium".to_string(), "vip".to_string()])
            .order_by("priority", SortOrder::Desc)
            .order_by("__created_at__", SortOrder::Asc)
            .limit(50)
            .offset(100);

        let (where_clause, order_clause, limit_clause, values) = builder.build();

        // Verify all parts are present
        assert!(where_clause.contains("WHERE"));
        assert!(where_clause.contains("AND"));
        assert!(where_clause.contains("OR"));
        assert!(where_clause.contains("IS NOT NULL"));
        assert!(where_clause.contains("NOT IN"));

        assert!(order_clause.contains("ORDER BY"));
        assert!(order_clause.contains("priority DESC"));
        assert!(order_clause.contains("__created_at__ ASC"));

        assert!(limit_clause.contains("LIMIT 50"));
        assert!(limit_clause.contains("OFFSET 100"));

        assert!(!values.is_empty());
    }

    #[test]
    fn test_query_builder_immutability() {
        // Test that QueryBuilder methods return new instances
        let original = QueryBuilder::new();
        let modified = original
            .clone()
            .filter(QueryFilter::eq("test", json!("value")));

        let original_result = original.build();
        let modified_result = modified.build();

        // Original should be unchanged
        assert_eq!(original_result.0, ""); // No where clause

        // Modified should have the filter
        assert!(modified_result.0.contains("WHERE"));
    }

    #[test]
    fn test_special_field_names() {
        // Test reserved SQL keywords as field names
        let builder = QueryBuilder::new()
            .filter(QueryFilter::eq("select", json!("value")))
            .filter(QueryFilter::eq("where", json!("value")))
            .filter(QueryFilter::eq("order", json!("value")))
            .order_by("group", SortOrder::Asc);

        let (where_clause, order_clause, _, _) = builder.build();

        // Should handle reserved keywords
        assert!(where_clause.contains("select"));
        assert!(where_clause.contains("where"));
        assert!(where_clause.contains("order"));
        assert!(order_clause.contains("group"));
    }

    #[test]
    fn test_unicode_and_special_characters_in_fields() {
        // Test Unicode field names
        let builder = QueryBuilder::new()
            .filter(QueryFilter::eq("用户名", json!("测试")))
            .filter(QueryFilter::like("描述", "%内容%"))
            .order_by("创建时间", SortOrder::Desc);

        let (where_clause, order_clause, _, values) = builder.build();

        assert!(where_clause.contains("用户名"));
        assert!(where_clause.contains("描述"));
        assert!(order_clause.contains("创建时间"));
        assert!(!values.is_empty());
    }

    #[test]
    fn test_very_large_values() {
        // Test handling of very large string values
        let large_string = "x".repeat(10000);
        let builder =
            QueryBuilder::new().filter(QueryFilter::eq("large_field", json!(large_string)));

        let (_, _, _, values) = builder.build();
        assert_eq!(values.len(), 1);
        assert_eq!(values[0].as_str().unwrap().len(), 10000);
    }

    // ========================================
    // Default Implementation Tests
    // ========================================

    #[test]
    fn test_query_builder_default() {
        let builder1 = QueryBuilder::new();
        let builder2 = QueryBuilder::default();

        let result1 = builder1.build();
        let result2 = builder2.build();

        assert_eq!(result1.0, result2.0);
        assert_eq!(result1.1, result2.1);
        assert_eq!(result1.2, result2.2);
        assert_eq!(result1.3, result2.3);
    }

    // ========================================
    // JOIN Tests
    // ========================================

    #[test]
    fn test_join_inner() {
        use crate::query_builder::{JoinClause, JoinType, SelectField};

        let builder = QueryBuilder::new()
            .select_fields(vec![
                SelectField::field("users.name"),
                SelectField::field("orders.total"),
            ])
            .join(JoinClause::new_on(
                JoinType::Inner,
                "orders",
                "users.id",
                "orders.user_id",
            ));

        let (select_clause, join_clause, _, _, _, _, _, _, _) = builder.build_full();

        assert_eq!(select_clause, "users.name, orders.total");
        assert_eq!(join_clause, "INNER JOIN orders ON users.id = orders.user_id");
    }

    #[test]
    fn test_join_left() {
        use crate::query_builder::{JoinClause, JoinType};

        let builder = QueryBuilder::new().join(JoinClause::new_on(
            JoinType::Left,
            "profiles",
            "users.id",
            "profiles.user_id",
        ));

        let (_, join_clause, _, _, _, _, _, _, _) = builder.build_full();
        assert_eq!(
            join_clause,
            "LEFT JOIN profiles ON users.id = profiles.user_id"
        );
    }

    #[test]
    fn test_join_with_alias() {
        use crate::query_builder::{JoinClause, JoinType};

        let builder = QueryBuilder::new().join(
            JoinClause::new_on(JoinType::Left, "orders", "users.id", "o.user_id")
                .with_alias("o"),
        );

        let (_, join_clause, _, _, _, _, _, _, _) = builder.build_full();
        assert_eq!(join_clause, "LEFT JOIN orders AS o ON users.id = o.user_id");
    }

    #[test]
    fn test_join_using() {
        use crate::query_builder::{JoinClause, JoinType};

        let builder = QueryBuilder::new().join(JoinClause::new_using(
            JoinType::Inner,
            "profiles",
            vec!["user_id".to_string()],
        ));

        let (_, join_clause, _, _, _, _, _, _, _) = builder.build_full();
        assert_eq!(join_clause, "INNER JOIN profiles USING (user_id)");
    }

    #[test]
    fn test_multiple_joins() {
        use crate::query_builder::{JoinClause, JoinType};

        let builder = QueryBuilder::new()
            .join(JoinClause::new_on(
                JoinType::Inner,
                "orders",
                "users.id",
                "orders.user_id",
            ))
            .join(JoinClause::new_on(
                JoinType::Left,
                "profiles",
                "users.id",
                "profiles.user_id",
            ));

        let (_, join_clause, _, _, _, _, _, _, _) = builder.build_full();
        assert!(join_clause.contains("INNER JOIN orders"));
        assert!(join_clause.contains("LEFT JOIN profiles"));
    }

    // ========================================
    // Aggregation Tests
    // ========================================

    #[test]
    fn test_select_count_all() {
        use crate::query_builder::SelectField;

        let builder = QueryBuilder::new().select(SelectField::count_all().with_alias("total"));

        let (select_clause, _, _, _, _, _, _, _, _) = builder.build_full();
        assert_eq!(select_clause, "COUNT(*) AS total");
    }

    #[test]
    fn test_select_count_field() {
        use crate::query_builder::SelectField;

        let builder = QueryBuilder::new().select(SelectField::count("id").with_alias("count"));

        let (select_clause, _, _, _, _, _, _, _, _) = builder.build_full();
        assert_eq!(select_clause, "COUNT(id) AS count");
    }

    #[test]
    fn test_select_count_distinct() {
        use crate::query_builder::SelectField;

        let builder =
            QueryBuilder::new().select(SelectField::count_distinct("user_id").with_alias("unique"));

        let (select_clause, _, _, _, _, _, _, _, _) = builder.build_full();
        assert_eq!(select_clause, "COUNT(DISTINCT user_id) AS unique");
    }

    #[test]
    fn test_select_aggregates() {
        use crate::query_builder::SelectField;

        let builder = QueryBuilder::new().select_fields(vec![
            SelectField::sum("amount").with_alias("total"),
            SelectField::avg("price").with_alias("average"),
            SelectField::min("created_at").with_alias("first"),
            SelectField::max("updated_at").with_alias("last"),
        ]);

        let (select_clause, _, _, _, _, _, _, _, _) = builder.build_full();
        assert!(select_clause.contains("SUM(amount) AS total"));
        assert!(select_clause.contains("AVG(price) AS average"));
        assert!(select_clause.contains("MIN(created_at) AS first"));
        assert!(select_clause.contains("MAX(updated_at) AS last"));
    }

    #[test]
    fn test_select_field_with_alias() {
        use crate::query_builder::SelectField;

        let builder =
            QueryBuilder::new().select(SelectField::field_as("user_name", "display_name"));

        let (select_clause, _, _, _, _, _, _, _, _) = builder.build_full();
        assert_eq!(select_clause, "user_name AS display_name");
    }

    // ========================================
    // GROUP BY Tests
    // ========================================

    #[test]
    fn test_group_by_single() {
        use crate::query_builder::{GroupBy, SelectField};

        let builder = QueryBuilder::new()
            .select_fields(vec![
                SelectField::field("status"),
                SelectField::count_all().with_alias("count"),
            ])
            .group_by(GroupBy::single("status"));

        let (_, _, _, group_by_clause, _, _, _, _, _) = builder.build_full();
        assert_eq!(group_by_clause, "GROUP BY status");
    }

    #[test]
    fn test_group_by_multiple() {
        use crate::query_builder::GroupBy;

        let builder = QueryBuilder::new().group_by(GroupBy::new(vec![
            "category".to_string(),
            "status".to_string(),
        ]));

        let (_, _, _, group_by_clause, _, _, _, _, _) = builder.build_full();
        assert_eq!(group_by_clause, "GROUP BY category, status");
    }

    // ========================================
    // HAVING Tests
    // ========================================

    #[test]
    fn test_having_single_condition() {
        use crate::query_builder::{GroupBy, SelectField};

        let builder = QueryBuilder::new()
            .select_fields(vec![
                SelectField::field("category"),
                SelectField::count_all().with_alias("count"),
            ])
            .group_by(
                GroupBy::single("category").having(QueryFilter::gt("COUNT(*)", json!(10))),
            );

        let (_, _, _, group_by_clause, having_clause, _, _, _, having_values) =
            builder.build_full();

        assert_eq!(group_by_clause, "GROUP BY category");
        assert_eq!(having_clause, "HAVING COUNT(*) > $1");
        assert_eq!(having_values.len(), 1);
    }

    #[test]
    fn test_having_multiple_conditions() {
        use crate::query_builder::GroupBy;

        let builder = QueryBuilder::new().group_by(
            GroupBy::single("user_id")
                .having(QueryFilter::gt("COUNT(order_id)", json!(5)))
                .having(QueryFilter::gte("SUM(total_amount)", json!(1000.0))),
        );

        let (_, _, _, _, having_clause, _, _, _, having_values) = builder.build_full();

        assert!(having_clause.contains("HAVING"));
        assert!(having_clause.contains("COUNT(order_id) > $1"));
        assert!(having_clause.contains("SUM(total_amount) >= $2"));
        assert_eq!(having_values.len(), 2);
    }

    // ========================================
    // Complex Query Tests
    // ========================================

    #[test]
    fn test_complex_query_with_all_features() {
        use crate::query_builder::{GroupBy, JoinClause, JoinType, SelectField};

        let builder = QueryBuilder::new()
            .select_fields(vec![
                SelectField::field("users.name"),
                SelectField::count("orders.id").with_alias("order_count"),
                SelectField::sum("orders.total_amount").with_alias("total_spent"),
            ])
            .join(JoinClause::new_on(
                JoinType::Inner,
                "orders",
                "users.id",
                "orders.user_id",
            ))
            .filter(QueryFilter::eq("orders.status", json!("completed")))
            .group_by(
                GroupBy::single("users.name")
                    .having(QueryFilter::gt("COUNT(orders.id)", json!(3))),
            )
            .order_by("total_spent", SortOrder::Desc)
            .limit(10);

        let (
            select_clause,
            join_clause,
            where_clause,
            group_by_clause,
            having_clause,
            order_clause,
            limit_clause,
            where_values,
            having_values,
        ) = builder.build_full();

        // Verify all parts
        assert!(select_clause.contains("users.name"));
        assert!(select_clause.contains("COUNT(orders.id)"));
        assert!(select_clause.contains("SUM(orders.total_amount)"));

        assert!(join_clause.contains("INNER JOIN orders"));

        assert!(where_clause.contains("WHERE"));
        assert!(where_clause.contains("orders.status"));

        assert_eq!(group_by_clause, "GROUP BY users.name");

        assert!(having_clause.contains("HAVING"));
        assert!(having_clause.contains("COUNT(orders.id)"));

        assert!(order_clause.contains("ORDER BY"));
        assert!(order_clause.contains("total_spent DESC"));

        assert_eq!(limit_clause, "LIMIT 10");

        assert_eq!(where_values.len(), 1);
        assert_eq!(having_values.len(), 1);
    }

    #[test]
    fn test_backward_compatibility_build_method() {
        // Test that old build() method still works for simple queries
        let builder = QueryBuilder::new()
            .filter(QueryFilter::eq("status", json!("active")))
            .order_by("name", SortOrder::Asc)
            .limit(20);

        let (where_clause, order_clause, limit_clause, values) = builder.build();

        assert!(where_clause.contains("WHERE"));
        assert!(order_clause.contains("ORDER BY"));
        assert!(limit_clause.contains("LIMIT"));
        assert!(!values.is_empty());
    }

    #[test]
    fn test_empty_select_defaults_to_all() {
        use crate::query_builder::sql_generation::SqlGenerator;

        let fields = vec![];
        let select_clause = SqlGenerator::build_select_clause(&fields);

        assert_eq!(select_clause, "*");
    }

    #[test]
    fn test_join_types_to_sql() {
        use crate::query_builder::JoinType;

        assert_eq!(JoinType::Inner.to_sql(), "INNER JOIN");
        assert_eq!(JoinType::Left.to_sql(), "LEFT JOIN");
        assert_eq!(JoinType::Right.to_sql(), "RIGHT JOIN");
        assert_eq!(JoinType::Full.to_sql(), "FULL OUTER JOIN");
        assert_eq!(JoinType::Cross.to_sql(), "CROSS JOIN");
    }
}
