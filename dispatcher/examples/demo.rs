use dispatcher::{DatabaseConfig, Dispatcher};
use store_object::{generic_store::{GenericStore, CacheParams}, TableMetadata, StoreObject, QueryBuilder, QueryFilter, SortOrder};
use table_derive::model;
use signal_system::DatabaseEvent;
use cache_system::{CacheConfig, CacheManager};
use uuid::Uuid;
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

    #[field(readonly)]
    pub created_at: chrono::DateTime<chrono::Utc>,

    #[field(readonly)]
    pub updated_at: chrono::DateTime<chrono::Utc>,

    #[soft_delete]
    pub is_active: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Storehaus Demo\n");

    // Database setup
    let config = DatabaseConfig::new(
        "localhost".to_string(),
        5432,
        "storehaus".to_string(),
        "postgres".to_string(),
        "password".to_string(),
    ).with_max_connections(5);

    let mut dispatcher = Dispatcher::new(config).await?;
    dispatcher.health_check().await?;
    println!("✅ Database connected");

    // Signals setup
    let signal_manager = Arc::new(signal_system::SignalManager::new());
    signal_manager.add_callback(|event: &DatabaseEvent| {
        println!("🔔 {} on {}: {:?}",
            match event.event_type {
                signal_system::EventType::Create => "Created",
                signal_system::EventType::Update => "Updated",
                signal_system::EventType::Delete => "Deleted",
            },
            event.table_name,
            event.record_id.as_ref().unwrap_or(&"batch".to_string())
        );
    });
    println!("✅ Signals setup");

    // Cache setup (optional)
    let cache_manager = match setup_cache().await {
        Ok(manager) => {
            println!("✅ Cache connected");
            Some(manager)
        },
        Err(_) => {
            println!("⚠️  Cache unavailable, continuing without it");
            None
        }
    };

    // Auto-migrate and register store
    dispatcher.auto_migrate::<User>(false).await?;

    let cache_params = cache_manager.clone().map(|cm| {
        CacheParams::new(cm)
            .with_ttl(900)
            .with_prefix("users".to_string())
    });

    let user_store = GenericStore::<User>::new(
        dispatcher.pool().clone(),
        Some(signal_manager.clone()),
        cache_params,
    );

    dispatcher.register_store("users".to_string(), user_store)?;
    let user_store = dispatcher.get_store::<GenericStore<User>>("users")?;

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
    let user = User {
        id: Uuid::new_v4(),
        name: "John Doe".to_string(),
        email: format!("john-{}@demo.com", Uuid::new_v4()),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        is_active: true,
    };

    let created_user = user_store.create(user).await?;
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
    let updated_user = user_store.update(&created_user.id, updated).await?;
    println!("Updated: {}", updated_user.name);

    // Create more users for queries
    let test_users = vec![
        User {
            id: Uuid::new_v4(),
            name: "Alice Johnson".to_string(),
            email: "alice@demo.com".to_string(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            is_active: true,
        },
        User {
            id: Uuid::new_v4(),
            name: "Bob Wilson".to_string(),
            email: "bob@demo.com".to_string(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            is_active: false,
        },
    ];

    let mut test_ids = Vec::new();
    for user in test_users {
        let created = user_store.create(user).await?;
        println!("Created test user: {}", created.name);
        test_ids.push(created.id);
    }

    println!("\n=== Query Operations ===");

    // Find active users
    let active_query = QueryBuilder::new()
        .filter(QueryFilter::eq("is_active", json!(true)))
        .order_by("name", SortOrder::Asc);

    let active_users = user_store.find(active_query).await?;
    println!("Active users: {}", active_users.len());

    // Pattern search
    let name_query = QueryBuilder::new()
        .filter(QueryFilter::like("name", "%John%"));

    let john_users = user_store.find(name_query).await?;
    println!("Users with 'John': {}", john_users.len());

    // Count total
    let total = user_store.count().await?;
    println!("Total users: {}", total);

    println!("\n=== Batch Operations ===");

    // Batch update
    let batch_updates = test_ids.iter().enumerate().map(|(i, id)| {
        let user = User {
            id: *id,
            name: format!("Updated User {}", i + 1),
            email: format!("updated-{}@demo.com", i + 1),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            is_active: true,
        };
        (*id, user)
    }).take(2).collect();

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

    dispatcher.health_check().await?;
    println!("\n🎉 Demo completed successfully!");

    Ok(())
}

async fn setup_cache() -> Result<Arc<CacheManager>, Box<dyn std::error::Error>> {
    let config = CacheConfig::new(
        "redis://localhost:6379".to_string(),
        1800,
        "demo".to_string(),
    );

    let manager = Arc::new(CacheManager::new(config)?);
    manager.ping().await?;
    Ok(manager)
}