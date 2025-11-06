use super::filter::QueryFilter;

/// Represents a GROUP BY clause with optional HAVING conditions
#[derive(Debug, Clone, PartialEq)]
pub struct GroupBy {
    /// Fields to group by
    pub fields: Vec<String>,
    /// Optional HAVING conditions for filtering grouped results
    pub having: Option<Vec<QueryFilter>>,
}

impl GroupBy {
    /// Create a new GROUP BY clause with the specified fields
    pub fn new(fields: Vec<String>) -> Self {
        Self {
            fields,
            having: None,
        }
    }

    /// Create a GROUP BY clause with a single field
    pub fn single(field: impl Into<String>) -> Self {
        Self {
            fields: vec![field.into()],
            having: None,
        }
    }

    /// Add HAVING conditions to filter grouped results
    pub fn with_having(mut self, conditions: Vec<QueryFilter>) -> Self {
        self.having = Some(conditions);
        self
    }

    /// Add a single HAVING condition
    pub fn having(mut self, condition: QueryFilter) -> Self {
        match &mut self.having {
            Some(conditions) => conditions.push(condition),
            None => self.having = Some(vec![condition]),
        }
        self
    }

    /// Check if this GROUP BY has HAVING conditions
    pub fn has_having(&self) -> bool {
        self.having
            .as_ref()
            .map(|h| !h.is_empty())
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_group_by_new() {
        let group_by = GroupBy::new(vec!["category".to_string(), "status".to_string()]);

        assert_eq!(group_by.fields.len(), 2);
        assert_eq!(group_by.fields[0], "category");
        assert_eq!(group_by.fields[1], "status");
        assert_eq!(group_by.having, None);
    }

    #[test]
    fn test_group_by_single() {
        let group_by = GroupBy::single("user_id");

        assert_eq!(group_by.fields.len(), 1);
        assert_eq!(group_by.fields[0], "user_id");
        assert_eq!(group_by.having, None);
    }

    #[test]
    fn test_group_by_with_having() {
        let group_by = GroupBy::single("category").with_having(vec![
            QueryFilter::gt("COUNT(*)", json!(5)),
            QueryFilter::lt("AVG(price)", json!(100)),
        ]);

        assert!(group_by.has_having());
        assert_eq!(group_by.having.as_ref().unwrap().len(), 2);
    }

    #[test]
    fn test_group_by_having_chain() {
        let group_by = GroupBy::single("category")
            .having(QueryFilter::gt("COUNT(*)", json!(5)))
            .having(QueryFilter::lt("AVG(price)", json!(100)));

        assert!(group_by.has_having());
        assert_eq!(group_by.having.as_ref().unwrap().len(), 2);
    }

    #[test]
    fn test_has_having_empty() {
        let group_by = GroupBy::single("category");
        assert!(!group_by.has_having());

        let group_by_with_empty = GroupBy {
            fields: vec!["category".to_string()],
            having: Some(vec![]),
        };
        assert!(!group_by_with_empty.has_having());
    }
}