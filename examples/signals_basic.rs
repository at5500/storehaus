//! # Basic Signal System Example
//!
//! This example introduces the StoreHaus signal system:
//! - Setting up signal callbacks
//! - Handling database events (Create, Update, Delete)
//! - Basic error handling in callbacks
//! - Understanding event data structure

use storehaus::prelude::*;

/// Simple user model for signal demonstrations
#[model]
#[table(name = "users")]
pub struct User {
    #[primary_key]
    pub id: Uuid,

    #[field(create, update)]
    pub name: String,

    #[field(create, update)]
    pub email: String,

    #[field(create, update)]
    pub status: String,
}

/// Product model to show cross-table signals
#[model]
#[table(name = "products")]
pub struct Product {
    #[primary_key]
    pub id: Uuid,

    #[field(create, update)]
    pub name: String,

    #[field(create, update)]
    pub price: i32,

    #[field(create, update)]
    pub stock: i32,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("⚡ StoreHaus Basic Signal System");
    println!("===============================");

    // 1. Database Setup
    let config = DatabaseConfig::new(
        "localhost".to_string(), 5432, "storehaus".to_string(),
        "postgres".to_string(), "password".to_string(),
        1, 5, 30, 600, 3600,
    );

    let mut storehaus = StoreHaus::new(config).await?;
    storehaus.auto_migrate::<User>(true).await?;
    storehaus.auto_migrate::<Product>(true).await?;

    // 2. Signal System Setup
    println!("\n🔧 Setting up Signal System");
    println!("---------------------------");

    let signal_config = SignalConfig::new(
        30,   // callback_timeout_seconds
        100,  // max_callbacks
        true, // remove_failing_callbacks
        3,    // max_consecutive_failures
        60,   // cleanup_interval_seconds
        true, // auto_remove_inactive_callbacks
        300,  // inactive_callback_threshold_seconds
    );
    let signal_manager = SignalManager::new(signal_config);

    println!("✅ Signal manager configured");

    // 3. Basic Event Logging Callback
    println!("\n📋 Setting up event logging...");

    signal_manager
        .add_callback(|event: DatabaseEvent| async move {
            println!(
                "📡 EVENT: {:?} on '{}' - Record ID: {:?}",
                event.event_type,
                event.table_name,
                event.record_id
            );

            // Show event timestamp
            println!(
                "   ⏰ Timestamp: {}",
                event.timestamp.format("%Y-%m-%d %H:%M:%S%.3f")
            );

            // Show tags if any
            if !event.tags.is_empty() {
                println!("   🏷️  Tags: {:?}", event.tags);
            }

            Ok(())
        })
        .await?;

    // 4. Create-specific Callback
    println!("📝 Setting up create event handler...");

    signal_manager
        .add_callback(|event: DatabaseEvent| async move {
            if matches!(event.event_type, EventType::Create) {
                println!("✨ NEW RECORD CREATED!");
                println!("   Table: {}", event.table_name);

                // Access payload data
                if !event.payload.is_empty() {
                    println!("   📦 Payload fields: {}", event.payload.len());
                    for key in event.payload.keys() {
                        println!("     • {}", key);
                    }
                }

                // Table-specific logic
                match event.table_name.as_str() {
                    "users" => {
                        println!("   🎉 Welcome new user!");
                        // In a real app: send welcome email, create user profile, etc.
                    },
                    "products" => {
                        println!("   🛍️  New product added to catalog!");
                        // In a real app: update search index, notify subscribers, etc.
                    },
                    _ => {}
                }
            }
            Ok(())
        })
        .await?;

    // 5. Update-specific Callback
    println!("📝 Setting up update event handler...");

    signal_manager
        .add_callback(|event: DatabaseEvent| async move {
            if matches!(event.event_type, EventType::Update) {
                println!("📝 RECORD UPDATED!");
                println!("   Table: {}", event.table_name);

                // Example: Track important status changes
                if event.table_name == "users" {
                    if let Some(status) = event.payload.get("status") {
                        println!("   👤 User status changed: {:?}", status);
                        // In a real app: send notifications, update permissions, etc.
                    }
                }

                if event.table_name == "products" {
                    if let Some(stock) = event.payload.get("stock") {
                        println!("   📦 Product stock updated: {:?}", stock);
                        // In a real app: check low stock alerts, update inventory, etc.
                    }
                }
            }
            Ok(())
        })
        .await?;

    // 6. Delete-specific Callback
    println!("📝 Setting up delete event handler...");

    signal_manager
        .add_callback(|event: DatabaseEvent| async move {
            if matches!(event.event_type, EventType::Delete) {
                println!("🗑️  RECORD DELETED!");
                println!("   Table: {}", event.table_name);

                // Cleanup logic
                match event.table_name.as_str() {
                    "users" => {
                        println!("   🧹 Cleaning up user data...");
                        // In a real app: remove from mailing lists, anonymize data, etc.
                    },
                    "products" => {
                        println!("   📋 Removing from catalog...");
                        // In a real app: update search index, handle pending orders, etc.
                    },
                    _ => {}
                }
            }
            Ok(())
        })
        .await?;

    println!("✅ All signal callbacks registered");

    // 7. Create Stores with Signal Manager
    let user_store = GenericStore::<User>::new(
        storehaus.pool().clone(),
        Some(signal_manager.clone()),
        None,
    );

    let product_store = GenericStore::<Product>::new(
        storehaus.pool().clone(),
        Some(signal_manager.clone()),
        None,
    );

    storehaus.register_store("users".to_string(), user_store)?;
    storehaus.register_store("products".to_string(), product_store)?;

    let user_store = storehaus.get_store::<GenericStore<User>>("users")?;
    let product_store = storehaus.get_store::<GenericStore<Product>>("products")?;

    // 8. Demonstrate Signals in Action
    println!("\n🎬 Triggering Events");
    println!("===================");

    // CREATE events
    println!("\n1️⃣ Creating users (watch for Create signals)...");

    let user1 = User::new(
        Uuid::new_v4(),
        "Alice Johnson".to_string(),
        "alice@example.com".to_string(),
        "active".to_string(),
    );

    let user2 = User::new(
        Uuid::new_v4(),
        "Bob Smith".to_string(),
        "bob@example.com".to_string(),
        "pending".to_string(),
    );

    let created_user1 = user_store.create(user1, Some(vec!["onboarding".to_string()])).await?;
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    let created_user2 = user_store.create(user2, Some(vec!["onboarding".to_string()])).await?;
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // CREATE events for products
    println!("\n2️⃣ Creating products (watch for Create signals)...");

    let product1 = Product::new(
        Uuid::new_v4(),
        "Wireless Mouse".to_string(),
        2999, // $29.99
        50,
    );

    let _created_product = product_store.create(product1, Some(vec!["new-arrival".to_string()])).await?;
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // UPDATE events
    println!("\n3️⃣ Updating records (watch for Update signals)...");

    let mut updated_user = created_user2.clone();
    updated_user.status = "active".to_string();

    let _updated = user_store.update(
        &created_user2.id,
        updated_user,
        Some(vec!["status-change".to_string()]),
    ).await?;

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // DELETE events
    println!("\n4️⃣ Deleting records (watch for Delete signals)...");

    let _deleted = user_store.delete(&created_user1.id).await?;
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // 9. Show Signal Statistics
    println!("\n📊 Signal System Statistics");
    println!("---------------------------");

    let callback_count = signal_manager.callback_count().await;
    println!("Active callbacks: {}", callback_count);

    println!("\n🎉 Basic Signal System Demo Complete!");
    println!("====================================");

    println!("\n🎯 Key Concepts:");
    println!("✅ Signals are fired automatically on database operations");
    println!("✅ Callbacks run asynchronously and don't block operations");
    println!("✅ Event data includes type, table, record ID, tags, and payload");
    println!("✅ Multiple callbacks can handle the same event");
    println!("✅ Callbacks can implement business logic and side effects");

    println!("\n🔧 Event Types:");
    println!("  • Create: New records added to database");
    println!("  • Update: Existing records modified");
    println!("  • Delete: Records removed from database");

    println!("\n📦 Event Data:");
    println!("  • event_type: Create/Update/Delete");
    println!("  • table_name: Which table was affected");
    println!("  • record_id: ID of the affected record");
    println!("  • tags: Custom tags for operation categorization");
    println!("  • payload: Field data that was changed");
    println!("  • timestamp: When the event occurred");

    println!("\n📚 Next Steps:");
    println!("  • Try signals_advanced.rs for complex async workflows");
    println!("  • Explore error handling and retry mechanisms");
    println!("  • Learn about performance monitoring with signals");

    Ok(())
}