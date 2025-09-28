use storehaus::prelude::*;

#[model(soft)]
#[table(name = "users")]
pub struct User {
    #[primary_key]
    pub id: Uuid,

    #[field(create, update)]
    pub name: String,

    #[field(create, update)]
    pub email: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🏷️  StoreHaus Tags Demo");
    println!("=====================");

    // Setup database configuration
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

    // Create StoreHaus instance
    let mut storehaus = StoreHaus::new(config).await?;

    // Setup signal manager to capture tagged operations
    let signal_config = SignalConfig::new(
        30,        // callback_timeout_seconds
        1000,      // max_callbacks
        true,      // remove_failing_callbacks
        3,         // max_consecutive_failures
        60,        // cleanup_interval_seconds
        true,      // auto_remove_inactive_callbacks
        300,       // inactive_callback_threshold_seconds
    );
    let signal_manager = SignalManager::new(signal_config);

    // Add callback to log tagged operations
    signal_manager
        .add_callback(|event: DatabaseEvent| async move {
            if !event.tags.is_empty() {
                println!(
                    "📡 Tagged operation: {:?} on {} with tags: {:?}",
                    event.event_type, event.table_name, event.tags
                );
            }
            Ok(())
        })
        .await?;

    // Auto-migrate table (will create __tags__ column)
    println!("🔄 Running auto-migration...");
    storehaus.auto_migrate::<User>(true).await?;

    // Create store with signal manager
    let user_store = GenericStore::<User>::new(
        storehaus.pool().clone(),
        Some(signal_manager.clone()),
        None,
    );

    storehaus.register_store("users".to_string(), user_store)?;
    let user_store = storehaus.get_store::<GenericStore<User>>("users")?;

    println!("✅ Setup complete!\n");

    // Create users with different tags
    println!("👤 Creating users with tags...");

    let user1 = User::new(
        Uuid::new_v4(),
        "Alice Smith".to_string(),
        "alice@example.com".to_string(),
    );

    let user2 = User::new(
        Uuid::new_v4(),
        "Bob Johnson".to_string(),
        "bob@example.com".to_string(),
    );

    let user3 = User::new(
        Uuid::new_v4(),
        "Carol Davis".to_string(),
        "carol@example.com".to_string(),
    );

    // Create users with tags (this will trigger signals with tags)
    let _created_user1 = user_store
        .create(
            user1,
            Some(vec![
                "developer".to_string(),
                "backend".to_string(),
                "team-alpha".to_string(),
            ]),
        )
        .await?;
    let created_user2 = user_store
        .create(
            user2,
            Some(vec![
                "developer".to_string(),
                "frontend".to_string(),
                "team-beta".to_string(),
            ]),
        )
        .await?;
    let created_user3 = user_store
        .create(
            user3,
            Some(vec!["manager".to_string(), "team-alpha".to_string()]),
        )
        .await?;

    println!("✅ Created {} users\n", 3);

    println!("🔍 Tags are now system fields and work through signals!");
    println!("📊 All create and update operations with tags have been logged via signals.");

    // For now, let's just show that regular queries work
    let all_users = user_store.list_all().await?;
    println!("\n📋 All users in database ({} total):", all_users.len());
    for user in all_users {
        println!("  - {} ({})", user.name, user.email);
    }

    // Update a user's tags
    println!("\n🔄 Updating user tags...");
    let updated_user = created_user2.clone();

    let updated = user_store
        .update(
            &created_user2.id,
            updated_user,
            Some(vec![
                "developer".to_string(),
                "frontend".to_string(),
                "team-beta".to_string(),
                "lead".to_string(),
            ]),
        )
        .await?;
    println!(
        "✅ Updated {} with new tags (including 'lead')",
        updated.name
    );

    // Delete a user (will also be tagged in signals)
    println!("\n🗑️  Deleting user...");
    let deleted = user_store.delete(&created_user3.id).await?;
    if deleted {
        println!("✅ Deleted Carol (manager, team-alpha)");
    }

    println!("\n🎉 Tags demo completed!");
    println!("📊 Summary:");
    println!(
        "  - Created 3 users with different tag combinations using create() with optional tags"
    );
    println!("  - Updated user tags using update() with optional tags");
    println!("  - All operations were captured in signals with tag information");
    println!("  - Tags and __is_active__ are now system fields (not part of model definition)");
    println!("  - Database has __tags__ column ready for future tag storage");
    println!("  - Signals contain all tag information for WAL or other systems");
    println!("  - Using #[model(soft)] attribute for automatic soft delete support");

    Ok(())
}
