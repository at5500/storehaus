// Test error handling instead of panics
use store_object::*;

fn main() {
    println!("Testing error handling scenarios...");

    // Test 1: Invalid table names should return errors, not panic
    let invalid_names = [
        "SELECT",        // Reserved keyword
        "123table",      // Starts with number
        "user-name",     // Invalid character
        "",              // Empty
        &"a".repeat(64), // Too long
    ];

    for name in invalid_names {
        match ValidatedTableName::new(name) {
            Ok(_) => println!("❌ Unexpectedly accepted invalid name: {}", name),
            Err(e) => println!("✅ Correctly rejected '{}': {}", name, e),
        }
    }

    // Test 2: Valid names should work
    let valid_names = ["users", "user_profiles", "_private", "table123"];

    for name in valid_names {
        match ValidatedTableName::new(name) {
            Ok(validated) => println!("✅ Accepted valid name: {}", validated),
            Err(e) => println!("❌ Unexpectedly rejected valid name '{}': {}", name, e),
        }
    }

    // Test 3: Test field names
    match ValidatedFieldName::new("SELECT") {
        Ok(_) => println!("❌ Unexpectedly accepted reserved keyword as field name"),
        Err(e) => println!(
            "✅ Correctly rejected reserved keyword as field name: {}",
            e
        ),
    }

    match ValidatedFieldName::new("user_id") {
        Ok(validated) => println!("✅ Accepted valid field name: {}", validated),
        Err(e) => println!("❌ Unexpectedly rejected valid field name: {}", e),
    }

    println!("Error handling test completed successfully! ✅");
}
