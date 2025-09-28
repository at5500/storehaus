//! # Basic Caching Example
//!
//! This example demonstrates basic caching functionality:
//! - Setting up Redis cache
//! - Basic cache configuration
//! - Understanding cache hits vs misses
//! - Cache invalidation on updates

use storehaus::prelude::*;
use std::sync::Arc;
use std::time::Instant;

/// Simple product model for caching demonstration
#[model]
#[table(name = "products")]
pub struct Product {
    #[primary_key]
    pub id: Uuid,

    #[field(create, update)]
    pub name: String,

    #[field(create, update)]
    pub description: String,

    #[field(create, update)]
    pub price: i32,

    #[field(create, update)]
    pub category: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 StoreHaus Basic Caching Example");
    println!("==================================");

    // 1. Database Setup
    let db_config = DatabaseConfig::new(
        "localhost".to_string(), 5432, "storehaus".to_string(),
        "postgres".to_string(), "password".to_string(),
        1, 5, 30, 600, 3600,
    );

    let mut storehaus = StoreHaus::new(db_config).await?;
    storehaus.auto_migrate::<Product>(true).await?;
    println!("✅ Database connected and migrated");

    // 2. Cache Setup
    println!("\n🗄️  Setting up Redis Cache");
    println!("--------------------------");

    let cache_config = CacheConfig::new(
        "redis://localhost:6379".to_string(),
        10,    // pool_size
        5000,  // timeout_ms
        100,   // max_connections
        3000,  // connection_timeout_ms
    );

    let cache_manager = Arc::new(CacheManager::new(cache_config)?);

    // Test Redis connectivity
    match cache_manager.ping().await {
        Ok(_) => println!("✅ Redis connection healthy"),
        Err(e) => {
            println!("❌ Redis connection failed: {}", e);
            println!("💡 Please start Redis: docker run -d --name redis -p 6379:6379 redis:7-alpine");
            return Ok(());
        }
    }

    // 3. Create Stores (with and without cache)
    println!("\n🏪 Store Configuration");
    println!("----------------------");

    // Store WITHOUT cache (for comparison)
    let uncached_store = GenericStore::<Product>::new(
        storehaus.pool().clone(),
        None, // No signals
        None, // No cache
    );

    // Store WITH cache
    let cache_params = CacheParams::new(
        cache_manager.clone(),
        3600, // TTL: 1 hour
        "products", // Cache key prefix
    );

    let cached_store = GenericStore::<Product>::new(
        storehaus.pool().clone(),
        None,                   // No signals
        Some(cache_params),     // Enable cache
    );

    storehaus.register_store("products_uncached".to_string(), uncached_store)?;
    storehaus.register_store("products_cached".to_string(), cached_store)?;

    let uncached_store = storehaus.get_store::<GenericStore<Product>>("products_uncached")?;
    let cached_store = storehaus.get_store::<GenericStore<Product>>("products_cached")?;

    println!("✅ Stores configured:");
    println!("   • Uncached store: Direct database access");
    println!("   • Cached store: Redis cache with 1-hour TTL");

    // 4. Create Test Data
    println!("\n📦 Creating Test Data");
    println!("---------------------");

    let products = vec![
        Product::new(
            Uuid::new_v4(),
            "Wireless Mouse".to_string(),
            "Ergonomic wireless mouse with precision tracking".to_string(),
            2999, // $29.99
            "Electronics".to_string(),
        ),
        Product::new(
            Uuid::new_v4(),
            "Mechanical Keyboard".to_string(),
            "Cherry MX Blue switches with RGB backlighting".to_string(),
            12999, // $129.99
            "Electronics".to_string(),
        ),
        Product::new(
            Uuid::new_v4(),
            "Coffee Mug".to_string(),
            "Ceramic mug perfect for your favorite beverage".to_string(),
            1499, // $14.99
            "Kitchen".to_string(),
        ),
    ];

    let mut created_products = Vec::new();
    for product in products {
        let created = uncached_store.create(product, None).await?;
        created_products.push(created);
    }

    println!("✅ Created {} test products", created_products.len());

    // 5. Cache Performance Comparison
    println!("\n⚡ Performance Comparison");
    println!("========================");

    let test_product = &created_products[0];

    // Test uncached store (always hits database)
    println!("\n📊 Uncached Store Performance:");
    let start = Instant::now();
    let _result1 = uncached_store.get_by_id(&test_product.id).await?;
    let uncached_time1 = start.elapsed();

    let start = Instant::now();
    let _result2 = uncached_store.get_by_id(&test_product.id).await?;
    let uncached_time2 = start.elapsed();

    println!("  First fetch:  {:?} (database)", uncached_time1);
    println!("  Second fetch: {:?} (database)", uncached_time2);

    // Test cached store
    println!("\n💾 Cached Store Performance:");

    // First fetch (cache miss - goes to database)
    let start = Instant::now();
    let cached_result1 = cached_store.get_by_id(&test_product.id).await?;
    let cache_miss_time = start.elapsed();

    // Second fetch (cache hit - comes from Redis)
    let start = Instant::now();
    let cached_result2 = cached_store.get_by_id(&test_product.id).await?;
    let cache_hit_time = start.elapsed();

    println!("  First fetch:  {:?} (cache miss → database + cache)", cache_miss_time);
    println!("  Second fetch: {:?} (cache hit → Redis)", cache_hit_time);

    // Verify data consistency
    assert_eq!(cached_result1.as_ref().unwrap().name, cached_result2.as_ref().unwrap().name);
    println!("✅ Data consistency verified");

    // Calculate speedup
    if cache_hit_time.as_nanos() > 0 {
        let speedup = cache_miss_time.as_nanos() as f64 / cache_hit_time.as_nanos() as f64;
        println!("🚀 Cache hit is {:.1}x faster!", speedup);
    }

    // 6. Cache Invalidation Demo
    println!("\n🔄 Cache Invalidation");
    println!("====================");

    // Update the product (this should invalidate the cache)
    let mut updated_product = test_product.clone();
    updated_product.price = 2599; // Price drop!
    updated_product.description = "Ergonomic wireless mouse with precision tracking - NOW ON SALE!".to_string();

    println!("Updating product: '{}' price {} → {}",
        updated_product.name, test_product.price, updated_product.price);

    let _updated = cached_store.update(&test_product.id, updated_product, None).await?;

    // Fetch again - should get updated data (cache was invalidated)
    let start = Instant::now();
    let after_update = cached_store.get_by_id(&test_product.id).await?;
    let after_update_time = start.elapsed();

    println!("Fetch after update: {:?}", after_update_time);

    if let Some(product) = after_update {
        println!("✅ Updated product retrieved:");
        println!("   Name: {}", product.name);
        println!("   Price: ${:.2}", product.price as f64 / 100.0);
        println!("   Description: {}", product.description);
    }

    // 7. Multiple Record Caching
    println!("\n📋 Multiple Record Operations");
    println!("=============================");

    // Fetch multiple products to populate cache
    println!("Fetching all products to populate cache...");
    let start = Instant::now();

    for product in &created_products {
        let _ = cached_store.get_by_id(&product.id).await?;
    }
    let populate_time = start.elapsed();
    println!("Cache populated in: {:?}", populate_time);

    // Now fetch them all again (should be fast cache hits)
    println!("Fetching all products from cache...");
    let start = Instant::now();

    for product in &created_products {
        let _ = cached_store.get_by_id(&product.id).await?;
    }
    let cached_fetch_time = start.elapsed();
    println!("All cache hits in: {:?}", cached_fetch_time);

    if populate_time.as_nanos() > 0 && cached_fetch_time.as_nanos() > 0 {
        let speedup = populate_time.as_nanos() as f64 / cached_fetch_time.as_nanos() as f64;
        println!("🚀 Cached fetches are {:.1}x faster!", speedup);
    }

    // 8. Cache Key Information
    println!("\n🔑 Cache Key Structure");
    println!("=====================");

    println!("Cache keys follow this pattern:");
    println!("  • Format: 'prefix:record_id'");
    println!("  • Example: 'products:{}' ", test_product.id);
    println!("  • TTL: 1 hour (3600 seconds)");
    println!("  • Automatic invalidation on updates/deletes");

    println!("\n🎉 Basic Caching Demo Complete!");
    println!("==============================");

    println!("\n🎯 Key Takeaways:");
    println!("✅ Caching dramatically improves read performance");
    println!("✅ First access is slower (cache miss), subsequent accesses are faster");
    println!("✅ Cache is automatically invalidated on updates and deletes");
    println!("✅ TTL ensures data doesn't get stale");
    println!("✅ Redis provides distributed caching for multiple app instances");

    println!("\n🔧 How Caching Works:");
    println!("1. First read: Database → Cache → Return data");
    println!("2. Subsequent reads: Cache → Return data (faster!)");
    println!("3. Update/Delete: Database → Invalidate cache");
    println!("4. Next read: Database → Cache → Return data (cache rebuilt)");

    println!("\n⚙️  Cache Configuration:");
    println!("  • TTL: How long data stays in cache");
    println!("  • Prefix: Namespace for cache keys");
    println!("  • Pool size: Number of Redis connections");
    println!("  • Timeout: How long to wait for Redis operations");

    println!("\n📚 Next Steps:");
    println!("  • Try caching_advanced.rs for custom TTL and cache strategies");
    println!("  • Explore query result caching (coming soon)");
    println!("  • Learn about cache monitoring and metrics");

    Ok(())
}