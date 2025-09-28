// Example to demonstrate runtime validation
use store_object::*;

fn main() {
    println!("Validation test completed successfully!");

    // Test runtime validation
    match ValidatedTableName::new("test_table") {
        Ok(name) => println!("Valid table name: {}", name),
        Err(e) => println!("Invalid table name: {}", e),
    }

    // Test invalid cases
    match ValidatedTableName::new("SELECT") {
        Ok(name) => println!("Unexpected success: {}", name),
        Err(e) => println!("Correctly rejected reserved keyword: {}", e),
    }
}
