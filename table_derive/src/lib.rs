use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

mod parsing;
mod sql_generation;
mod model_macro;

use parsing::{parse_table_attributes, parse_field_attributes};
use sql_generation::{generate_table_metadata_impl, generate_helper_impl};
use model_macro::model_attribute;

/// Derive macro for TableMetadata trait
///
/// Usage:
/// ```rust
/// #[derive(TableMetadata)]
/// #[table(name = "customers")]
/// pub struct Customer {
///     #[primary_key]
///     pub id: Uuid,
///
///     #[field(create, update)]
///     pub first_name: String,
///
///     #[field(readonly)]
///     pub created_at: DateTime<Utc>,
///
///     #[soft_delete]
///     pub is_active: bool,
/// }
/// ```
#[proc_macro_derive(
    TableMetadata,
    attributes(table, primary_key, field, soft_delete, auto_increment)
)]
pub fn derive_table_metadata(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;

    // Parse table attributes
    let table_attrs = parse_table_attributes(&input.attrs);

    // Parse field attributes
    let field_info = parse_field_attributes(&input.data);

    let table_name = &table_attrs.name;

    // Generate the TableMetadata implementation
    let table_metadata_impl = generate_table_metadata_impl(name, table_name, &field_info);

    // Generate the helper methods implementation
    let helper_impl = generate_helper_impl(name, table_name, &field_info);

    let expanded = quote::quote! {
        #table_metadata_impl
        #helper_impl
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
