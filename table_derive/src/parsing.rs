//! Parsing utilities for table and field attributes
//!
//! This module handles the parsing of `#[table]` and `#[field]` attributes
//! and validation of table and field names.

use quote::quote;
use std::collections::HashMap;
use syn::{
    parse::Parse, parse::ParseStream, Attribute, Data, Error, Fields, Ident, Meta, Result, Token,
};

/// Validate table name and return syn::Error for better proc macro error handling
pub fn validate_table_name_syn(name: &str, span: proc_macro2::Span) -> Result<()> {
    validate_identifier(name)
        .map_err(|e| Error::new(span, format!("Invalid table name '{}': {}", name, e)))
}

/// Validate field name and return syn::Error for better proc macro error handling
pub fn validate_field_name_syn(name: &str, span: proc_macro2::Span) -> Result<()> {
    // Allow system fields to bypass validation since they use reserved names intentionally
    if is_system_field(name) {
        return Ok(());
    }

    validate_identifier(name)
        .map_err(|e| Error::new(span, format!("Invalid field name '{}': {}", name, e)))
}

/// Check if a field name is a system field that should bypass validation
fn is_system_field(name: &str) -> bool {
    matches!(
        name,
        "__created_at__" | "__updated_at__" | "__tags__" | "__is_active__"
    )
}

/// Validation logic that mirrors store_object::validation module
/// This ensures compile-time validation matches runtime validation
fn validate_identifier(name: &str) -> std::result::Result<(), String> {
    // Check if empty
    if name.is_empty() {
        return Err("Name cannot be empty".to_string());
    }

    // Check length (PostgreSQL limit)
    if name.len() > 63 {
        return Err(format!(
            "Name '{}' is too long: {} characters (max 63)",
            name,
            name.len()
        ));
    }

    // Check first character (must be letter or underscore)
    let first_char = name
        .chars()
        .next()
        .ok_or_else(|| "Name cannot be empty".to_string())?;
    if !first_char.is_ascii_alphabetic() && first_char != '_' {
        return Err(format!(
            "Name '{}' must start with a letter or underscore",
            name
        ));
    }

    // Check all characters (alphanumeric or underscore only)
    if !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        return Err(format!("Name '{}' contains invalid characters: only alphanumeric characters and underscores are allowed", name));
    }

    // Check for reserved keywords (same list as in validation.rs)
    if is_reserved_keyword(name) {
        return Err(format!("Name '{}' is a reserved SQL keyword", name));
    }

    Ok(())
}

/// Check if a name is a reserved SQL keyword
/// This mirrors the logic in store_object::validation::ValidatedTableName
fn is_reserved_keyword(name: &str) -> bool {
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

#[derive(Debug)]
struct FieldOperations {
    operations: Vec<Ident>,
}

impl Parse for FieldOperations {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut operations = Vec::new();

        while !input.is_empty() {
            let op: Ident = input.parse()?;
            operations.push(op);

            if input.peek(Token![,]) {
                let _: Token![,] = input.parse()?;
            }
        }

        Ok(FieldOperations { operations })
    }
}

#[derive(Debug)]
pub struct TableInfo {
    pub name: String,
    pub has_auto_increment: bool,
    pub auto_soft_delete: bool,
    #[allow(dead_code)]
    pub composite_indexes: Vec<Vec<String>>,        // #[index(field1, field2)]
    #[allow(dead_code)]
    pub composite_unique_indexes: Vec<Vec<String>>, // #[unique(field1, field2)]
}

#[derive(Debug)]
pub struct FieldInfo {
    pub primary_key_field: Option<Ident>,
    pub primary_key_type: Option<String>,
    pub create_fields: Vec<String>,
    pub update_fields: Vec<String>,
    pub soft_delete_field: Option<String>,
    pub field_types: HashMap<String, String>, // field_name -> rust_type
    #[allow(dead_code)]
    pub indexed_fields: Vec<String>,          // fields marked with #[index]
    #[allow(dead_code)]
    pub unique_fields: Vec<String>,           // fields marked with #[unique]
}

pub fn parse_table_attributes(attrs: &[Attribute]) -> Result<TableInfo> {
    let mut table_name = None;
    let mut has_auto_increment = false;
    let mut auto_soft_delete = false;
    let mut composite_indexes = Vec::new();
    let mut composite_unique_indexes = Vec::new();

    // First pass: find #[table(...)] attribute
    for attr in attrs {
        if attr.path().is_ident("table") {
            if let Meta::List(meta_list) = &attr.meta {
                let mut name = None;

                // Parse nested tokens manually since syn 2.0 changed the API
                let mut tokens = meta_list.tokens.clone().into_iter().peekable();

                while let Some(token) = tokens.next() {
                    if let proc_macro2::TokenTree::Ident(key) = token {
                        let key_str = key.to_string();

                        // Expect '=' after key
                        if let Some(proc_macro2::TokenTree::Punct(punct)) = tokens.peek() {
                            if punct.as_char() == '=' {
                                tokens.next(); // consume '='

                                // Get the value
                                if let Some(proc_macro2::TokenTree::Literal(lit)) = tokens.next() {
                                    let value = lit.to_string().trim_matches('"').to_string();

                                    match key_str.as_str() {
                                        "name" => name = Some(value),
                                        _ => {} // ignore unknown keys
                                    }
                                }
                            }
                        }

                        // Skip comma if present
                        if let Some(proc_macro2::TokenTree::Punct(punct)) = tokens.peek() {
                            if punct.as_char() == ',' {
                                tokens.next(); // consume ','
                            }
                        }

                        match key_str.as_str() {
                            "auto_increment" => has_auto_increment = true,
                            "auto_soft_delete" => auto_soft_delete = true,
                            _ => {} // ignore unknown keys
                        }
                    }
                }

                table_name = name;
            }
        }
    }

    let table_name = table_name.ok_or_else(|| {
        Error::new(
            proc_macro2::Span::call_site(),
            "table attribute is required: add #[table(name = \"table_name\")] to your struct",
        )
    })?;

    // Validate table name at compile time with proper error handling
    validate_table_name_syn(&table_name, proc_macro2::Span::call_site())?;

    // Second pass: find #[index(...)] and #[unique(...)] attributes
    for attr in attrs {
        if attr.path().is_ident("index") {
            if let Meta::List(meta_list) = &attr.meta {
                let fields = parse_field_list(&meta_list.tokens)?;
                composite_indexes.push(fields);
            }
        } else if attr.path().is_ident("unique") {
            if let Meta::List(meta_list) = &attr.meta {
                let fields = parse_field_list(&meta_list.tokens)?;
                composite_unique_indexes.push(fields);
            }
        }
    }

    Ok(TableInfo {
        name: table_name,
        has_auto_increment,
        auto_soft_delete,
        composite_indexes,
        composite_unique_indexes,
    })
}

/// Parse a list of field names from tokens like (field1, field2, field3)
fn parse_field_list(tokens: &proc_macro2::TokenStream) -> Result<Vec<String>> {
    let mut fields = Vec::new();
    let mut tokens_iter = tokens.clone().into_iter().peekable();

    while let Some(token) = tokens_iter.next() {
        if let proc_macro2::TokenTree::Ident(ident) = token {
            fields.push(ident.to_string());
        }

        // Skip commas
        if let Some(proc_macro2::TokenTree::Punct(punct)) = tokens_iter.peek() {
            if punct.as_char() == ',' {
                tokens_iter.next();
            }
        }
    }

    if fields.is_empty() {
        return Err(Error::new(
            proc_macro2::Span::call_site(),
            "index or unique attribute requires at least one field name",
        ));
    }

    Ok(fields)
}

pub fn parse_field_attributes(data: &Data, table_info: &TableInfo) -> Result<FieldInfo> {
    if let Data::Struct(data_struct) = data {
        if let Fields::Named(fields_named) = &data_struct.fields {
            let mut primary_key_field = None;
            let mut primary_key_type = None;
            let mut create_fields = Vec::new();
            let mut update_fields = Vec::new();
            let mut soft_delete_field = None;
            let mut field_types = HashMap::new();
            let mut indexed_fields = Vec::new();
            let mut unique_fields = Vec::new();

            for field in &fields_named.named {
                let field_name = field
                    .ident
                    .as_ref()
                    .ok_or_else(|| Error::new_spanned(field, "Field must have a name"))?;
                let field_name_str = field_name.to_string();

                // Skip validation for soft delete fields as they're system fields
                if !has_attribute(&field.attrs, "soft_delete") {
                    // Validate field name at compile time with proper error handling
                    validate_field_name_syn(&field_name_str, field_name.span())?;
                }

                let ty = &field.ty;
                let type_string = quote!(#ty).to_string();
                // Normalize type string by removing all whitespace for consistent matching
                let normalized_type_string = type_string.replace(" ", "");

                // Store field type
                field_types.insert(field_name_str.clone(), normalized_type_string.clone());

                // Check for primary_key attribute
                if has_attribute(&field.attrs, "primary_key") {
                    primary_key_field = Some(field_name.clone());
                    primary_key_type = Some(normalized_type_string.clone());
                }

                // Check for soft_delete attribute
                if has_attribute(&field.attrs, "soft_delete") {
                    soft_delete_field = Some(field_name_str.clone());
                }

                // Check for index attributes
                if has_attribute(&field.attrs, "index") {
                    indexed_fields.push(field_name_str.clone());
                }

                // Check for unique attributes
                if has_attribute(&field.attrs, "unique") {
                    unique_fields.push(field_name_str.clone());
                }

                // Check for field attributes
                let is_readonly = has_attribute(&field.attrs, "readonly");

                if let Some(field_ops) = parse_field_operations(&field.attrs) {
                    if field_ops.contains(&"create".to_string()) {
                        create_fields.push(field_name_str.clone());
                    }
                    if field_ops.contains(&"update".to_string()) && !is_readonly {
                        update_fields.push(field_name_str);
                    }
                }
            }

            // Primary key is now optional - if not provided, table will have no primary key
            // This is useful for settings tables and other key-value stores
            let primary_key_field = primary_key_field;
            let primary_key_type = primary_key_type;

            // Add system fields to field_types and update_fields
            field_types.insert(
                "__created_at__".to_string(),
                "chrono::DateTime<chrono::Utc>".to_string(),
            );
            field_types.insert(
                "__updated_at__".to_string(),
                "chrono::DateTime<chrono::Utc>".to_string(),
            );
            field_types.insert("__tags__".to_string(), "Option<Vec<String>>".to_string());

            // Note: __tags__ is automatically added to update_fields because it has #[field(update)] attribute

            // If auto_soft_delete is enabled, automatically add __is_active__ field
            if table_info.auto_soft_delete && soft_delete_field.is_none() {
                soft_delete_field = Some("__is_active__".to_string());
                field_types.insert("__is_active__".to_string(), "bool".to_string());
                // Note: __is_active__ is automatically added to update_fields because it has #[field(update)] attribute
            }

            return Ok(FieldInfo {
                primary_key_field,
                primary_key_type,
                create_fields,
                update_fields,
                soft_delete_field,
                field_types,
                indexed_fields,
                unique_fields,
            });
        }
    }

    Err(Error::new(
        proc_macro2::Span::call_site(),
        "TableMetadata can only be derived for structs with named fields",
    ))
}

pub fn has_attribute(attrs: &[Attribute], name: &str) -> bool {
    attrs.iter().any(|attr| attr.path().is_ident(name))
}

pub fn parse_field_operations(attrs: &[Attribute]) -> Option<Vec<String>> {
    for attr in attrs {
        if attr.path().is_ident("field") {
            // Proper parsing using syn's Meta
            let meta = attr.meta.clone();
            return match meta {
                Meta::List(meta_list) => {
                    let mut operations = Vec::new();

                    // Parse using custom Parse implementation
                    if let Ok(field_ops) = meta_list.parse_args::<FieldOperations>() {
                        for ident in field_ops.operations {
                            match ident.to_string().as_str() {
                                "create" => operations.push("create".to_string()),
                                "update" => operations.push("update".to_string()),
                                "readonly" => {
                                    // readonly fields are not included in create/update
                                }
                                _ => {} // Ignore unknown operations
                            }
                        }
                    }

                    Some(operations)
                }
                Meta::Path(_) => {
                    // #[field] without arguments - default behavior
                    Some(vec!["create".to_string(), "update".to_string()])
                }
                Meta::NameValue(_) => {
                    // #[field = "value"] - not supported
                    None
                }
            };
        }
    }

    None
}

#[cfg(test)]
mod validation_tests {
    use super::*;

    // Helper functions for tests - these call the _syn versions but panic on error
    fn validate_table_name(name: &str) {
        if let Err(e) = validate_table_name_syn(name, proc_macro2::Span::call_site()) {
            panic!("Invalid table name: {}", e);
        }
    }

    fn validate_field_name(name: &str) {
        if let Err(e) = validate_field_name_syn(name, proc_macro2::Span::call_site()) {
            panic!("Invalid field name: {}", e);
        }
    }

    #[test]
    fn test_valid_table_names() {
        // These should not panic
        validate_table_name("users");
        validate_table_name("user_profiles");
        validate_table_name("_private");
        validate_table_name("table123");
        validate_table_name("a");
    }

    #[test]
    #[should_panic(expected = "Invalid table name")]
    fn test_reserved_keyword() {
        validate_table_name("SELECT");
    }

    #[test]
    #[should_panic(expected = "Invalid table name")]
    fn test_invalid_start() {
        validate_table_name("123table");
    }

    #[test]
    #[should_panic(expected = "Invalid table name")]
    fn test_invalid_chars() {
        validate_table_name("user-table");
    }

    #[test]
    #[should_panic(expected = "Invalid table name")]
    fn test_empty_name() {
        validate_table_name("");
    }

    #[test]
    fn test_field_validation() {
        // These should not panic
        validate_field_name("id");
        validate_field_name("user_id");
        validate_field_name("field123");
    }

    #[test]
    #[should_panic(expected = "Invalid field name")]
    fn test_invalid_field() {
        validate_field_name("SELECT");
    }

    #[test]
    fn test_sql_injection_prevention() {
        // These malicious names should be rejected
        let malicious_names = [
            "users; DROP TABLE users; --",
            "users' OR '1'='1",
            "users/**/UNION/**/SELECT",
            "users\"; DELETE FROM users; --",
        ];

        for name in malicious_names {
            let result = std::panic::catch_unwind(|| {
                validate_table_name(name);
            });
            assert!(result.is_err(), "Should panic for malicious name: {}", name);
        }
    }
}
