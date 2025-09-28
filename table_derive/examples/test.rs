use storehaus::prelude::*;

#[model]
#[table(name = "test")]
pub struct TestModel {
    #[primary_key]
    pub id: Uuid,

    #[field(create, update)]
    pub name: String,
}

fn main() {
    println!("Table name: {}", TestModel::table_name());
    println!("Create fields: {:?}", TestModel::create_fields());
    println!("Update fields: {:?}", TestModel::update_fields());
}
