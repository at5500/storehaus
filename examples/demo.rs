use storehaus::prelude::*;
use serde_json::json;
use std::sync::Arc;

#[model]
#[table(name = "users")]
pub struct User {
    #[primary_key]
    pub id: Uuid,

    #[field(create, update)]
    pub name: String,

    #[field(create, update)]
    pub email: String,

    #[soft_delete]
    pub __is_active__: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Storehaus Demo\n");

    // Database setup
    let config = DatabaseConfig::new(
        "localhost".to_string(), // host
        5432,                    // port
        "storehaus".to_string(), // database
        "postgres".to_string(),  // username
        "password".to_string(),  // password
        1,                       // min_connections
        5,                       // max_connections
        30,                      // connection_timeout_seconds
        600,                     // idle_timeout_seconds
        3600,              // max_lifetime_seconds
    );

    let mut storehaus = StoreHaus::new(config).await?;
    storehaus.health_check().await?;
    println!("✅ Database connected");

    // Signals setup
    let signal_config = SignalConfig::new(
        30,        // callback_timeout_seconds
        100,       // max_callbacks
        true,      // remove_failing_callbacks
        3,         // max_consecutive_failures
        60,        // cleanup_interval_seconds
        true,      // auto_remove_inactive_callbacks
        300,       // inactive_callback_threshold_seconds
    );
    let signal_manager = SignalManager::new(signal_config);
    signal_manager
        .add_callback(|event: DatabaseEvent| async move {
            println!(
                "🔔 {} on {}: {:?}",
                match event.event_type {
                    EventType::Create => "Created",
                    EventType::Update => "Updated",
                    EventType::Delete => "Deleted",
                },
                event.table_name,
                event.record_id.as_ref().unwrap_or(&"batch".to_string())
            );
            Ok(())
        })
        .await?;
    println!("✅ Signals setup");

    // Cache setup (optional)
    let cache_manager = match setup_cache().await {
        Ok(manager) => {
            println!("✅ Cache connected");
            Some(manager)
        }
        Err(_) => {
            println!("⚠️  Cache unavailable, continuing without it");
            None
        }
    };

    // Auto-migrate and register store
    storehaus.auto_migrate::<User>(false).await?;

    let cache_params = cache_manager
        .clone()
        .map(|cm| CacheParams::new(cm, 900, "users"));

    let user_store = GenericStore::<User>::new(
        storehaus.pool().clone(),
        Some(signal_manager.clone()),
        cache_params,
    );

    storehaus.register_store("users".to_string(), user_store)?;
    let user_store = storehaus.get_store::<GenericStore<User>>("users")?;

    // Clean up any existing demo data
    let existing_count = user_store.count().await?;
    if existing_count > 0 {
        println!("🧹 Cleaning {} existing records...", existing_count);
        let existing_users = user_store.list_all().await?;
        let existing_ids: Vec<Uuid> = existing_users.iter().map(|u| u.id).collect();
        if !existing_ids.is_empty() {
            user_store.delete_many(existing_ids).await?;
        }
    }

    println!("✅ Store registered\n");

    // Demo operations
    println!("=== CRUD Operations ===");

    // Create user
    let user = User::new(
        Uuid::new_v4(),
        "John Doe".to_string(),
        format!("john-{}@demo.com", Uuid::new_v4()),
         true,
    );

    let created_user = user_store.create(user, None).await?;
    println!("Created: {} ({})", created_user.name, created_user.id);

    // Test cache performance if available
    if cache_manager.is_some() {
        let start = std::time::Instant::now();
        let _ = user_store.get_by_id(&created_user.id).await?;
        let first = start.elapsed();

        let start = std::time::Instant::now();
        let _ = user_store.get_by_id(&created_user.id).await?;
        let second = start.elapsed();

        println!("Cache test - First: {:?}, Second: {:?}", first, second);
        if first > second {
            println!("✅ Cache working!");
        }
    }

    // Update user
    let mut updated = created_user.clone();
    updated.name = "John Smith".to_string();
    let updated_user = user_store.update(&created_user.id, updated, None).await?;
    println!("Updated: {}", updated_user.name);

    // Create more users for queries
    let test_users = vec![
        User::new(
            Uuid::new_v4(),
            "Alice Johnson".to_string(),
            "alice@demo.com".to_string(),
            true,
        ),
        User::new(
            Uuid::new_v4(),
            "Bob Wilson".to_string(),
            "bob@demo.com".to_string(),
            false,
        ),
    ];

    let mut test_ids = Vec::new();
    for user in test_users {
        let created = user_store.create(user, None).await?;
        println!("Created test user: {}", created.name);
        test_ids.push(created.id);
    }

    println!("\n=== Query Operations ===");

    // Find active users
    let active_query = QueryBuilder::new()
        .filter(QueryFilter::eq("__is_active__", json!(true)))
        .order_by("name", SortOrder::Asc);

    let active_users = user_store.find(active_query).await?;
    println!("Active users: {}", active_users.len());

    // Pattern search
    let name_query = QueryBuilder::new().filter(QueryFilter::like("name", "%John%"));

    let john_users = user_store.find(name_query).await?;
    println!("Users with 'John': {}", john_users.len());

    // Count total
    let total = user_store.count().await?;
    println!("Total users: {}", total);

    println!("\n=== Batch Operations ===");

    // Batch update
    let batch_updates = test_ids
        .iter()
        .enumerate()
        .map(|(i, id)| {
            let user = User::new(
                *id,
                format!("Updated User {}", i + 1),
                format!("updated-{}@demo.com", i + 1),
                true,
            );
            (*id, user)
        })
        .take(2)
        .collect();

    let updated_batch = user_store.update_many(batch_updates).await?;
    println!("Batch updated: {}", updated_batch.len());

    // Cleanup
    println!("\n=== Cleanup ===");

    // Delete test users
    let deleted_batch = user_store.delete_many(test_ids).await?;
    println!("Batch deleted: {}", deleted_batch.len());

    // Delete main user
    let deleted = user_store.delete(&created_user.id).await?;
    println!("Main user deleted: {}", deleted);

    storehaus.health_check().await?;
    println!("\n🎉 Demo completed successfully!");

    Ok(())
}

async fn setup_cache() -> Result<Arc<CacheManager>, Box<dyn std::error::Error>> {
    let config = CacheConfig::new(
        "redis://localhost:6379".to_string(),
        10,   // pool_size
        5000, // timeout_ms
        100,  // max_connections
        3000, // connection_timeout_ms
    );

    let manager = Arc::new(CacheManager::new(config)?);
    manager.ping().await?;
    Ok(manager)
}
