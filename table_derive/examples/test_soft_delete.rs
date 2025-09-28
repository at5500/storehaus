
use storehaus::prelude::*;
// use table_derive::{model, TableMetadata};

#[model]
#[table(name = "test_soft_delete", auto_soft_delete)]
pub struct TestSoftDeleteModel {
    #[primary_key]
    pub id: Uuid,

    #[field(create, update)]
    pub name: String,
}

fn main() {
    println!("Table name: {}", TestSoftDeleteModel::table_name());
    println!("Create fields: {:?}", TestSoftDeleteModel::create_fields());
    println!("Update fields: {:?}", TestSoftDeleteModel::update_fields());
    println!(
        "Supports soft delete: {}",
        TestSoftDeleteModel::supports_soft_delete()
    );
    println!(
        "Soft delete field: {:?}",
        TestSoftDeleteModel::soft_delete_field()
    );
}
