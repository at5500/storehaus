//! Implementation of the `#[model]` attribute macro
//!
//! This module provides the `#[model]` macro that automatically adds
//! system fields and derives to database model structs.

use crate::parsing::parse_table_attributes;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields};

/// Convenience attribute macro that adds all necessary derives for a database model
///
/// This macro automatically adds system fields (__created_at__, __updated_at__, __tags__,
/// and optionally __is_active__ for soft delete) and generates a convenient `new()` method
/// that only requires user-defined fields.
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
///
/// // Create instances easily without system fields:
/// let user = User::new(Uuid::new_v4(), "John Doe".to_string());
///
/// // With soft delete support:
/// #[model]
/// #[table(name = "users", auto_soft_delete)]
/// pub struct User {
///     #[primary_key]
///     pub id: Uuid,
///     #[field(create, update)]
///     pub name: String,
/// }
/// ```
pub fn model_attribute(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);

    // Parse the model attributes
    let name = &input.ident;
    let attrs = &input.attrs;
    let vis = &input.vis;
    let generics = &input.generics;

    // Parse table attributes to determine what system fields to add
    let table_info = match parse_table_attributes(&input.attrs) {
        Ok(info) => info,
        Err(e) => return e.to_compile_error().into(),
    };

    // Extract original fields from the struct
    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields_named) => fields_named.clone(),
            _ => panic!("model can only be used on structs with named fields"),
        },
        _ => panic!("model can only be used on structs"),
    };

    // Add system fields based on table configuration
    let mut system_fields = Vec::new();

    // Always add __created_at__ and __updated_at__ (readonly)
    system_fields.push(quote! {
        #[readonly]
        pub __created_at__: chrono::DateTime<chrono::Utc>
    });
    system_fields.push(quote! {
        #[readonly]
        pub __updated_at__: chrono::DateTime<chrono::Utc>
    });

    // Always add __tags__ (editable)
    system_fields.push(quote! {
        #[field(update)]
        pub __tags__: Option<Vec<String>>
    });

    // Add __is_active__ if auto_soft_delete is enabled (editable)
    if table_info.auto_soft_delete {
        system_fields.push(quote! {
            #[field(update)]
            pub __is_active__: bool
        });
    }

    // Combine original fields with system fields
    let fields_vec: Vec<_> = fields.named.iter().collect();

    // Generate new() method parameters - only user-defined fields
    let new_params: Vec<_> = fields
        .named
        .iter()
        .map(|field| {
            let name = &field.ident;
            let ty = &field.ty;
            quote! { #name: #ty }
        })
        .collect();

    // Generate new() method field assignments - user fields + system fields
    let user_field_assignments: Vec<_> = fields
        .named
        .iter()
        .map(|field| {
            let name = &field.ident;
            quote! { #name }
        })
        .collect();

    // Generate system field default values
    let mut system_field_assignments = vec![
        quote! { __created_at__: Default::default() },
        quote! { __updated_at__: Default::default() },
        quote! { __tags__: None },
    ];

    if table_info.auto_soft_delete {
        system_field_assignments.push(quote! { __is_active__: true });
    }

    let expanded = quote! {
        #[derive(Debug, Clone, serde::Serialize, serde::Deserialize, sqlx::FromRow, TableMetadata)]
        #(#attrs)*
        #vis struct #name #generics {
            #(#fields_vec),*,
            #(#system_fields),*
        }

        impl #generics #name #generics {
            /// Create a new instance with automatic system field initialization
            pub fn new(#(#new_params),*) -> Self {
                Self {
                    #(#user_field_assignments),*,
                    #(#system_field_assignments),*
                }
            }
        }
    };

    TokenStream::from(expanded)
}
