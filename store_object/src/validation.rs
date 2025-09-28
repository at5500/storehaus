//! Validation module
//!
//! This module provides validation functionality.

use std::fmt;

/// Validation errors for database identifiers
#[derive(Debug, Clone, PartialEq)]
pub enum ValidationError {
    /// Name contains invalid characters (only alphanumeric and underscore allowed)
    InvalidCharacters(String),
    /// Name is too long (PostgreSQL limit is 63 characters)
    TooLong {
        name: String,
        length: usize,
        max_length: usize,
    },
    /// Name is empty
    Empty,
    /// Name starts with invalid character (must start with letter or underscore)
    InvalidStartCharacter(String),
    /// Name is a reserved SQL keyword
    ReservedKeyword(String),
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::InvalidCharacters(name) => {
                write!(f, "Invalid characters in name '{}': only alphanumeric characters and underscores are allowed", name)
            }
            ValidationError::TooLong {
                name,
                length,
                max_length,
            } => {
                write!(
                    f,
                    "Name '{}' is too long: {} characters (max {})",
                    name, length, max_length
                )
            }
            ValidationError::Empty => {
                write!(f, "Name cannot be empty")
            }
            ValidationError::InvalidStartCharacter(name) => {
                write!(f, "Name '{}' must start with a letter or underscore", name)
            }
            ValidationError::ReservedKeyword(name) => {
                write!(f, "Name '{}' is a reserved SQL keyword", name)
            }
        }
    }
}

impl std::error::Error for ValidationError {}

/// A validated table name that is safe to use in SQL queries
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ValidatedTableName(String);

impl ValidatedTableName {
    /// PostgreSQL identifier length limit
    const MAX_LENGTH: usize = 63;

    /// Create a new validated table name
    pub fn new(name: &str) -> Result<Self, ValidationError> {
        Self::validate_identifier(name)?;
        Ok(Self(name.to_string()))
    }

    /// Get the validated name as a string slice
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Get the validated name as a String
    pub fn into_string(self) -> String {
        self.0
    }

    /// Common validation logic for SQL identifiers
    fn validate_identifier(name: &str) -> Result<(), ValidationError> {
        // Check if empty
        if name.is_empty() {
            return Err(ValidationError::Empty);
        }

        // Check length
        if name.len() > Self::MAX_LENGTH {
            return Err(ValidationError::TooLong {
                name: name.to_string(),
                length: name.len(),
                max_length: Self::MAX_LENGTH,
            });
        }

        // Check first character (must be letter or underscore)
        let first_char = name.chars().next().ok_or(ValidationError::Empty)?; // This should never happen due to empty check above, but being defensive
        if !first_char.is_ascii_alphabetic() && first_char != '_' {
            return Err(ValidationError::InvalidStartCharacter(name.to_string()));
        }

        // Check all characters (alphanumeric or underscore only)
        if !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
            return Err(ValidationError::InvalidCharacters(name.to_string()));
        }

        // Check for reserved keywords
        if Self::is_reserved_keyword(name) {
            return Err(ValidationError::ReservedKeyword(name.to_string()));
        }

        Ok(())
    }

    /// Check if a name is a reserved SQL keyword
    fn is_reserved_keyword(name: &str) -> bool {
        // Common SQL reserved keywords that should not be used as identifiers
        const RESERVED_KEYWORDS: &[&str] = &[
            // SQL Standard keywords
            "SELECT",
            "INSERT",
            "UPDATE",
            "DELETE",
            "FROM",
            "WHERE",
            "JOIN",
            "INNER",
            "LEFT",
            "RIGHT",
            "FULL",
            "OUTER",
            "ON",
            "AS",
            "AND",
            "OR",
            "NOT",
            "NULL",
            "TRUE",
            "FALSE",
            "CASE",
            "WHEN",
            "THEN",
            "ELSE",
            "END",
            "IF",
            "EXISTS",
            "IN",
            "LIKE",
            "BETWEEN",
            "ORDER",
            "BY",
            "GROUP",
            "HAVING",
            "LIMIT",
            "OFFSET",
            "UNION",
            "ALL",
            "DISTINCT",
            "COUNT",
            "SUM",
            "AVG",
            "MIN",
            "MAX",
            "CREATE",
            "DROP",
            "ALTER",
            "TABLE",
            "INDEX",
            "VIEW",
            "DATABASE",
            "SCHEMA",
            "PRIMARY",
            "KEY",
            "FOREIGN",
            "REFERENCES",
            "UNIQUE",
            "CHECK",
            "DEFAULT",
            "CONSTRAINT",
            "COLUMN",
            "ADD",
            "MODIFY",
            "RENAME",
            "TO",
            // PostgreSQL specific keywords
            "SERIAL",
            "BIGSERIAL",
            "SMALLSERIAL",
            "TEXT",
            "VARCHAR",
            "CHAR",
            "INTEGER",
            "BIGINT",
            "SMALLINT",
            "DECIMAL",
            "NUMERIC",
            "REAL",
            "DOUBLE",
            "PRECISION",
            "BOOLEAN",
            "DATE",
            "TIME",
            "TIMESTAMP",
            "TIMESTAMPTZ",
            "INTERVAL",
            "UUID",
            "JSON",
            "JSONB",
            "ARRAY",
            "RETURNING",
            "CONFLICT",
            "NOTHING",
            "EXCLUDED",
            "GENERATED",
            "ALWAYS",
            "STORED",
            "IDENTITY",
            "CYCLE",
            "RESTART",
            "CACHE",
            "OWNED",
            "SEQUENCE",
            "TRIGGER",
            "FUNCTION",
            "PROCEDURE",
            "LANGUAGE",
            "PLPGSQL",
            "SECURITY",
            "DEFINER",
            "INVOKER",
            "STABLE",
            "IMMUTABLE",
            "VOLATILE",
            "STRICT",
            "CALLED",
            "RETURNS",
            "DECLARE",
            "BEGIN",
            "EXCEPTION",
            // System fields to prevent conflicts
            "__CREATED_AT__",
            "__UPDATED_AT__",
            "__IS_ACTIVE__",
            "__TAGS__",
        ];

        RESERVED_KEYWORDS.contains(&name.to_ascii_uppercase().as_str())
    }
}

impl fmt::Display for ValidatedTableName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A validated field name that is safe to use in SQL queries
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ValidatedFieldName(String);

impl ValidatedFieldName {
    /// Create a new validated field name
    pub fn new(name: &str) -> Result<Self, ValidationError> {
        ValidatedTableName::validate_identifier(name)?;
        Ok(Self(name.to_string()))
    }

    /// Get the validated name as a string slice
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Get the validated name as a String
    pub fn into_string(self) -> String {
        self.0
    }
}

impl fmt::Display for ValidatedFieldName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Utility functions for validation
pub mod utils {
    use super::*;

    /// Validate a table name and return the validated version
    pub fn validate_table_name(name: &str) -> Result<ValidatedTableName, ValidationError> {
        ValidatedTableName::new(name)
    }

    /// Validate a field name and return the validated version
    pub fn validate_field_name(name: &str) -> Result<ValidatedFieldName, ValidationError> {
        ValidatedFieldName::new(name)
    }

    /// Check if a string is a valid table name without creating a ValidatedTableName
    pub fn is_valid_table_name(name: &str) -> bool {
        ValidatedTableName::new(name).is_ok()
    }

    /// Check if a string is a valid field name without creating a ValidatedFieldName
    pub fn is_valid_field_name(name: &str) -> bool {
        ValidatedFieldName::new(name).is_ok()
    }

    /// Sanitize a name by replacing invalid characters with underscores
    /// Note: This should be used carefully as it may create naming conflicts
    pub fn sanitize_name(name: &str) -> String {
        let mut sanitized = String::with_capacity(name.len());

        for (i, c) in name.chars().enumerate() {
            if i == 0 {
                // First character must be letter or underscore
                if c.is_ascii_alphabetic() || c == '_' {
                    sanitized.push(c);
                } else {
                    sanitized.push('_');
                }
            } else {
                // Other characters can be alphanumeric or underscore
                if c.is_ascii_alphanumeric() || c == '_' {
                    sanitized.push(c);
                } else {
                    sanitized.push('_');
                }
            }
        }

        // Truncate if too long
        if sanitized.len() > ValidatedTableName::MAX_LENGTH {
            sanitized.truncate(ValidatedTableName::MAX_LENGTH);
        }

        // Ensure not empty
        if sanitized.is_empty() {
            sanitized = "unnamed".to_string();
        }

        sanitized
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_table_names() {
        let valid_names = [
            "users",
            "user_profiles",
            "UserProfiles",
            "_private_table",
            "table123",
            "a",
            &"a".repeat(63), // Max length
        ];

        for name in valid_names {
            assert!(
                ValidatedTableName::new(name).is_ok(),
                "Should accept valid name: {}",
                name
            );
        }
    }

    #[test]
    fn test_invalid_table_names() {
        let test_cases = [
            ("", ValidationError::Empty),
            (
                "123table",
                ValidationError::InvalidStartCharacter("123table".to_string()),
            ),
            (
                "user-name",
                ValidationError::InvalidCharacters("user-name".to_string()),
            ),
            (
                "user name",
                ValidationError::InvalidCharacters("user name".to_string()),
            ),
            (
                "user@domain",
                ValidationError::InvalidCharacters("user@domain".to_string()),
            ),
            (
                "SELECT",
                ValidationError::ReservedKeyword("SELECT".to_string()),
            ),
            (
                "select",
                ValidationError::ReservedKeyword("select".to_string()),
            ),
            (
                "__created_at__",
                ValidationError::ReservedKeyword("__created_at__".to_string()),
            ),
        ];

        for (name, expected_error) in test_cases {
            let result = ValidatedTableName::new(name);
            assert!(result.is_err(), "Should reject invalid name: {}", name);
            assert_eq!(result.unwrap_err(), expected_error);
        }
    }

    #[test]
    fn test_too_long_name() {
        let long_name = "a".repeat(64); // One character over limit
        let result = ValidatedTableName::new(&long_name);

        assert!(result.is_err());
        match result.unwrap_err() {
            ValidationError::TooLong {
                length, max_length, ..
            } => {
                assert_eq!(length, 64);
                assert_eq!(max_length, 63);
            }
            _ => panic!("Expected TooLong error"),
        }
    }

    #[test]
    fn test_field_name_validation() {
        // Field names use the same validation as table names
        assert!(ValidatedFieldName::new("id").is_ok());
        assert!(ValidatedFieldName::new("user_id").is_ok());
        assert!(ValidatedFieldName::new("123field").is_err());
        assert!(ValidatedFieldName::new("SELECT").is_err());
    }

    #[test]
    fn test_sanitize_name() {
        let test_cases = [
            ("user-name", "user_name"),
            ("user name", "user_name"),
            ("123user", "_23user"),
            ("user@domain.com", "user_domain_com"),
            ("", "unnamed"),
        ];

        for (input, expected) in test_cases {
            let result = utils::sanitize_name(input);
            assert_eq!(result, expected, "Sanitization failed for: {}", input);
        }
    }

    #[test]
    fn test_sanitize_long_name() {
        let long_name = "a".repeat(100);
        let sanitized = utils::sanitize_name(&long_name);
        assert_eq!(sanitized.len(), 63);
        assert_eq!(sanitized, "a".repeat(63));
    }

    #[test]
    fn test_reserved_keywords() {
        let keywords = ["SELECT", "INSERT", "UPDATE", "DELETE", "FROM", "WHERE"];

        for keyword in keywords {
            assert!(ValidatedTableName::new(keyword).is_err());
            assert!(ValidatedTableName::new(&keyword.to_lowercase()).is_err());
        }
    }

    #[test]
    fn test_system_fields_reserved() {
        let system_fields = [
            "__created_at__",
            "__updated_at__",
            "__is_active__",
            "__tags__",
        ];

        for field in system_fields {
            assert!(ValidatedFieldName::new(field).is_err());
            assert!(ValidatedFieldName::new(&field.to_uppercase()).is_err());
        }
    }

    #[test]
    fn test_display_traits() {
        let table_name = ValidatedTableName::new("users").unwrap();
        let field_name = ValidatedFieldName::new("id").unwrap();

        assert_eq!(format!("{}", table_name), "users");
        assert_eq!(format!("{}", field_name), "id");
    }

    #[test]
    fn test_utility_functions() {
        assert!(utils::is_valid_table_name("users"));
        assert!(!utils::is_valid_table_name("SELECT"));

        assert!(utils::is_valid_field_name("id"));
        assert!(!utils::is_valid_field_name("123field"));
    }
}
