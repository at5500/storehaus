//! Procedural macros for generating database table metadata and operations
//!
//! This crate provides the `#[model]` macro and `TableMetadata` derive for automatic
//! generation of database operations and metadata for struct types.

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

mod model_macro;
mod parsing;
mod sql_generation;

use model_macro::model_attribute;
use parsing::{parse_field_attributes, parse_table_attributes};
use sql_generation::{
    generate_database_executor_impl, generate_helper_impl, generate_table_metadata_impl,
};

/// Derive macro for TableMetadata trait
///
/// Note: It's recommended to use the `#[model]` attribute macro instead,
/// which automatically includes this derive along with other necessary derives.
///
/// Manual usage (not recommended):
/// ```rust
/// #[derive(Debug, Clone, serde::Serialize, serde::Deserialize, sqlx::FromRow, sqlx::Type, TableMetadata)]
/// #[table(name = "customers")]
/// pub struct Customer {
///     #[primary_key]
///     pub id: Uuid,
///
///     #[field(create, update)]
///     pub first_name: String,
///
///     #[field(readonly)]
///     pub __created_at__: DateTime<Utc>,
///
///     #[soft_delete]
///     pub __is_active__: bool,
/// }
/// ```
///
/// Recommended usage:
/// ```rust
/// use table_derive::model;
///
/// #[model]
/// #[table(name = "customers")]
/// pub struct Customer {
///     #[primary_key]
///     pub id: Uuid,
///
///     #[field(create, update)]
///     pub first_name: String,
///
///     #[field(readonly)]
///     pub __created_at__: DateTime<Utc>,
///
///     #[soft_delete]
///     pub __is_active__: bool,
/// }
/// ```
#[proc_macro_derive(
    TableMetadata,
    attributes(table, primary_key, field, soft_delete, auto_increment, readonly, index, unique)
)]
pub fn derive_table_metadata(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;

    // Parse table attributes - handle errors properly
    let table_info = match parse_table_attributes(&input.attrs) {
        Ok(attrs) => attrs,
        Err(e) => return e.to_compile_error().into(),
    };

    // Parse field attributes - handle errors properly
    let field_info = match parse_field_attributes(&input.data, &table_info) {
        Ok(info) => info,
        Err(e) => return e.to_compile_error().into(),
    };

    // Generate the TableMetadata implementation
    let table_metadata_impl = generate_table_metadata_impl(name, &table_info, &field_info);

    // Generate the helper methods implementation
    let helper_impl = generate_helper_impl(name, &table_info, &field_info);

    // Generate the DatabaseExecutor implementation
    let database_executor_impl = generate_database_executor_impl(name, &field_info);

    let expanded = quote::quote! {
        #table_metadata_impl
        #helper_impl
        #database_executor_impl
    };

    TokenStream::from(expanded)
}

/// Convenience attribute macro that adds all necessary derives for a database model
///
/// Usage:
/// ```rust
/// use table_derive::model;
///
/// #[model]
/// #[table(name = "users")]
/// pub struct User {
///     #[primary_key]
///     pub id: Uuid,
///     #[field(create, update)]
///     pub name: String,
/// }
/// ```
#[proc_macro_attribute]
pub fn model(_attr: TokenStream, item: TokenStream) -> TokenStream {
    model_attribute(_attr, item)
}
