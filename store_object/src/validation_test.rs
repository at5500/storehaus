//! Validation Test module
//!
//! This module provides validation test functionality.

// Test file to verify validation works with various edge cases
use super::validation::*;

// Test with valid table name
struct TestUser {
    id: i32,
    name: String,
}

// This should compile fine
#[derive(table_derive::TableMetadata)]
#[table(name = "users")]
struct ValidTable {
    #[primary_key]
    id: i32,
    #[field(create, update)]
    name: String,
}

// Test edge cases that should fail at compile time if uncommented:

// #[derive(table_derive::TableMetadata)]
// #[table(name = "SELECT")]  // Reserved keyword - should fail
// struct InvalidReservedTable {
//     #[primary_key]
//     id: i32,
// }

// #[derive(table_derive::TableMetadata)]
// #[table(name = "123invalid")]  // Starts with number - should fail
// struct InvalidStartTable {
//     #[primary_key]
//     id: i32,
// }

// #[derive(table_derive::TableMetadata)]
// #[table(name = "user-name")]  // Invalid character - should fail
// struct InvalidCharTable {
//     #[primary_key]
//     id: i32,
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runtime_validation() {
        // Test valid names
        assert!(ValidatedTableName::new("users").is_ok());
        assert!(ValidatedTableName::new("user_profiles").is_ok());
        assert!(ValidatedTableName::new("_private").is_ok());

        // Test invalid names
        assert!(ValidatedTableName::new("SELECT").is_err());
        assert!(ValidatedTableName::new("123table").is_err());
        assert!(ValidatedTableName::new("user-name").is_err());
        assert!(ValidatedTableName::new("").is_err());

        // Test field names
        assert!(ValidatedFieldName::new("id").is_ok());
        assert!(ValidatedFieldName::new("user_id").is_ok());
        assert!(ValidatedFieldName::new("SELECT").is_err());
        assert!(ValidatedFieldName::new("123field").is_err());
    }

    #[test]
    fn test_sql_injection_prevention() {
        // These should all be rejected
        let malicious_names = [
            "users; DROP TABLE users; --",
            "users' OR '1'='1",
            "users/**/UNION/**/SELECT",
            "users\"; DELETE FROM users; --",
        ];

        for name in malicious_names {
            assert!(ValidatedTableName::new(name).is_err(),
                   "Should reject malicious name: {}", name);
        }
    }
}
