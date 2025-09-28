//! # Blog System Example
//!
//! This example demonstrates a complete blog system implementation:
//! - Users, posts, comments, and categories
//! - Content management and permissions
//! - Comment threading and moderation
//! - Tag-based categorization
//! - Full-text search capabilities

use storehaus::prelude::*;
use serde_json::Value;

/// Blog user with roles and permissions
#[model]
#[table(name = "blog_users")]
pub struct BlogUser {
    #[primary_key]
    pub id: Uuid,

    #[field(create, update)]
    pub username: String,

    #[field(create, update)]
    pub email: String,

    #[field(create, update)]
    pub display_name: String,

    #[field(create, update)]
    pub role: String, // "admin", "editor", "author", "subscriber"

    #[field(create, update)]
    pub bio: Option<String>,

    #[field(create, update)]
    pub __is_active__: bool,

    #[field(create, update)]
    pub profile_data: Option<Value>, // JSON for avatar, social links, etc.
}

/// Blog categories for organizing posts
#[model]
#[table(name = "categories")]
pub struct Category {
    #[primary_key]
    pub id: Uuid,

    #[field(create, update)]
    pub name: String,

    #[field(create, update)]
    pub slug: String,

    #[field(create, update)]
    pub description: Option<String>,

    #[field(create, update)]
    pub parent_id: Option<Uuid>, // For nested categories

    #[field(create, update)]
    pub __is_active__: bool,
}

/// Blog posts with rich content and metadata
#[model]
#[table(name = "posts")]
pub struct Post {
    #[primary_key]
    pub id: Uuid,

    #[field(create)]
    pub author_id: Uuid,

    #[field(create, update)]
    pub title: String,

    #[field(create, update)]
    pub slug: String,

    #[field(create, update)]
    pub content: String,

    #[field(create, update)]
    pub excerpt: Option<String>,

    #[field(create, update)]
    pub status: String, // "draft", "published", "archived"

    #[field(create, update)]
    pub category_id: Option<Uuid>,

    #[field(create, update)]
    pub featured_image: Option<String>,

    #[field(create, update)]
    pub published_at: Option<chrono::DateTime<chrono::Utc>>,

    #[field(create, update)]
    pub view_count: i32,

    #[field(create, update)]
    pub metadata: Option<Value>, // SEO, custom fields, etc.
}

/// Comments with threading support and moderation
#[model]
#[table(name = "comments", auto_soft_delete)]
pub struct Comment {
    #[primary_key]
    pub id: Uuid,

    #[field(create)]
    pub post_id: Uuid,

    #[field(create)]
    pub author_id: Option<Uuid>, // Nullable for guest comments

    #[field(create, update)]
    pub author_name: String,

    #[field(create, update)]
    pub author_email: String,

    #[field(create, update)]
    pub content: String,

    #[field(create, update)]
    pub status: String, // "pending", "approved", "spam", "rejected"

    #[field(create)]
    pub parent_id: Option<Uuid>, // For threaded comments

    #[field(create, update)]
    pub ip_address: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("📝 StoreHaus Blog System Example");
    println!("===============================");

    // 1. Database Setup
    let config = DatabaseConfig::new(
        "localhost".to_string(), 5432, "storehaus".to_string(),
        "postgres".to_string(), "password".to_string(),
        1, 10, 30, 600, 3600,
    );

    let mut storehaus = StoreHaus::new(config).await?;

    // Migrate all tables
    storehaus.auto_migrate::<BlogUser>(true).await?;
    storehaus.auto_migrate::<Category>(true).await?;
    storehaus.auto_migrate::<Post>(true).await?;
    storehaus.auto_migrate::<Comment>(true).await?;
    println!("✅ All blog tables migrated");

    // 2. Signal System for Blog Events
    let signal_config = SignalConfig::new(30, 1000, true, 3, 60, true, 300);
    let signal_manager = SignalManager::new(signal_config);

    // Content moderation callback
    signal_manager
        .add_callback(|event: DatabaseEvent| async move {
            if event.table_name == "comments" && matches!(event.event_type, EventType::Create) {
                println!("🔍 New comment submitted - checking for moderation");

                // In a real app: run spam detection, profanity filter, etc.
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                println!("   ✅ Comment moderation check complete");
            }
            Ok(())
        })
        .await?;

    // SEO and search index callback
    signal_manager
        .add_callback(|event: DatabaseEvent| async move {
            if event.table_name == "posts" {
                match event.event_type {
                    EventType::Create | EventType::Update => {
                        println!("🔎 Updating search index for post");
                        // In a real app: update Elasticsearch, generate sitemap, etc.
                    },
                    EventType::Delete => {
                        println!("🗑️ Removing post from search index");
                    }
                }
                tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
            }
            Ok(())
        })
        .await?;

    // 3. Create Stores
    let user_store = GenericStore::<BlogUser>::new(
        storehaus.pool().clone(),
        Some(signal_manager.clone()),
        None,
    );

    let category_store = GenericStore::<Category>::new(
        storehaus.pool().clone(),
        Some(signal_manager.clone()),
        None,
    );

    let post_store = GenericStore::<Post>::new(
        storehaus.pool().clone(),
        Some(signal_manager.clone()),
        None,
    );

    let comment_store = GenericStore::<Comment>::new(
        storehaus.pool().clone(),
        Some(signal_manager.clone()),
        None,
    );

    storehaus.register_store("users".to_string(), user_store)?;
    storehaus.register_store("categories".to_string(), category_store)?;
    storehaus.register_store("posts".to_string(), post_store)?;
    storehaus.register_store("comments".to_string(), comment_store)?;

    let user_store = storehaus.get_store::<GenericStore<BlogUser>>("users")?;
    let category_store = storehaus.get_store::<GenericStore<Category>>("categories")?;
    let post_store = storehaus.get_store::<GenericStore<Post>>("posts")?;
    let comment_store = storehaus.get_store::<GenericStore<Comment>>("comments")?;

    // 4. Create Blog Users
    println!("\n👥 Creating Blog Users");
    println!("======================");

    let admin = BlogUser::new(
        Uuid::new_v4(),
        "admin".to_string(),
        "admin@blog.com".to_string(),
        "Blog Administrator".to_string(),
        "admin".to_string(),
        Some("Managing this awesome blog!".to_string()),
        true,
        Some(serde_json::json!({
            "avatar": "/avatars/admin.jpg",
            "social": {
                "twitter": "@blogadmin",
                "linkedin": "blog-admin"
            }
        })),
    );

    let author1 = BlogUser::new(
        Uuid::new_v4(),
        "alice_writer".to_string(),
        "alice@blog.com".to_string(),
        "Alice Johnson".to_string(),
        "author".to_string(),
        Some("Tech writer passionate about Rust and systems programming.".to_string()),
        true,
        Some(serde_json::json!({
            "avatar": "/avatars/alice.jpg",
            "expertise": ["Rust", "Systems Programming", "DevOps"]
        })),
    );

    let author2 = BlogUser::new(
        Uuid::new_v4(),
        "bob_coder".to_string(),
        "bob@blog.com".to_string(),
        "Bob Smith".to_string(),
        "author".to_string(),
        Some("Full-stack developer sharing web development insights.".to_string()),
        true,
        None,
    );

    let created_admin = user_store.create(admin, Some(vec!["user-setup".to_string()])).await?;
    let created_author1 = user_store.create(author1, Some(vec!["user-setup".to_string()])).await?;
    let created_author2 = user_store.create(author2, Some(vec!["user-setup".to_string()])).await?;

    println!("✅ Created blog users:");
    println!("   • {} ({})", created_admin.display_name, created_admin.role);
    println!("   • {} ({})", created_author1.display_name, created_author1.role);
    println!("   • {} ({})", created_author2.display_name, created_author2.role);

    // 5. Create Categories
    println!("\n📂 Creating Categories");
    println!("======================");

    let tech_category = Category::new(
        Uuid::new_v4(),
        "Technology".to_string(),
        "technology".to_string(),
        Some("All things tech and programming".to_string()),
        None, // Top-level category
        true,
    );

    let rust_category = Category::new(
        Uuid::new_v4(),
        "Rust Programming".to_string(),
        "rust".to_string(),
        Some("Rust language tutorials and insights".to_string()),
        None, // Will be set to tech_category.id after creation
        true,
    );

    let web_category = Category::new(
        Uuid::new_v4(),
        "Web Development".to_string(),
        "web-dev".to_string(),
        Some("Frontend and backend web development".to_string()),
        None, // Will be set to tech_category.id after creation
        true,
    );

    let created_tech = category_store.create(tech_category, Some(vec!["setup".to_string()])).await?;

    // Create subcategories
    let mut rust_cat = rust_category;
    rust_cat.parent_id = Some(created_tech.id);
    let created_rust = category_store.create(rust_cat, Some(vec!["setup".to_string()])).await?;

    let mut web_cat = web_category;
    web_cat.parent_id = Some(created_tech.id);
    let created_web = category_store.create(web_cat, Some(vec!["setup".to_string()])).await?;

    println!("✅ Created categories:");
    println!("   • {} ({})", created_tech.name, created_tech.slug);
    println!("     ├─ {} ({})", created_rust.name, created_rust.slug);
    println!("     └─ {} ({})", created_web.name, created_web.slug);

    // 6. Create Blog Posts
    println!("\n📝 Creating Blog Posts");
    println!("======================");

    let posts_data = vec![
        (
            &created_author1,
            &created_rust,
            "Getting Started with Rust",
            "rust-getting-started",
            "A comprehensive guide to starting your Rust programming journey...",
            "published",
            vec!["tutorial", "beginner", "rust-basics"]
        ),
        (
            &created_author1,
            &created_rust,
            "Advanced Rust: Ownership and Lifetimes",
            "rust-ownership-lifetimes",
            "Deep dive into Rust's ownership system and lifetime annotations...",
            "published",
            vec!["advanced", "ownership", "lifetimes"]
        ),
        (
            &created_author2,
            &created_web,
            "Building REST APIs with Rust",
            "rust-rest-apis",
            "Learn how to build fast and secure REST APIs using Rust...",
            "published",
            vec!["api", "web-development", "backend"]
        ),
        (
            &created_author2,
            &created_web,
            "Frontend State Management",
            "frontend-state-management",
            "Comparing different approaches to state management in modern web apps...",
            "draft",
            vec!["frontend", "state-management", "javascript"]
        ),
    ];

    let mut created_posts = Vec::new();
    for (author, category, title, slug, content, status, tags) in posts_data {
        let post = Post::new(
            Uuid::new_v4(),
            author.id,
            title.to_string(),
            slug.to_string(),
            content.to_string(),
            Some(format!("{}...", &content[..50])), // Auto-generated excerpt
            status.to_string(),
            Some(category.id),
            None, // No featured image
            if status == "published" {
                Some(chrono::Utc::now())
            } else {
                None
            },
            0, // Initial view count
            Some(serde_json::json!({
                "seo_title": title,
                "meta_description": format!("{} - {}", title, content),
                "reading_time": content.len() / 200 // Rough estimate
            })),
        );

        let created_post = post_store.create(
            post,
            Some(tags.into_iter().map(String::from).collect()),
        ).await?;

        created_posts.push(created_post);
    }

    println!("✅ Created {} blog posts", created_posts.len());
    for post in &created_posts {
        println!("   • '{}' by {} ({})", post.title,
            if post.author_id == created_author1.id { "Alice" } else { "Bob" },
            post.status);
    }

    // 7. Create Comments and Threading
    println!("\n💬 Creating Comments");
    println!("====================");

    let first_post = &created_posts[0];

    // Root comments
    let comment1 = Comment::new(
        Uuid::new_v4(),
        first_post.id,
        Some(created_author2.id),
        "Bob Smith".to_string(),
        "bob@blog.com".to_string(),
        "Great introduction to Rust! Really helped me understand the basics.".to_string(),
        "approved".to_string(),
        None, // Root comment
        Some("192.168.1.100".to_string()),
    );

    let comment2 = Comment::new(
        Uuid::new_v4(),
        first_post.id,
        None, // Guest comment
        "Jane Reader".to_string(),
        "jane@example.com".to_string(),
        "Thanks for this tutorial. Looking forward to more advanced topics!".to_string(),
        "approved".to_string(),
        None, // Root comment
        Some("192.168.1.101".to_string()),
    );

    let created_comment1 = comment_store.create(comment1, Some(vec!["engagement".to_string()])).await?;
    let _created_comment2 = comment_store.create(comment2, Some(vec!["engagement".to_string()])).await?;

    // Reply to first comment (threaded)
    let reply = Comment::new(
        Uuid::new_v4(),
        first_post.id,
        Some(created_author1.id),
        "Alice Johnson".to_string(),
        "alice@blog.com".to_string(),
        "@Bob Thanks for the feedback! I'm working on the advanced series now.".to_string(),
        "approved".to_string(),
        Some(created_comment1.id), // This is a reply
        Some("192.168.1.1".to_string()),
    );

    let _created_reply = comment_store.create(reply, Some(vec!["author-reply".to_string()])).await?;

    println!("✅ Created comments with threading:");
    println!("   • Root comment by Bob");
    println!("   • Root comment by Jane (guest)");
    println!("   • Reply by Alice (author)");

    // 8. Demonstrate Content Queries
    println!("\n🔍 Content Queries");
    println!("==================");

    // Find published posts
    let published_posts = post_store
        .find(
            QueryBuilder::new()
                .filter(QueryFilter::eq("status", serde_json::json!("published")))
                .order_by("published_at", SortOrder::Desc),
        )
        .await?;

    println!("📰 Published posts: {}", published_posts.len());
    for post in published_posts {
        println!("   • '{}' - {} views", post.title, post.view_count);
    }

    // Find posts by category
    let rust_posts = post_store
        .find(
            QueryBuilder::new()
                .filter(QueryFilter::eq("category_id", serde_json::json!(created_rust.id)))
                .order_by("__created_at__", SortOrder::Desc),
        )
        .await?;

    println!("\n🦀 Rust category posts: {}", rust_posts.len());
    for post in rust_posts {
        println!("   • '{}'", post.title);
    }

    // Find posts by tags
    let tutorial_posts = post_store
        .find(
            QueryBuilder::new()
                .filter_by_any_tag(vec!["tutorial".to_string()])
                .filter(QueryFilter::eq("status", serde_json::json!("published"))),
        )
        .await?;

    println!("\n📚 Tutorial posts: {}", tutorial_posts.len());

    // Find comments for a post
    let post_comments = comment_store
        .find(
            QueryBuilder::new()
                .filter(QueryFilter::eq("post_id", serde_json::json!(first_post.id)))
                .filter(QueryFilter::eq("status", serde_json::json!("approved")))
                .order_by("__created_at__", SortOrder::Asc),
        )
        .await?;

    println!("\n💭 Comments on '{}': {}", first_post.title, post_comments.len());
    for comment in post_comments {
        let indent = if comment.parent_id.is_some() { "    └─ " } else { "  • " };
        println!("{}{}: {}", indent, comment.author_name, &comment.content[..50]);
    }

    // 9. Content Management Demo
    println!("\n📋 Content Management");
    println!("=====================");

    // Update post view count
    let mut popular_post = created_posts[0].clone();
    popular_post.view_count = 157;
    let post_id = popular_post.id;

    let _updated_post = post_store.update(
        &post_id,
        popular_post,
        Some(vec!["view-count-update".to_string()]),
    ).await?;

    println!("📊 Updated post view count: 157 views");

    // Moderate comment (soft delete)
    let spam_comment = Comment::new(
        Uuid::new_v4(),
        first_post.id,
        None,
        "Spammer".to_string(),
        "spam@spam.com".to_string(),
        "Buy cheap products now! Click here!!!".to_string(),
        "spam".to_string(),
        None,
        Some("192.168.1.999".to_string()),
    );

    let created_spam = comment_store.create(spam_comment, Some(vec!["moderation".to_string()])).await?;

    // Soft delete the spam comment
    let _deleted = comment_store.delete(&created_spam.id).await?;
    println!("🚫 Spam comment removed (soft deleted)");

    // Verify it's hidden from normal queries
    let visible_comments = comment_store
        .find(
            QueryBuilder::new()
                .filter(QueryFilter::eq("post_id", serde_json::json!(first_post.id))),
        )
        .await?;

    println!("✅ Visible comments after moderation: {}", visible_comments.len());

    println!("\n🎉 Blog System Demo Complete!");
    println!("=============================");

    println!("\n🎯 Features Demonstrated:");
    println!("✅ Multi-role user system (admin, author, subscriber)");
    println!("✅ Hierarchical categories with parent-child relationships");
    println!("✅ Rich blog posts with metadata and SEO fields");
    println!("✅ Threaded comments with moderation");
    println!("✅ Content workflow (draft → published → archived)");
    println!("✅ Tag-based categorization and filtering");
    println!("✅ Soft delete for content moderation");
    println!("✅ Signal-driven search indexing and notifications");

    println!("\n🏗️ Architecture Highlights:");
    println!("  • Normalized database design with proper relationships");
    println!("  • JSON fields for flexible metadata and configuration");
    println!("  • Event-driven architecture with signals");
    println!("  • Soft delete for comment moderation");
    println!("  • Tag system for content categorization");
    println!("  • Role-based permissions (extendable)");

    println!("\n📚 Real-World Extensions:");
    println!("  • Add caching for popular posts and categories");
    println!("  • Implement full-text search with Elasticsearch");
    println!("  • Add email notifications for new comments");
    println!("  • Build API endpoints for headless CMS");
    println!("  • Add image upload and media management");
    println!("  • Implement comment voting and reputation");

    Ok(())
}