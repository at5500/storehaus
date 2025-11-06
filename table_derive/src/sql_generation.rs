//! SQL code generation for database operations
//!
//! This module generates SQL queries and Rust code for database operations
//! based on parsed table and field metadata.

use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

use crate::parsing::{FieldInfo, TableInfo};

/// Validate and escape SQL identifier to prevent injection
/// This function ensures that field names are safe for SQL generation
/// Note: Table and field names are already validated at parse time,
/// so this is an additional safety check
fn safe_sql_identifier(name: &str) -> String {
    // At this point, names should already be validated by parsing stage
    // This is a defensive check to ensure no invalid names slip through

    // Basic validation (should never fail if parsing validation worked)
    if name.is_empty() {
        panic!("SQL identifier cannot be empty");
    }

    if name.len() > 63 {
        panic!(
            "SQL identifier '{}' is too long: {} characters (max 63)",
            name,
            name.len()
        );
    }

    // Check characters (should already be validated)
    if !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        panic!("SQL identifier '{}' contains invalid characters", name);
    }

    // Check first character
    let first_char = name
        .chars()
        .next()
        .unwrap_or_else(|| panic!("SQL identifier cannot be empty"));
    if !first_char.is_ascii_alphabetic() && first_char != '_' {
        panic!(
            "SQL identifier '{}' must start with letter or underscore",
            name
        );
    }

    // Use double quotes to safely escape the identifier
    // This protects against any edge cases and reserved words
    format!("\"{}\"", name)
}

pub fn generate_table_metadata_impl(
    name: &Ident,
    table_info: &TableInfo,
    field_info: &FieldInfo,
) -> TokenStream {
    let table_name = &table_info.name;

    let primary_key_field = &field_info.primary_key_field;
    let primary_key_type = &field_info.primary_key_type;
    let create_fields = &field_info.create_fields;
    let update_fields = &field_info.update_fields;
    let soft_delete_field = &field_info.soft_delete_field;

    // Parse the primary key type into a TokenStream if present
    let primary_key_type_tokens: Option<TokenStream> = primary_key_type.as_ref().map(|pk_type| {
        pk_type.parse().unwrap_or_else(|e| {
            panic!(
                "Failed to parse primary key type '{}': {}",
                pk_type, e
            )
        })
    });

    // Generate CREATE SQL
    let create_field_names: Vec<_> = create_fields
        .iter()
        .map(|f| safe_sql_identifier(f))
        .collect();
    let create_placeholders: Vec<_> = (1..=create_fields.len())
        .map(|i| format!("${}", i))
        .collect();
    let create_sql = format!(
        "INSERT INTO {} ({}, __created_at__, __updated_at__) VALUES ({}, NOW(), NOW()) RETURNING *",
        safe_sql_identifier(table_name),
        create_field_names.join(", "),
        create_placeholders.join(", ")
    );

    // Generate UPDATE SQL - only if primary key exists
    let update_sql = if let Some(pk_field) = primary_key_field {
        let update_assignments: Vec<_> = update_fields
            .iter()
            .enumerate()
            .map(|(i, field)| format!("{} = ${}", safe_sql_identifier(field), i + 1))
            .collect();
        format!(
            "UPDATE {} SET {}, __updated_at__ = NOW() WHERE {} = ${} RETURNING *",
            safe_sql_identifier(table_name),
            update_assignments.join(", "),
            safe_sql_identifier(&pk_field.to_string()),
            update_fields.len() + 1
        )
    } else {
        // For tables without primary key, UPDATE is not supported via this method
        String::new()
    };

    // Generate LIST_ALL SQL
    let list_all_sql = if let Some(soft_delete_field_name) = soft_delete_field {
        format!(
            "SELECT * FROM {} WHERE {} = TRUE ORDER BY \"__created_at__\" DESC",
            safe_sql_identifier(table_name),
            safe_sql_identifier(soft_delete_field_name)
        )
    } else {
        format!(
            "SELECT * FROM {} ORDER BY \"__created_at__\" DESC",
            safe_sql_identifier(table_name)
        )
    };

    // Generate DELETE_BY_ID SQL - only if primary key exists
    let delete_by_id_sql = if let Some(pk_field) = primary_key_field {
        format!(
            "DELETE FROM {} WHERE {} = $1",
            safe_sql_identifier(table_name),
            safe_sql_identifier(&pk_field.to_string())
        )
    } else {
        // For tables without primary key, DELETE requires manual query building
        String::new()
    };

    // Generate GET_BY_ID SQL - only if primary key exists
    let get_by_id_sql = if let Some(pk_field) = primary_key_field {
        format!(
            "SELECT * FROM {} WHERE {} = $1",
            safe_sql_identifier(table_name),
            safe_sql_identifier(&pk_field.to_string())
        )
    } else {
        // For tables without primary key, lookups require manual query building
        String::new()
    };

    // Generate COUNT_ALL SQL
    let count_all_sql = format!(
        "SELECT COUNT(*) as total FROM {}",
        safe_sql_identifier(table_name)
    );

    // Generate SELECT_BASE SQL
    let select_base_sql = format!("SELECT * FROM {}", safe_sql_identifier(table_name));

    // Generate COUNT_BASE SQL
    let count_base_sql = format!("SELECT COUNT(*) FROM {}", safe_sql_identifier(table_name));

    let create_fields_vec = quote! {
        vec![#(#create_fields),*]
    };

    let update_fields_vec = quote! {
        vec![#(#update_fields),*]
    };

    let soft_delete_field_option = match soft_delete_field {
        Some(field) => quote! { Some(#field) },
        None => quote! { None },
    };

    let has_soft_delete = soft_delete_field.is_some();

    // Generate binding expressions for update fields
    let _bind_calls: Vec<_> = update_fields
        .iter()
        .map(|field_name| {
            let field_ident: proc_macro2::Ident =
                Ident::new(field_name, proc_macro2::Span::call_site());
            quote! { .bind(self.#field_ident.clone()) }
        })
        .collect();

    // Generate type Id and methods that depend on primary key
    let (id_type, extract_id_impl, primary_key_field_impl) = if let Some(pk_type_tokens) = primary_key_type_tokens {
        let pk_field = primary_key_field.as_ref().unwrap();
        (
            quote! { type Id = #pk_type_tokens; },
            quote! {
                fn extract_id(&self) -> Self::Id {
                    self.#pk_field
                }
            },
            quote! {
                fn primary_key_field() -> &'static str {
                    stringify!(#pk_field)
                }
            }
        )
    } else {
        (
            quote! { type Id = store_object::NoId; },
            quote! {
                fn extract_id(&self) -> Self::Id {
                    store_object::NoId
                }
            },
            quote! {
                fn primary_key_field() -> &'static str {
                    ""
                }
            }
        )
    };

    quote! {
        impl store_object::TableMetadata for #name {
            #id_type

            fn table_name() -> &'static str {
                #table_name
            }

            fn create_sql() -> &'static str {
                #create_sql
            }

            fn update_sql() -> &'static str {
                #update_sql
            }

            fn list_all_sql() -> &'static str {
                #list_all_sql
            }

            fn delete_by_id_sql() -> &'static str {
                #delete_by_id_sql
            }

            fn get_by_id_sql() -> &'static str {
                #get_by_id_sql
            }

            fn count_all_sql() -> &'static str {
                #count_all_sql
            }

            fn select_base_sql() -> &'static str {
                #select_base_sql
            }

            fn count_base_sql() -> &'static str {
                #count_base_sql
            }

            fn supports_soft_delete() -> bool {
                #has_soft_delete
            }

            fn soft_delete_field() -> Option<&'static str> {
                #soft_delete_field_option
            }

            #extract_id_impl

            fn create_fields() -> Vec<&'static str> {
                #create_fields_vec
            }

            fn update_fields() -> Vec<&'static str> {
                #update_fields_vec
            }

            #primary_key_field_impl

            fn create_table_sql() -> String {
                Self::generate_create_table_sql()
            }

            fn get_table_fields() -> Vec<(&'static str, &'static str)> {
                Self::generate_table_fields()
            }

            fn create_indexes_sql() -> Vec<String> {
                Self::generate_indexes_sql()
            }

            // Database operations moved to DatabaseExecutor trait

            fn bind_update_params_owned<'a>(
                &'a self,
                sql: &'a str
            ) -> sqlx::query::QueryAs<'a, sqlx::Postgres, Self, sqlx::postgres::PgArguments>
            where
                Self: Sized
            {
                let query = sqlx::query_as::<_, Self>(sql);
                query #(#_bind_calls)*
            }

            fn bind_update_params_raw_owned<'a>(
                &'a self,
                sql: &'a str
            ) -> sqlx::query::Query<'a, sqlx::Postgres, sqlx::postgres::PgArguments>
            {
                let query = sqlx::query(sql);
                query #(#_bind_calls)*
            }

        }
    }
}

pub fn generate_helper_impl(
    name: &Ident,
    table_info: &TableInfo,
    field_info: &FieldInfo,
) -> TokenStream {
    let table_name = &table_info.name;
    let has_auto_increment = &table_info.has_auto_increment;

    let primary_key_field = &field_info.primary_key_field;
    let primary_key_type = &field_info.primary_key_type;
    let soft_delete_field = &field_info.soft_delete_field;

    // Parse the primary key type into a TokenStream if present
    let primary_key_type_tokens: Option<TokenStream> = primary_key_type.as_ref().map(|pk_type| {
        pk_type.parse().unwrap_or_else(|e| {
            panic!(
                "Failed to parse primary key type '{}': {}",
                pk_type, e
            )
        })
    });

    // Generate proper soft_delete_field token for use in conditionals
    let soft_delete_field_option = match soft_delete_field {
        Some(field) => quote! { Some(#field) },
        None => quote! { None },
    };

    // Generate field type mappings for compile-time injection
    let field_type_mappings: Vec<_> = field_info
        .field_types
        .iter()
        .map(|(name, rust_type)| {
            let rust_type_str = rust_type.as_str();
            quote! {
                types.insert(#name, #rust_type_str);
            }
        })
        .collect();

    quote! {
        // Generate helper methods for DDL operations in a separate impl block
        impl #name {
            // Helper function for safe SQL identifiers
            fn safe_sql_identifier(name: &str) -> String {
                // At this point, names should already be validated by parsing stage
                // This is a defensive check to ensure no invalid names slip through
                if name.is_empty() {
                    return "\"\"".to_string();
                }

                // Quote SQL identifiers to prevent keyword conflicts
                format!("\"{}\"", name.replace("\"", "\"\""))
            }
            fn generate_create_table_sql() -> String {
                let table_name = #table_name;

                // Generate field definitions
                let mut field_definitions = Vec::new();

                // Create field_types map at compile time
                let field_types = Self::get_field_types();

                // Add primary key with proper type and default (only if primary key exists)
                let pk_field_name = stringify!(#primary_key_field);

                // Only add primary key if it's not empty (tables without primary key will have empty string)
                if !pk_field_name.is_empty() {
                    let pk_rust_type = stringify!(#primary_key_type_tokens);

                    // Determine if this is an auto-increment field
                    let is_pk_auto_increment = {
                        let has_auto_inc = #has_auto_increment;
                        has_auto_inc
                    };

                    let (pk_pg_type, pk_default) = if is_pk_auto_increment {
                        // Use SERIAL types for auto-increment fields
                        match pk_rust_type.trim() {
                            "i16" => ("SMALLSERIAL".to_string(), "".to_string()),
                            "i32" => ("SERIAL".to_string(), "".to_string()),
                            "i64" => ("BIGSERIAL".to_string(), "".to_string()),
                            _ => ("SERIAL".to_string(), "".to_string()) // Default to SERIAL
                        }
                    } else {
                        // Regular field types
                        let pg_type = Self::rust_type_to_pg_type(pk_rust_type);
                        let default = match pk_rust_type.trim() {
                            "Uuid" | "uuid :: Uuid" | "uuid::Uuid" => "DEFAULT gen_random_uuid()",
                            _ => ""
                        };
                        (pg_type.to_string(), default.to_string())
                    };

                    field_definitions.push(format!(
                        "{} {} PRIMARY KEY {}",
                        pk_field_name,
                        pk_pg_type,
                        pk_default
                    ));
                }

                // Add create fields with their actual types
                let field_types = Self::get_field_types();
                for field_name in Self::create_fields() {
                    if field_name != pk_field_name {
                        if let Some(rust_type) = field_types.get(field_name) {
                            let pg_type = Self::rust_type_to_pg_type(rust_type);
                            let constraint = if ::storehaus::type_mapping::is_optional_type(rust_type) {
                                ""  // Optional types are nullable
                            } else {
                                " NOT NULL"  // Required types are NOT NULL
                            };
                            let safe_field_name = Self::safe_sql_identifier(field_name);
                            field_definitions.push(format!("{} {}{}", safe_field_name, pg_type, constraint));
                        } else {
                            // Fallback: если тип не найден, используем VARCHAR
                            println!("Warning: No type found for field {}, using VARCHAR", field_name);
                            let safe_field_name = Self::safe_sql_identifier(field_name);
                            field_definitions.push(format!("{} VARCHAR NOT NULL", safe_field_name));
                        }
                    }
                }

                // Add readonly system fields (timestamps)
                field_definitions.push("__created_at__ TIMESTAMP WITH TIME ZONE DEFAULT NOW()".to_string());
                field_definitions.push("__updated_at__ TIMESTAMP WITH TIME ZONE DEFAULT NOW()".to_string());

                // Add soft delete system field if present
                if let Some(soft_delete_field_name) = #soft_delete_field_option {
                    let safe_field_name = Self::safe_sql_identifier(soft_delete_field_name);
                    field_definitions.push(format!("{} BOOLEAN DEFAULT TRUE", safe_field_name));
                }

                // Add tags field for tagging operations
                field_definitions.push("__tags__ TEXT[] DEFAULT '{}'".to_string());

                format!(
                    "CREATE TABLE IF NOT EXISTS {} ({})",
                    Self::safe_sql_identifier(table_name),
                    field_definitions.join(", ")
                )
            }

            fn generate_table_fields() -> Vec<(&'static str, &'static str)> {
                let mut fields = Vec::new();
                fields.push((stringify!(#primary_key_field), Self::rust_type_to_pg_type(stringify!(#primary_key_type_tokens))));

                // Get field types map to properly map Rust types to PostgreSQL types
                let field_types = Self::get_field_types();

                // Add create/update fields with their actual PostgreSQL types
                for field_name in Self::create_fields() {
                    if field_name != stringify!(#primary_key_field) {
                        if let Some(rust_type) = field_types.get(field_name) {
                            let pg_type = Self::rust_type_to_pg_type(rust_type);
                            fields.push((field_name, pg_type));
                        } else {
                            // Fallback to VARCHAR if type not found
                            fields.push((field_name, "VARCHAR"));
                        }
                    }
                }

                fields.push(("__created_at__", "TIMESTAMP WITH TIME ZONE"));
                fields.push(("__updated_at__", "TIMESTAMP WITH TIME ZONE"));

                if let Some(soft_delete_field_name) = #soft_delete_field_option {
                    fields.push((soft_delete_field_name, "BOOLEAN"));
                }

                fields
            }

            fn generate_indexes_sql() -> Vec<String> {
                let table_name = #table_name;
                let safe_table_name = Self::safe_sql_identifier(table_name);
                let mut indexes = Vec::new();

                // Add index for soft delete if present
                if let Some(soft_delete_field_name) = #soft_delete_field_option {
                    let safe_field_name = Self::safe_sql_identifier(soft_delete_field_name);
                    indexes.push(format!(
                        "CREATE INDEX IF NOT EXISTS idx_{}_{} ON {} ({})",
                        table_name, soft_delete_field_name, safe_table_name, safe_field_name
                    ));
                }

                // Add __created_at__ index
                indexes.push(format!(
                    "CREATE INDEX IF NOT EXISTS idx_{}_created_at ON {} (\"__created_at__\")",
                    table_name, safe_table_name
                ));

                // Add __updated_at__ index
                indexes.push(format!(
                    "CREATE INDEX IF NOT EXISTS idx_{}_updated_at ON {} (\"__updated_at__\")",
                    table_name, safe_table_name
                ));

                // Add GIN index for tags array for efficient tag searching
                indexes.push(format!(
                    "CREATE INDEX IF NOT EXISTS idx_{}_tags ON {} USING GIN(\"__tags__\")",
                    table_name, safe_table_name
                ));

                indexes
            }

            fn rust_type_to_pg_type(rust_type: &str) -> &'static str {
                ::storehaus::type_mapping::rust_type_to_pg_type(rust_type)
            }

            fn get_field_types() -> std::collections::HashMap<&'static str, &'static str> {
                let mut types = std::collections::HashMap::new();

                // Add field type mappings using the extracted information
                #(#field_type_mappings)*

                types
            }
        }

    }
}

/// Generate DatabaseExecutor trait implementation with proper async methods
pub fn generate_database_executor_impl(name: &Ident, field_info: &FieldInfo) -> TokenStream {
    // Generate binding expressions for create fields
    let create_bind_calls: Vec<_> = field_info
        .create_fields
        .iter()
        .map(|field_name| {
            let field_ident: proc_macro2::Ident =
                Ident::new(field_name, proc_macro2::Span::call_site());
            quote! { .bind(self.#field_ident.clone()) }
        })
        .collect();

    // Generate binding expressions for update fields
    let update_bind_calls: Vec<_> = field_info
        .update_fields
        .iter()
        .map(|field_name| {
            let field_ident: proc_macro2::Ident =
                Ident::new(field_name, proc_macro2::Span::call_site());
            quote! { .bind(self.#field_ident.clone()) }
        })
        .collect();

    // Generate update methods only if primary key exists
    let update_methods = if field_info.primary_key_field.is_some() {
        quote! {
            async fn execute_update(&self, pool: &sqlx::PgPool) -> Result<Self, store_object::StorehausError>
            where
                Self: Sized + Send + Sync,
                Self: for<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow>
            {
                let sql = Self::update_sql();
                let id = self.extract_id();
                sqlx::query_as::<_, Self>(sql)
                    #(#update_bind_calls)*
                    .bind(&id)
                    .fetch_one(pool)
                    .await
                    .map_err(|e| store_object::StorehausError::database_operation(Self::table_name(), "update", e))
            }

            async fn execute_update_tx(&self, tx: &mut sqlx::Transaction<'_, sqlx::Postgres>) -> Result<Self, store_object::StorehausError>
            where
                Self: Sized + Send + Sync,
                Self: for<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow>
            {
                let sql = Self::update_sql();
                let id = self.extract_id();
                sqlx::query_as::<_, Self>(sql)
                    #(#update_bind_calls)*
                    .bind(&id)
                    .fetch_one(tx.as_mut())
                    .await
                    .map_err(|e| store_object::StorehausError::database_operation(Self::table_name(), "update", e))
            }
        }
    } else {
        quote! {
            async fn execute_update(&self, pool: &sqlx::PgPool) -> Result<Self, store_object::StorehausError>
            where
                Self: Sized + Send + Sync,
                Self: for<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow>
            {
                Err(store_object::StorehausError::validation(
                    Self::table_name(),
                    "primary_key",
                    "Table has no primary key. Use update_where with QueryBuilder instead."
                ))
            }

            async fn execute_update_tx(&self, tx: &mut sqlx::Transaction<'_, sqlx::Postgres>) -> Result<Self, store_object::StorehausError>
            where
                Self: Sized + Send + Sync,
                Self: for<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow>
            {
                Err(store_object::StorehausError::validation(
                    Self::table_name(),
                    "primary_key",
                    "Table has no primary key. Use update_where with QueryBuilder instead."
                ))
            }
        }
    };

    quote! {
        #[async_trait::async_trait]
        impl store_object::DatabaseExecutor for #name {
            async fn execute_create(&self, pool: &sqlx::PgPool) -> Result<Self, store_object::StorehausError>
            where
                Self: Sized + Send + Sync,
                Self: for<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow>
            {
                let sql = Self::create_sql();
                sqlx::query_as::<_, Self>(sql)
                    #(#create_bind_calls)*
                    .fetch_one(pool)
                    .await
                    .map_err(|e| store_object::StorehausError::database_operation(Self::table_name(), "create", e))
            }

            #update_methods

            async fn execute_create_tx(&self, tx: &mut sqlx::Transaction<'_, sqlx::Postgres>) -> Result<Self, store_object::StorehausError>
            where
                Self: Sized + Send + Sync,
                Self: for<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow>
            {
                let sql = Self::create_sql();
                sqlx::query_as::<_, Self>(sql)
                    #(#create_bind_calls)*
                    .fetch_one(tx.as_mut())
                    .await
                    .map_err(|e| store_object::StorehausError::database_operation(Self::table_name(), "create", e))
            }
        }
    }
}
