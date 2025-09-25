use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput};

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
pub fn model_attribute(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);

    let name = &input.ident;
    let attrs = &input.attrs;
    let vis = &input.vis;
    let generics = &input.generics;

    // Extract fields from the struct
    let fields = match &input.data {
        Data::Struct(data) => &data.fields,
        _ => panic!("model can only be used on structs"),
    };

    // Add all the necessary derives to the struct
    let expanded = quote! {
        #[derive(Debug, Clone, serde::Serialize, serde::Deserialize, sqlx::FromRow, sqlx::Type, TableMetadata)]
        #(#attrs)*
        #vis struct #name #generics #fields
    };

    TokenStream::from(expanded)
}