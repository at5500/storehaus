//! # Basic Usage Example
//!
//! This example demonstrates the fundamental concepts of StoreHaus:
//! - Defining models with the `#[model]` macro
//! - Using the convenient `Model::new()` method
//! - Basic CRUD operations (Create, Read, Update, Delete)
//! - Working with StoreHaus coordination
//!
//! This is the perfect starting point for new users.

use storehaus::prelude::*;

/// A simple user model demonstrating basic field types
#[model]
#[table(name = "users")]
pub struct User {
    /// Primary key field - required for all models
    #[primary_key]
    pub id: Uuid,

    /// Basic string field that can be created and updated
    #[field(create, update)]
    pub name: String,

    /// Email field with unique constraint in a real application
    #[field(create, update)]
    pub email: String,

    /// Optional field - demonstrates nullable columns
    #[field(create, update)]
    pub phone: Option<String>,

    /// Age field demonstrating integer types
    #[field(create, update)]
    pub age: Option<i32>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 StoreHaus Basic Usage Example");
    println!("=================================");

    // 1. Setup Database Connection
    println!("\n📊 Step 1: Database Setup");
    println!("--------------------------");

    let config = DatabaseConfig::new(
        "localhost".to_string(),
        5432,
        "storehaus".to_string(),
        "postgres".to_string(),
        "password".to_string(),
        1,    // min_connections
        5,    // max_connections
        30,   // connection_timeout_seconds
        600,  // idle_timeout_seconds
        3600, // max_lifetime_seconds
    );

    let mut storehaus = StoreHaus::new(config).await?;
    println!("✅ Connected to PostgreSQL database");

    // 2. Auto-migrate the table
    storehaus.auto_migrate::<User>(true).await?;
    println!("✅ Auto-migrated 'users' table");

    // 3. Create a store for User operations
    let store = GenericStore::<User>::new(
        storehaus.pool().clone(),
        None, // No signals for this basic example
        None, // No caching for this basic example
    );

    storehaus.register_store("users".to_string(), store)?;
    let user_store = storehaus.get_store::<GenericStore<User>>("users")?;

    println!("✅ User store configured and ready");

    // 4. CREATE - Demonstrate the new() method
    println!("\n📝 Step 2: Creating Records");
    println!("----------------------------");

    // ✨ NEW: Using the convenient new() method!
    // No need to specify system fields (__created_at__, __updated_at__, __tags__)
    let user1 = User::new(
        Uuid::new_v4(),
        "Alice Johnson".to_string(),
        "alice@example.com".to_string(),
        Some("+1-555-0101".to_string()),
        Some(28),
    );

    let user2 = User::new(
        Uuid::new_v4(),
        "Bob Smith".to_string(),
        "bob@example.com".to_string(),
        None, // No phone number
        Some(35),
    );

    let user3 = User::new(
        Uuid::new_v4(),
        "Carol Williams".to_string(),
        "carol@example.com".to_string(),
        Some("+1-555-0103".to_string()),
        None, // Age not provided
    );

    // Create records in the database
    let created_user1 = user_store.create(user1, None).await?;
    let created_user2 = user_store.create(user2, None).await?;
    let created_user3 = user_store.create(user3, None).await?;

    println!("✅ Created user: {} ({})", created_user1.name, created_user1.email);
    println!("✅ Created user: {} ({})", created_user2.name, created_user2.email);
    println!("✅ Created user: {} ({})", created_user3.name, created_user3.email);

    // 5. READ - Demonstrate different ways to retrieve data
    println!("\n📖 Step 3: Reading Records");
    println!("--------------------------");

    // Get by ID
    let fetched_user = user_store.get_by_id(&created_user1.id).await?;
    match fetched_user {
        Some(user) => println!("📋 Found user by ID: {} (age: {:?})", user.name, user.age),
        None => println!("❌ User not found"),
    }

    // Get all users
    let all_users = user_store.list_all().await?;
    println!("📋 Total users in database: {}", all_users.len());

    for user in &all_users {
        println!("  • {}: {} (phone: {:?})",
            user.name, user.email, user.phone);
    }

    // 6. UPDATE - Modify existing records
    println!("\n✏️  Step 4: Updating Records");
    println!("----------------------------");

    // Update user's phone number
    let mut updated_user = created_user2.clone();
    updated_user.phone = Some("+1-555-0102".to_string());
    updated_user.age = Some(36); // Birthday!

    let result = user_store.update(&created_user2.id, updated_user, None).await?;
    println!("✅ Updated user: {} (new phone: {:?})", result.name, result.phone);

    // 7. COUNT - Get total records
    println!("\n🔢 Step 5: Counting Records");
    println!("---------------------------");

    let total_count = user_store.count().await?;
    println!("📊 Total users: {}", total_count);

    // 8. DELETE - Remove records
    println!("\n🗑️  Step 6: Deleting Records");
    println!("----------------------------");

    let was_deleted = user_store.delete(&created_user3.id).await?;
    if was_deleted {
        println!("✅ Deleted user: {}", created_user3.name);
    } else {
        println!("❌ Failed to delete user");
    }

    // Verify deletion
    let remaining_count = user_store.count().await?;
    println!("📊 Remaining users: {}", remaining_count);

    // 9. System Fields - Show automatic fields
    println!("\n⚙️  Step 7: System Fields");
    println!("-------------------------");

    let remaining_users = user_store.list_all().await?;
    for user in remaining_users {
        println!("👤 {}: created at {:?}", user.name, user.__created_at__);
        println!("   Updated at: {:?}", user.__updated_at__);
        println!("   Tags: {:?}", user.__tags__);
    }

    println!("\n🎉 Basic Usage Demo Complete!");
    println!("==============================");
    println!("\n🎯 Key Takeaways:");
    println!("✅ Models are easy to define with #[model]");
    println!("✅ User::new() provides a clean API - no system fields needed");
    println!("✅ CRUD operations are straightforward");
    println!("✅ System fields (__created_at__, __updated_at__, __tags__) are automatic");
    println!("✅ StoreHaus manages database connections and stores");

    println!("\n📚 Next Steps:");
    println!("  • Try 02_model_definition.rs for advanced model features");
    println!("  • Explore advanced_querying.rs for complex queries");
    println!("  • Check system_fields_demo.rs to understand automatic fields");

    Ok(())
}