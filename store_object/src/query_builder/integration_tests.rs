//! Query builder utilities
//!
//! This module provides SQL query construction utilities.

#[cfg(test)]
mod integration_tests {
    use crate::query_builder::{QueryBuilder, QueryFilter, QueryOperator, SortOrder};
    use serde_json::{json, Value};
    use std::time::Instant;

    // ========================================
    // Performance Edge Cases
    // ========================================

    #[test]
    fn test_performance_many_conditions() {
        let start = Instant::now();

        let mut builder = QueryBuilder::new();

        // Add 1000 conditions
        for i in 0..1000 {
            builder = builder.filter(QueryFilter::eq(&format!("field_{}", i), json!(i)));
        }

        let (where_clause, _, _, values) = builder.build();
        let duration = start.elapsed();

        // Should complete reasonably fast (< 100ms)
        assert!(duration.as_millis() < 100);
        assert!(!where_clause.is_empty());
        assert_eq!(values.len(), 1000);
    }

    #[test]
    fn test_performance_deep_nesting() {
        let start = Instant::now();

        // Create deeply nested query (50 levels)
        let mut filter = QueryFilter::eq("base", json!("value"));

        for i in 0..50 {
            filter = QueryFilter::and(vec![
                filter,
                QueryFilter::eq(&format!("level_{}", i), json!(i)),
            ]);
        }

        let builder = QueryBuilder::new().filter(filter);
        let (where_clause, _, _, values) = builder.build();
        let duration = start.elapsed();

        // Should complete reasonably fast
        assert!(duration.as_millis() < 50);
        assert!(!where_clause.is_empty());
        assert_eq!(values.len(), 51); // base + 50 levels
    }

    #[test]
    fn test_performance_large_in_clause() {
        let start = Instant::now();

        // Create large IN clause with 1000 values
        let values: Vec<Value> = (0..1000).map(|i| json!(i)).collect();
        let filter = QueryFilter::in_values("id", values);

        let builder = QueryBuilder::new().filter(filter);
        let (where_clause, _, _, params) = builder.build();
        let duration = start.elapsed();

        // Should complete reasonably fast
        assert!(duration.as_millis() < 50);
        assert!(where_clause.contains("IN"));
        assert_eq!(params.len(), 1000);
    }

    // ========================================
    // Memory Usage Edge Cases
    // ========================================

    #[test]
    fn test_memory_usage_large_strings() {
        // Test with very large string values
        let large_string = "x".repeat(100_000); // 100KB string

        let filter = QueryFilter::eq("data", json!(large_string));
        let builder = QueryBuilder::new().filter(filter);

        let (_, _, _, values) = builder.build();

        assert_eq!(values.len(), 1);
        assert_eq!(values[0].as_str().unwrap().len(), 100_000);
    }

    #[test]
    fn test_memory_usage_many_large_values() {
        // Test with many large values
        let values: Vec<Value> = (0..100)
            .map(|i| json!(format!("large_value_{}_", i) + &"x".repeat(1000)))
            .collect();

        let filter = QueryFilter::in_values("text_field", values);
        let builder = QueryBuilder::new().filter(filter);

        let (_, _, _, params) = builder.build();
        assert_eq!(params.len(), 100);
    }

    // ========================================
    // Real-World Scenario Tests
    // ========================================

    #[test]
    fn test_ecommerce_product_search_query() {
        // Simulate complex e-commerce product search
        let builder = QueryBuilder::new()
            // Category and subcategory filter
            .filter(QueryFilter::or(vec![
                QueryFilter::eq("category", json!("electronics")),
                QueryFilter::eq("category", json!("computers")),
            ]))
            // Price range
            .filter(QueryFilter::and(vec![
                QueryFilter::gte("price", json!(50.0)),
                QueryFilter::lte("price", json!(500.0)),
            ]))
            // In stock and not discontinued
            .filter(QueryFilter::gt("stock_quantity", json!(0)))
            .filter(QueryFilter::ne("status", json!("discontinued")))
            // Brand filter
            .filter(QueryFilter::in_values(
                "brand",
                vec![json!("Apple"), json!("Samsung"), json!("Sony")],
            ))
            // Search in title and description
            .filter(QueryFilter::or(vec![
                QueryFilter::ilike("title", "%laptop%"),
                QueryFilter::ilike("description", "%laptop%"),
            ]))
            // Tags for features
            .filter_by_any_tag(vec!["fast-shipping".to_string(), "bestseller".to_string()])
            // Sorting by relevance and price
            .order_by("relevance_score", SortOrder::Desc)
            .order_by("price", SortOrder::Asc)
            // Pagination
            .limit(20)
            .offset(40);

        let (where_clause, order_clause, limit_clause, values) = builder.build();

        // Verify all parts are present and correct
        assert!(where_clause.contains("WHERE"));
        assert!(where_clause.contains("category"));
        assert!(where_clause.contains("price"));
        assert!(where_clause.contains("stock_quantity"));
        assert!(where_clause.contains("__tags__"));
        // Verify values are properly parameterized
        assert!(values.iter().any(|v| v.as_str() == Some("electronics")));
        assert!(values.iter().any(|v| v.as_str() == Some("Apple")));

        assert!(order_clause.contains("ORDER BY"));
        assert!(order_clause.contains("relevance_score DESC"));
        assert!(order_clause.contains("price ASC"));

        assert_eq!(limit_clause, "LIMIT 20 OFFSET 40");
        assert!(!values.is_empty());
    }

    #[test]
    fn test_user_analytics_query() {
        // Simulate complex user analytics query
        let builder = QueryBuilder::new()
            // Active users from last 30 days
            .filter(QueryFilter::gte("last_login", json!("2024-01-01")))
            .filter(QueryFilter::ne("status", json!("banned")))
            // Age demographics
            .filter(QueryFilter::and(vec![
                QueryFilter::gte("age", json!(18)),
                QueryFilter::lte("age", json!(65)),
            ]))
            // Geographic filters
            .filter(QueryFilter::in_values(
                "country",
                vec![json!("US"), json!("CA"), json!("UK"), json!("DE")],
            ))
            // Engagement metrics
            .filter(QueryFilter::gt("login_count", json!(5)))
            .filter(QueryFilter::is_not_null("profile_completed_at"))
            // User types
            .filter(QueryFilter::or(vec![
                QueryFilter::eq("subscription_type", json!("premium")),
                QueryFilter::and(vec![
                    QueryFilter::eq("subscription_type", json!("free")),
                    QueryFilter::gt("usage_score", json!(50)),
                ]),
            ]))
            // Tags for behavior
            .filter_by_all_tags(vec!["verified".to_string(), "active".to_string()])
            // Order by engagement
            .order_by("last_login", SortOrder::Desc)
            .order_by("usage_score", SortOrder::Desc)
            // Large result set
            .limit(1000)
            .offset(0);

        let (where_clause, order_clause, limit_clause, values) = builder.build();

        assert!(where_clause.contains("last_login"));
        assert!(where_clause.contains("country"));
        assert!(where_clause.contains("subscription_type"));
        assert!(where_clause.contains("__tags__"));
        assert!(order_clause.contains("last_login DESC"));
        assert_eq!(limit_clause, "LIMIT 1000 OFFSET 0");
        assert!(!values.is_empty());
    }

    #[test]
    fn test_audit_log_query() {
        // Simulate complex audit log query
        let builder = QueryBuilder::new()
            // Time range
            .filter(QueryFilter::and(vec![
                QueryFilter::gte("__created_at__", json!("2024-01-01T00:00:00Z")),
                QueryFilter::lt("__created_at__", json!("2024-02-01T00:00:00Z")),
            ]))
            // Event types
            .filter(QueryFilter::in_values(
                "event_type",
                vec![
                    json!("user_login"),
                    json!("user_logout"),
                    json!("data_access"),
                    json!("permission_change"),
                ],
            ))
            // Severity levels
            .filter(QueryFilter::or(vec![
                QueryFilter::eq("severity", json!("HIGH")),
                QueryFilter::and(vec![
                    QueryFilter::eq("severity", json!("MEDIUM")),
                    QueryFilter::is_not_null("user_id"),
                ]),
            ]))
            // IP address patterns
            .filter(QueryFilter::not_in_values(
                "ip_address",
                vec![json!("127.0.0.1"), json!("::1")],
            ))
            // User patterns
            .filter(QueryFilter::is_not_null("user_id"))
            .filter(QueryFilter::ne("user_type", json!("system")))
            // Tags for categorization
            .filter_by_any_tag(vec![
                "security".to_string(),
                "compliance".to_string(),
                "audit".to_string(),
            ])
            // Order chronologically
            .order_by("__created_at__", SortOrder::Desc)
            .order_by("severity", SortOrder::Desc)
            // Reasonable page size
            .limit(100)
            .offset(0);

        let (where_clause, order_clause, limit_clause, values) = builder.build();

        assert!(where_clause.contains("__created_at__"));
        assert!(where_clause.contains("event_type"));
        assert!(where_clause.contains("severity"));
        assert!(where_clause.contains("ip_address"));
        assert!(where_clause.contains("NOT IN"));
        assert!(order_clause.contains("__created_at__ DESC"));
        assert_eq!(limit_clause, "LIMIT 100 OFFSET 0");
        assert!(!values.is_empty());
    }

    // ========================================
    // Error Condition Tests
    // ========================================

    #[test]
    fn test_malformed_json_values() {
        // Test that malformed JSON values don't panic
        let filter = QueryFilter::eq("data", json!({"nested": {"very": {"deep": "value"}}}));
        let builder = QueryBuilder::new().filter(filter);

        let (_, _, _, values) = builder.build();
        assert_eq!(values.len(), 1);
    }

    #[test]
    fn test_circular_reference_prevention() {
        // Test that we don't create infinite loops in complex nested queries
        let base_filter = QueryFilter::eq("base", json!("value"));
        let mut complex_filter = base_filter;

        // Create complex nesting without circular references
        for i in 0..10 {
            complex_filter = QueryFilter::and(vec![
                complex_filter,
                QueryFilter::or(vec![
                    QueryFilter::eq(&format!("or_field_a_{}", i), json!(i)),
                    QueryFilter::eq(&format!("or_field_b_{}", i), json!(i + 100)),
                ]),
            ]);
        }

        let builder = QueryBuilder::new().filter(complex_filter);
        let (where_clause, _, _, values) = builder.build();

        // Should complete without infinite loops
        assert!(!where_clause.is_empty());
        assert_eq!(values.len(), 21); // 1 base + 20 nested values
    }

    #[test]
    fn test_concurrent_query_building() {
        use std::sync::Arc;
        use std::thread;

        // Test that query building is thread-safe
        let base_builder = Arc::new(
            QueryBuilder::new()
                .filter(QueryFilter::eq("shared", json!("value")))
                .order_by("__created_at__", SortOrder::Desc),
        );

        let handles: Vec<_> = (0..10)
            .map(|i| {
                let builder = base_builder.clone();
                thread::spawn(move || {
                    let thread_builder = (*builder)
                        .clone()
                        .filter(QueryFilter::eq(&format!("thread_{}", i), json!(i)))
                        .limit(i as i64 + 1);

                    thread_builder.build()
                })
            })
            .collect();

        // All threads should complete successfully
        for handle in handles {
            let result = handle.join().unwrap();
            assert!(!result.0.is_empty()); // where clause
            assert!(!result.1.is_empty()); // order clause
            assert!(!result.2.is_empty()); // limit clause
            assert!(!result.3.is_empty()); // values
        }
    }

    // ========================================
    // Compatibility Tests
    // ========================================

    #[test]
    fn test_postgresql_specific_features() {
        // Test PostgreSQL-specific query patterns
        let builder = QueryBuilder::new()
            // Array operations (PostgreSQL arrays)
            .filter(QueryFilter::condition(
                "tags",
                QueryOperator::Like,
                Some(json!("%{tag1}%")),
            ))
            // JSON operations
            .filter(QueryFilter::condition(
                "metadata->>'type'",
                QueryOperator::Eq,
                Some(json!("premium")),
            ))
            // Text search
            .filter(QueryFilter::condition(
                "search_vector",
                QueryOperator::Like,
                Some(json!("%search_term%")),
            ))
            // Case insensitive search
            .filter(QueryFilter::ilike("description", "%case%insensitive%"))
            // Null handling
            .filter(QueryFilter::is_not_null("deleted_at"));

        let (where_clause, _, _, values) = builder.build();

        assert!(where_clause.contains("LIKE"));
        assert!(where_clause.contains("ILIKE"));
        assert!(where_clause.contains("IS NOT NULL"));
        assert!(!values.is_empty());
    }

    #[test]
    fn test_query_builder_clone_and_modify() {
        // Test that cloning and modifying works correctly
        let base_query = QueryBuilder::new()
            .filter(QueryFilter::eq("status", json!("active")))
            .order_by("__created_at__", SortOrder::Desc);

        // Clone and create variations
        let admin_query = base_query
            .clone()
            .filter(QueryFilter::eq("role", json!("admin")))
            .limit(10);

        let user_query = base_query
            .clone()
            .filter(QueryFilter::eq("role", json!("user")))
            .limit(50);

        let base_result = base_query.build();
        let admin_result = admin_query.build();
        let user_result = user_query.build();

        // Base query should be unchanged
        assert!(!base_result.0.contains("role"));
        assert!(base_result.2.is_empty()); // no limit

        // Admin query should have role filter and limit 10
        assert!(admin_result.0.contains("role"));
        assert!(admin_result.2.contains("LIMIT 10"));
        assert!(admin_result.3.iter().any(|v| v.as_str() == Some("admin")));

        // User query should have role filter and limit 50
        assert!(user_result.0.contains("role"));
        assert!(user_result.2.contains("LIMIT 50"));
        assert!(user_result.3.iter().any(|v| v.as_str() == Some("user")));
    }
}
