use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

use crate::parsing::FieldInfo;

pub fn generate_table_metadata_impl(
    name: &Ident,
    table_name: &str,
    field_info: &FieldInfo,
) -> TokenStream {
    let primary_key_field = &field_info.primary_key_field;
    let primary_key_type = &field_info.primary_key_type;
    let create_fields = &field_info.create_fields;
    let update_fields = &field_info.update_fields;
    let has_soft_delete = field_info.has_soft_delete;
    let _field_types = &field_info.field_types;

    // Parse the primary key type into a TokenStream
    let primary_key_type_tokens: proc_macro2::TokenStream = primary_key_type.parse().unwrap();

    // Generate CREATE SQL
    let create_field_names: Vec<_> = create_fields.iter().map(|f| f.as_str()).collect();
    let create_placeholders: Vec<_> = (1..=create_fields.len())
        .map(|i| format!("${}", i))
        .collect();
    let create_sql = format!(
        "INSERT INTO {} ({}, created_at, updated_at) VALUES ({}, NOW(), NOW()) RETURNING *",
        table_name,
        create_field_names.join(", "),
        create_placeholders.join(", ")
    );

    // Generate UPDATE SQL
    let update_assignments: Vec<_> = update_fields
        .iter()
        .enumerate()
        .map(|(i, field)| format!("{} = ${}", field, i + 1))
        .collect();
    let update_sql = format!(
        "UPDATE {} SET {}, updated_at = NOW() WHERE id = ${} RETURNING *",
        table_name,
        update_assignments.join(", "),
        update_fields.len() + 1
    );

    let create_fields_vec = quote! {
        vec![#(#create_fields),*]
    };

    let update_fields_vec = quote! {
        vec![#(#update_fields),*]
    };

    // Generate binding expressions for create fields
    let create_bind_calls: Vec<_> = create_fields.iter().map(|field_name| {
        let field_ident: proc_macro2::Ident = syn::Ident::new(field_name, proc_macro2::Span::call_site());
        quote! { .bind(&self.#field_ident) }
    }).collect();

    // Generate binding expressions for update fields
    let update_bind_calls: Vec<_> = update_fields.iter().map(|field_name| {
        let field_ident: proc_macro2::Ident = syn::Ident::new(field_name, proc_macro2::Span::call_site());
        quote! { .bind(&self.#field_ident) }
    }).collect();


    quote! {
        impl TableMetadata for #name {
            type Id = #primary_key_type_tokens;

            fn table_name() -> &'static str {
                #table_name
            }

            fn create_sql() -> &'static str {
                #create_sql
            }

            fn update_sql() -> &'static str {
                #update_sql
            }

            fn supports_soft_delete() -> bool {
                #has_soft_delete
            }

            fn extract_id(&self) -> Self::Id {
                self.#primary_key_field
            }

            fn create_fields() -> Vec<&'static str> {
                #create_fields_vec
            }

            fn update_fields() -> Vec<&'static str> {
                #update_fields_vec
            }

            fn primary_key_field() -> &'static str {
                stringify!(#primary_key_field)
            }

            fn create_table_sql() -> String {
                Self::generate_create_table_sql()
            }

            fn get_table_fields() -> Vec<(&'static str, &'static str)> {
                Self::generate_table_fields()
            }

            fn create_indexes_sql() -> Vec<String> {
                Self::generate_indexes_sql()
            }

            fn execute_create(&self, pool: &sqlx::PgPool) -> impl std::future::Future<Output = Result<Self, sqlx::Error>> + Send
            where
                Self: Sized + for<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow> + Send + Sync
            {
                let sql = Self::create_sql();
                async move {
                    sqlx::query_as::<_, Self>(sql)
                        #(#create_bind_calls)*
                        .fetch_one(pool)
                        .await
                }
            }

            fn execute_update(&self, pool: &sqlx::PgPool) -> impl std::future::Future<Output = Result<Self, sqlx::Error>> + Send
            where
                Self: Sized + for<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow> + Send + Sync
            {
                let sql = Self::update_sql();
                let id = self.extract_id();
                async move {
                    sqlx::query_as::<_, Self>(sql)
                        #(#update_bind_calls)*
                        .bind(&id)
                        .fetch_one(pool)
                        .await
                }
            }

            fn execute_create_tx(&self, tx: &mut sqlx::Transaction<'_, sqlx::Postgres>) -> impl std::future::Future<Output = Result<Self, sqlx::Error>> + Send
            where
                Self: Sized + for<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow> + Send + Sync
            {
                let sql = Self::create_sql();
                async move {
                    sqlx::query_as::<_, Self>(sql)
                        #(#create_bind_calls)*
                        .fetch_one(tx.as_mut())
                        .await
                }
            }

            fn execute_update_tx(&self, tx: &mut sqlx::Transaction<'_, sqlx::Postgres>) -> impl std::future::Future<Output = Result<Self, sqlx::Error>> + Send
            where
                Self: Sized + for<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow> + Send + Sync
            {
                let sql = Self::update_sql();
                let id = self.extract_id();
                async move {
                    sqlx::query_as::<_, Self>(sql)
                        #(#update_bind_calls)*
                        .bind(&id)
                        .fetch_one(tx.as_mut())
                        .await
                }
            }

        }
    }
}

pub fn generate_helper_impl(
    name: &Ident,
    table_name: &str,
    field_info: &FieldInfo,
) -> TokenStream {
    let primary_key_field = &field_info.primary_key_field;
    let primary_key_type = &field_info.primary_key_type;
    let has_soft_delete = field_info.has_soft_delete;
    let has_auto_increment = field_info.has_auto_increment;
    let auto_increment_field = field_info.auto_increment_field.as_ref().unwrap_or(&String::new()).clone();

    // Parse the primary key type into a TokenStream
    let primary_key_type_tokens: proc_macro2::TokenStream = primary_key_type.parse().unwrap();

    // Generate field type mappings for compile-time injection
    let field_type_mappings: Vec<_> = field_info.field_types.iter()
        .map(|(name, rust_type)| {
            quote! {
                types.insert(#name, #rust_type);
            }
        })
        .collect();

    quote! {
        // Generate helper methods for DDL operations in a separate impl block
        impl #name {
            fn generate_create_table_sql() -> String {
                let table_name = #table_name;

                // Generate field definitions
                let mut field_definitions = Vec::new();

                // Create field_types map at compile time
                let field_types = Self::get_field_types();

                // Add primary key with proper type and default
                let pk_field_name = stringify!(#primary_key_field);
                let pk_rust_type = stringify!(#primary_key_type_tokens);

                // Determine if this is an auto-increment field
                let is_pk_auto_increment = #has_auto_increment && #auto_increment_field == pk_field_name;

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

                // Add create fields with their actual types
                let field_types = Self::get_field_types();
                for field_name in Self::create_fields() {
                    if field_name != pk_field_name {
                        if let Some(rust_type) = field_types.get(field_name) {
                            let pg_type = Self::rust_type_to_pg_type(rust_type);
                            let constraint = " NOT NULL";
                            field_definitions.push(format!("{} {}{}", field_name, pg_type, constraint));
                        } else {
                            // Fallback: если тип не найден, используем VARCHAR
                            println!("Warning: No type found for field {}, using VARCHAR", field_name);
                            field_definitions.push(format!("{} VARCHAR NOT NULL", field_name));
                        }
                    }
                }

                // Add readonly fields (timestamps)
                field_definitions.push("created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()".to_string());
                field_definitions.push("updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()".to_string());

                // Add soft delete field if present
                if #has_soft_delete {
                    field_definitions.push("is_active BOOLEAN DEFAULT TRUE".to_string());
                }

                format!(
                    "CREATE TABLE IF NOT EXISTS {} ({})",
                    table_name,
                    field_definitions.join(", ")
                )
            }

            fn generate_table_fields() -> Vec<(&'static str, &'static str)> {
                let mut fields = Vec::new();
                fields.push((stringify!(#primary_key_field), Self::rust_type_to_pg_type(stringify!(#primary_key_type_tokens))));

                // Add create/update fields - need to determine types at compile time
                // This is a simplified version - in a real implementation,
                // you'd need to extract field types from the original struct
                for field_name in Self::create_fields() {
                    if field_name != stringify!(#primary_key_field) {
                        fields.push((field_name, "VARCHAR"));
                    }
                }

                fields.push(("created_at", "TIMESTAMP WITH TIME ZONE"));
                fields.push(("updated_at", "TIMESTAMP WITH TIME ZONE"));

                if #has_soft_delete {
                    fields.push(("is_active", "BOOLEAN"));
                }

                fields
            }

            fn generate_indexes_sql() -> Vec<String> {
                let table_name = #table_name;
                let mut indexes = Vec::new();

                // Add index for soft delete if present
                if #has_soft_delete {
                    indexes.push(format!(
                        "CREATE INDEX IF NOT EXISTS idx_{}_{} ON {}({})",
                        table_name, "is_active", table_name, "is_active"
                    ));
                }

                // Add created_at index
                indexes.push(format!(
                    "CREATE INDEX IF NOT EXISTS idx_{}_{} ON {}({})",
                    table_name, "created_at", table_name, "created_at"
                ));

                indexes
            }

            fn rust_type_to_pg_type(rust_type: &str) -> &'static str {
                match rust_type.trim() {
                    "Uuid" | "uuid :: Uuid" | "uuid::Uuid" => "UUID",
                    "String" => "VARCHAR",
                    "i8" => "SMALLINT",
                    "i16" => "SMALLINT",
                    "i32" => "INTEGER",
                    "i64" => "BIGINT",
                    "u16" => "INTEGER",
                    "u32" => "BIGINT",
                    "u64" => "NUMERIC(20,0)", // PostgreSQL doesn't have native u64
                    "f32" => "REAL",
                    "f64" => "DOUBLE PRECISION",
                    "bool" => "BOOLEAN",
                    "chrono :: DateTime < chrono :: Utc >" |
                    "chrono::DateTime<chrono::Utc>" |
                    "chrono :: NaiveDateTime" |
                    "chrono::NaiveDateTime" => "TIMESTAMP WITH TIME ZONE",
                    "chrono :: Date < chrono :: Utc >" |
                    "chrono::Date<chrono::Utc>" |
                    "chrono :: NaiveDate" |
                    "chrono::NaiveDate" => "DATE",
                    "rust_decimal :: Decimal" |
                    "rust_decimal::Decimal" => "NUMERIC(28,10)",
                    "bigdecimal :: BigDecimal" |
                    "bigdecimal::BigDecimal" => "NUMERIC",
                    _ => "VARCHAR" // default fallback
                }
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

