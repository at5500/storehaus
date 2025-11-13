use serde_json::Value;
use std::collections::HashMap;

/// Type of update operation to perform on a field
#[derive(Debug, Clone)]
pub enum UpdateOperation {
    /// Set field to a specific value: field = $N
    Set(Value),

    /// Increment field by a value: field = field + $N
    Increment(Value),

    /// Decrement field by a value: field = field - $N
    Decrement(Value),

    /// Multiply field by a value: field = field * $N
    Multiply(Value),

    /// Divide field by a value: field = field / $N
    Divide(Value),
}

impl UpdateOperation {
    /// Generate the SQL expression for this operation
    /// Returns (sql_expression, needs_field_reference)
    /// Example: ("field = field + $1", true) or ("field = $1", false)
    pub fn to_sql(&self, field_name: &str, param_number: usize) -> String {
        match self {
            UpdateOperation::Set(_) => {
                format!("{} = ${}", field_name, param_number)
            }
            UpdateOperation::Increment(_) => {
                format!("{} = {} + ${}", field_name, field_name, param_number)
            }
            UpdateOperation::Decrement(_) => {
                format!("{} = {} - ${}", field_name, field_name, param_number)
            }
            UpdateOperation::Multiply(_) => {
                format!("{} = {} * ${}", field_name, field_name, param_number)
            }
            UpdateOperation::Divide(_) => {
                format!("{} = {} / ${}", field_name, field_name, param_number)
            }
        }
    }

    /// Get the value to bind as a parameter
    pub fn value(&self) -> &Value {
        match self {
            UpdateOperation::Set(v)
            | UpdateOperation::Increment(v)
            | UpdateOperation::Decrement(v)
            | UpdateOperation::Multiply(v)
            | UpdateOperation::Divide(v) => v,
        }
    }
}

/// Container for update operations
#[derive(Debug, Clone, Default)]
pub struct UpdateSet {
    pub operations: HashMap<String, UpdateOperation>,
}

impl UpdateSet {
    pub fn new() -> Self {
        Self {
            operations: HashMap::new(),
        }
    }

    /// Set a field to a specific value
    pub fn set(mut self, field: impl Into<String>, value: Value) -> Self {
        self.operations
            .insert(field.into(), UpdateOperation::Set(value));
        self
    }

    /// Increment a field by a value (atomic: field = field + value)
    pub fn increment(mut self, field: impl Into<String>, value: Value) -> Self {
        self.operations
            .insert(field.into(), UpdateOperation::Increment(value));
        self
    }

    /// Decrement a field by a value (atomic: field = field - value)
    pub fn decrement(mut self, field: impl Into<String>, value: Value) -> Self {
        self.operations
            .insert(field.into(), UpdateOperation::Decrement(value));
        self
    }

    /// Multiply a field by a value (atomic: field = field * value)
    pub fn multiply(mut self, field: impl Into<String>, value: Value) -> Self {
        self.operations
            .insert(field.into(), UpdateOperation::Multiply(value));
        self
    }

    /// Divide a field by a value (atomic: field = field / value)
    pub fn divide(mut self, field: impl Into<String>, value: Value) -> Self {
        self.operations
            .insert(field.into(), UpdateOperation::Divide(value));
        self
    }

    /// Check if there are any operations
    pub fn is_empty(&self) -> bool {
        self.operations.is_empty()
    }

    /// Get number of operations
    pub fn len(&self) -> usize {
        self.operations.len()
    }
}