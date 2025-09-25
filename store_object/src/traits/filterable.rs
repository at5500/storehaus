use async_trait::async_trait;
use crate::StorehausError;
use super::core::StoreObject;

/// Generic filter for database queries
#[derive(Clone, Debug)]
pub struct StoreFilter {
    pub conditions: Vec<(String, String)>, // (field_name, value)
}

impl StoreFilter {
    pub fn new() -> Self {
        Self {
            conditions: Vec::new(),
        }
    }

    pub fn add_condition(mut self, field: &str, value: &str) -> Self {
        self.conditions.push((field.to_string(), value.to_string()));
        self
    }

    pub fn build_where_clause(&self) -> (String, Vec<String>) {
        if self.conditions.is_empty() {
            return ("".to_string(), Vec::new());
        }

        let mut where_clause = " WHERE ".to_string();
        let mut values = Vec::new();

        for (i, (field, value)) in self.conditions.iter().enumerate() {
            if i > 0 {
                where_clause.push_str(" AND ");
            }
            where_clause.push_str(&format!("{} = ${}", field, i + 1));
            values.push(value.clone());
        }

        (where_clause, values)
    }
}

impl Default for StoreFilter {
    fn default() -> Self {
        Self::new()
    }
}

/// Trait for objects that support filtering
#[async_trait]
pub trait Filterable: StoreObject {
    /// List objects filtered by some criteria
    async fn list_by_filter(&self, filter: &StoreFilter) -> Result<Vec<Self::Model>, StorehausError>;
}