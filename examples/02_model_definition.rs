//! # Advanced Model Definition Example
//!
//! This example demonstrates advanced model definition techniques:
//! - Different field types and attributes
//! - Soft delete models
//! - Model relationships
//! - Complex data types (JSON, arrays)
//! - Field constraints and validation

use storehaus::prelude::*;
use serde_json::Value;

/// Standard model with various field types
#[model]
#[table(name = "articles")]
pub struct Article {
    #[primary_key]
    pub id: Uuid,

    /// String field that can be created and updated
    #[field(create, update)]
    pub title: String,

    /// Large text field for content
    #[field(create, update)]
    pub content: String,

    /// Author name (create-only - cannot be changed after creation)
    #[field(create)]
    pub author: String,

    /// Published status (can be updated but not set on creation)
    #[field(update)]
    pub is_published: Option<bool>,

    /// View count (neither create nor update - managed by application)
    pub view_count: i32,

    /// JSON metadata field for flexible data
    #[field(create, update)]
    pub metadata: Option<Value>,
}

/// Soft delete model - automatically handles __is_active__ field
#[model]
#[table(name = "comments", auto_soft_delete)]
pub struct Comment {
    #[primary_key]
    pub id: Uuid,

    /// Foreign key to article
    #[field(create)]
    pub article_id: Uuid,

    /// Comment author
    #[field(create, update)]
    pub author: String,

    /// Comment content
    #[field(create, update)]
    pub content: String,

    /// Rating (1-5 stars)
    #[field(create, update)]
    pub rating: i32,

    /// Reply to another comment (optional foreign key)
    #[field(create, update)]
    pub parent_comment_id: Option<Uuid>,
}

/// Model with complex data types
#[model]
#[table(name = "products")]
pub struct Product {
    #[primary_key]
    pub id: Uuid,

    #[field(create, update)]
    pub name: String,

    #[field(create, update)]
    pub description: String,

    /// Price in cents to avoid floating point issues
    #[field(create, update)]
    pub price_cents: i32,

    /// Categories as JSON array
    #[field(create, update)]
    pub categories: Option<Value>,

    /// Product attributes as JSON object
    #[field(create, update)]
    pub attributes: Option<Value>,

    /// Inventory count
    #[field(create, update)]
    pub stock: i32,

    /// Whether the product is active for sale
    #[field(create, update)]
    pub __is_active__: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🏗️  StoreHaus Advanced Model Definition");
    println!("======================================");

    // Database setup
    let config = DatabaseConfig::new(
        "localhost".to_string(),
        5432,
        "storehaus".to_string(),
        "postgres".to_string(),
        "password".to_string(),
        1, 5, 30, 600, 3600,
    );

    let mut storehaus = StoreHaus::new(config).await?;
    println!("✅ Database connected");

    // Auto-migrate all models
    storehaus.auto_migrate::<Article>(true).await?;
    storehaus.auto_migrate::<Comment>(true).await?;
    storehaus.auto_migrate::<Product>(true).await?;
    println!("✅ All tables migrated");

    // Create stores
    let article_store = GenericStore::<Article>::new(storehaus.pool().clone(), None, None);
    let comment_store = GenericStore::<Comment>::new(storehaus.pool().clone(), None, None);
    let product_store = GenericStore::<Product>::new(storehaus.pool().clone(), None, None);

    storehaus.register_store("articles".to_string(), article_store)?;
    storehaus.register_store("comments".to_string(), comment_store)?;
    storehaus.register_store("products".to_string(), product_store)?;

    let article_store = storehaus.get_store::<GenericStore<Article>>("articles")?;
    let comment_store = storehaus.get_store::<GenericStore<Comment>>("comments")?;
    let product_store = storehaus.get_store::<GenericStore<Product>>("products")?;

    // 1. Standard Model Examples
    println!("\n📝 Part 1: Standard Model (Article)");
    println!("------------------------------------");

    let article = Article::new(
        Uuid::new_v4(),
        "Understanding Rust Ownership".to_string(),
        "Rust's ownership system is one of its most distinctive features...".to_string(),
        "Jane Doe".to_string(),
        None, // is_published is update-only, so None on creation
        0,    // view_count starts at 0
        Some(serde_json::json!({
            "difficulty": "intermediate",
            "tags": ["rust", "programming", "tutorial"],
            "estimated_read_time": 15
        })),
    );

    let created_article = article_store.create(article, None).await?;
    println!("✅ Created article: '{}'", created_article.title);
    println!("   Author: {} (create-only field)", created_article.author);
    println!("   Published: {:?}", created_article.is_published);
    println!("   Metadata: {}", created_article.metadata.as_ref().unwrap());

    // Update the article (cannot change author, but can publish)
    let mut updated_article = created_article.clone();
    updated_article.is_published = Some(true);
    updated_article.view_count = 42; // This won't be saved (not in update fields)

    let published_article = article_store
        .update(&created_article.id, updated_article, None)
        .await?;

    println!("📰 Article published!");
    println!("   Published status: {:?}", published_article.is_published);
    println!("   View count remains: {} (not updatable)", published_article.view_count);

    // 2. Soft Delete Model Examples
    println!("\n💬 Part 2: Soft Delete Model (Comment)");
    println!("---------------------------------------");

    let comment1 = Comment::new(
        Uuid::new_v4(),
        created_article.id,
        "Alice Reader".to_string(),
        "Great explanation of ownership! Really helped me understand.".to_string(),
        5,
        None, // Not a reply
    );

    let comment2 = Comment::new(
        Uuid::new_v4(),
        created_article.id,
        "Bob Developer".to_string(),
        "I still struggle with lifetimes though.".to_string(),
        4,
        None,
    );

    let created_comment1 = comment_store.create(comment1, None).await?;
    let created_comment2 = comment_store.create(comment2, None).await?;

    println!("✅ Created comment by {}: '{}'",
        created_comment1.author, &created_comment1.content[..50]);
    println!("✅ Created comment by {}: '{}'",
        created_comment2.author, created_comment2.content);

    // Reply to first comment
    let reply = Comment::new(
        Uuid::new_v4(),
        created_article.id,
        "Jane Doe".to_string(), // Article author replying
        "@Alice Thanks! Check out my upcoming article on lifetimes.".to_string(),
        5,
        Some(created_comment1.id), // This is a reply
    );

    let created_reply = comment_store.create(reply, None).await?;
    println!("✅ Created reply: '{}'", created_reply.content);

    // Demonstrate soft delete
    println!("\n🗑️  Soft Delete Demonstration:");
    let deleted = comment_store.delete(&created_comment2.id).await?;
    println!("Soft deleted comment: {}", deleted);

    // Verify soft delete (comment should not appear in normal queries)
    let remaining_comments = comment_store.list_all().await?;
    println!("Remaining visible comments: {}", remaining_comments.len());
    for comment in remaining_comments {
        println!("  • {}: {}", comment.author, &comment.content[..30]);
        if let Some(parent_id) = comment.parent_comment_id {
            println!("    (Reply to comment: {})", parent_id);
        }
    }

    // 3. Complex Data Types Examples
    println!("\n🛍️  Part 3: Complex Data Types (Product)");
    println!("----------------------------------------");

    let product = Product::new(
        Uuid::new_v4(),
        "Mechanical Keyboard".to_string(),
        "Professional mechanical keyboard with Cherry MX switches".to_string(),
        15999, // $159.99 in cents
        Some(serde_json::json!(["Electronics", "Computer Accessories", "Gaming"])),
        Some(serde_json::json!({
            "switch_type": "Cherry MX Blue",
            "backlit": true,
            "wireless": false,
            "key_count": 104,
            "warranty_months": 24,
            "colors": ["Black", "White", "RGB"]
        })),
        25,
        true,
    );

    let created_product = product_store.create(product, None).await?;
    println!("✅ Created product: {}", created_product.name);
    println!("   Price: ${:.2}", created_product.price_cents as f64 / 100.0);
    println!("   Categories: {}", created_product.categories.as_ref().unwrap());
    println!("   Attributes: {}", created_product.attributes.as_ref().unwrap());
    println!("   Stock: {} units", created_product.stock);

    // Update product with new attributes
    let mut updated_product = created_product.clone();
    updated_product.price_cents = 13999; // Sale price: $139.99
    updated_product.stock = 20;

    // Add sale information to attributes
    if let Some(Value::Object(ref mut attrs)) = updated_product.attributes {
        attrs.insert("on_sale".to_string(), Value::Bool(true));
        attrs.insert("sale_ends".to_string(), Value::String("2024-12-31".to_string()));
    }

    let sale_product = product_store
        .update(&created_product.id, updated_product, Some(vec!["sale".to_string()]))
        .await?;

    println!("🏷️  Product updated for sale:");
    println!("   New price: ${:.2}", sale_product.price_cents as f64 / 100.0);
    println!("   Updated attributes: {}", sale_product.attributes.as_ref().unwrap());

    // 4. Field Constraint Demonstrations
    println!("\n🔒 Part 4: Field Constraints");
    println!("-----------------------------");

    println!("📋 Field Types Summary:");
    println!("  • #[field(create)]: Can only be set during creation (like 'author')");
    println!("  • #[field(update)]: Can only be changed via updates (like 'is_published')");
    println!("  • #[field(create, update)]: Can be set and changed (like 'title', 'content')");
    println!("  • No attribute: Managed by application logic (like 'view_count')");

    println!("\n🔄 Soft Delete Models:");
    println!("  • Add 'auto_soft_delete' to #[table] attribute");
    println!("  • Automatically adds __is_active__ field");
    println!("  • delete() sets __is_active__ = false instead of removing row");
    println!("  • Normal queries filter out soft-deleted records");

    println!("\n💾 Complex Data Types:");
    println!("  • Use serde_json::Value for flexible JSON data");
    println!("  • Store arrays, objects, and nested structures");
    println!("  • Perfect for configuration, metadata, and dynamic attributes");

    println!("\n🎉 Advanced Model Definition Complete!");
    println!("=====================================");
    println!("\n🎯 Key Learnings:");
    println!("✅ Field attributes control when data can be modified");
    println!("✅ Soft delete preserves data while hiding it from queries");
    println!("✅ JSON fields enable flexible, schema-less data storage");
    println!("✅ Model::new() works seamlessly with all field types");
    println!("✅ System fields are always managed automatically");

    println!("\n📚 Next Steps:");
    println!("  • Explore advanced_querying.rs for complex data retrieval");
    println!("  • Try signals_basic.rs to react to model changes");
    println!("  • Check caching_basic.rs for performance optimization");

    Ok(())
}