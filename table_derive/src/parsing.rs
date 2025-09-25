use std::collections::HashMap;
use syn::{Attribute, Data, Fields, Ident, Meta};
use quote::quote;

#[derive(Debug)]
pub struct TableAttributes {
    pub name: String,
}

#[derive(Debug)]
pub struct FieldInfo {
    pub primary_key_field: Ident,
    pub primary_key_type: String,
    pub create_fields: Vec<String>,
    pub update_fields: Vec<String>,
    pub has_soft_delete: bool,
    pub has_auto_increment: bool,
    pub auto_increment_field: Option<String>,
    pub field_types: HashMap<String, String>, // field_name -> rust_type
}

pub fn parse_table_attributes(attrs: &[Attribute]) -> TableAttributes {
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
                    }
                }

                return TableAttributes {
                    name: name.expect("table name is required"),
                };
            }
        }
    }

    panic!("table attribute is required");
}

pub fn parse_field_attributes(data: &Data) -> FieldInfo {
    if let Data::Struct(data_struct) = data {
        if let Fields::Named(fields_named) = &data_struct.fields {
            let mut primary_key_field = None;
            let mut primary_key_type = None;
            let mut create_fields = Vec::new();
            let mut update_fields = Vec::new();
            let mut has_soft_delete = false;
            let mut has_auto_increment = false;
            let mut auto_increment_field = None;
            let mut field_types = HashMap::new();

            for field in &fields_named.named {
                let field_name = field.ident.as_ref().unwrap();
                let field_name_str = field_name.to_string();
                let ty = &field.ty;
                let type_string = quote!(#ty).to_string();

                // Store field type
                field_types.insert(field_name_str.clone(), type_string.clone());

                // Check for primary_key attribute
                if has_attribute(&field.attrs, "primary_key") {
                    primary_key_field = Some(field_name.clone());
                    primary_key_type = Some(type_string);
                }

                // Check for soft_delete attribute
                if has_attribute(&field.attrs, "soft_delete") {
                    has_soft_delete = true;
                }

                // Check for auto_increment attribute
                if has_attribute(&field.attrs, "auto_increment") {
                    has_auto_increment = true;
                    auto_increment_field = Some(field_name_str.clone());
                }

                // Check for field attributes
                if let Some(field_ops) = parse_field_operations(&field.attrs) {
                    if field_ops.contains(&"create".to_string()) {
                        create_fields.push(field_name_str.clone());
                    }
                    if field_ops.contains(&"update".to_string()) {
                        update_fields.push(field_name_str);
                    }
                }
            }

            return FieldInfo {
                primary_key_field: primary_key_field.expect("primary_key field is required"),
                primary_key_type: primary_key_type.expect("primary_key type is required"),
                create_fields,
                update_fields,
                has_soft_delete,
                has_auto_increment,
                auto_increment_field,
                field_types,
            };
        }
    }

    panic!("TableMetadata can only be derived for structs with named fields");
}

pub fn has_attribute(attrs: &[Attribute], name: &str) -> bool {
    attrs.iter().any(|attr| attr.path().is_ident(name))
}

pub fn parse_field_operations(attrs: &[Attribute]) -> Option<Vec<String>> {
    for attr in attrs {
        if attr.path().is_ident("field") {
            // Simplified parsing for field operations
            let attr_str = quote!(#attr).to_string();
            let mut operations = Vec::new();

            if attr_str.contains("create") {
                operations.push("create".to_string());
            }
            if attr_str.contains("update") {
                operations.push("update".to_string());
            }
            if attr_str.contains("readonly") {
                // readonly fields are not included in create/update
            }

            return Some(operations);
        }
    }

    None
}
