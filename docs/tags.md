# Tagging System

StoreHaus provides a comprehensive tagging system that allows you to group, track, and query database operations. Tags are stored as a system field and are integrated throughout the entire operation lifecycle.

## Overview

The tagging system enables you to:

- **Group operations** by business context (e.g., "user-registration", "data-migration")
- **Track operation sources** (e.g., "api", "admin-panel", "batch-job")
- **Implement audit trails** with operation categorization
- **Query records** by tag combinations
- **Monitor activities** through signal events

Tags are automatically indexed for efficient searching and are included in all signal events for external monitoring.

## Basic Usage

### Creating Records with Tags

```rust
use uuid::Uuid;

// Create a user with registration tags
let user = User {
    id: Uuid::new_v4(),
    name: "John Doe".to_string(),
    email: "john@example.com".to_string(),
};

// Tags are passed as optional parameter
let created = user_store.create(
    user,
    Some(vec![
        "user-registration".to_string(),
        "source:web".to_string(),
        "campaign:summer2024".to_string(),
    ])
).await?;

// Without tags
let created_no_tags = user_store.create(user, None).await?;
```

### Updating Records with Tags

```rust
// Update user profile with tracking tags
let updated_user = User {
    id: existing_id,
    name: "John Smith".to_string(),  // Changed name
    email: "john.smith@example.com".to_string(),
};

let updated = user_store.update(
    &existing_id,
    updated_user,
    Some(vec![
        "profile-update".to_string(),
        "source:mobile-app".to_string(),
        "user-initiated".to_string(),
    ])
).await?;
```

### Batch Operations with Tags

```rust
// Update multiple records with shared tags
let updates = vec![
    (user1_id, updated_user1),
    (user2_id, updated_user2),
    (user3_id, updated_user3),
];

// Tags apply to all records in the batch
let results = user_store.update_many_with_tags(
    updates,
    vec![
        "bulk-update".to_string(),
        "data-migration".to_string(),
        "admin:john".to_string(),
    ]
).await?;
```

## Tag Querying

### Query by Any Tag

Find records that have at least one of the specified tags:

```rust
use store_object::{QueryBuilder, QueryFilter};

// Find users tagged with any premium-related tags
let premium_users = user_store.find(
    QueryBuilder::new()
        .filter_by_any_tag(vec![
            "premium".to_string(),
            "vip".to_string(),
            "enterprise".to_string(),
        ])
).await?;
```

### Query by All Tags

Find records that have all of the specified tags:

```rust
// Find users who were registered via web AND during a specific campaign
let campaign_web_users = user_store.find(
    QueryBuilder::new()
        .filter_by_all_tags(vec![
            "source:web".to_string(),
            "campaign:summer2024".to_string(),
        ])
).await?;
```

### Advanced Tag Filtering

```rust
// Combine tag filters with other conditions
let complex_query = user_store.find(
    QueryBuilder::new()
        .filter_by_any_tag(vec!["premium".to_string(), "vip".to_string()])
        .filter(QueryFilter::gte("__created_at__", json!("2024-01-01T00:00:00Z")))
        .filter(QueryFilter::eq("__is_active__", json!(true)))
        .order_by("__created_at__", SortOrder::Desc)
        .limit(100)
).await?;
```

## Tag Naming Conventions

### Hierarchical Tags

Use colons to create hierarchical tag structures:

```rust
// Source hierarchy
"source:web"
"source:mobile:ios"
"source:mobile:android"
"source:api:v1"
"source:api:v2"

// User action hierarchy
"action:user:login"
"action:user:logout"
"action:admin:user-create"
"action:admin:user-delete"

// Business context hierarchy
"context:onboarding:step1"
"context:onboarding:step2"
"context:checkout:payment"
"context:checkout:confirmation"
```

### Category-Based Tags

```rust
// Operation categories
let operation_tags = vec![
    "category:user-management".to_string(),
    "priority:high".to_string(),
    "environment:production".to_string(),
];

// Feature flags
let feature_tags = vec![
    "feature:new-ui".to_string(),
    "experiment:ab-test-v2".to_string(),
    "rollout:50percent".to_string(),
];

// Compliance and audit
let audit_tags = vec![
    "audit:gdpr".to_string(),
    "retention:7years".to_string(),
    "privacy:sensitive".to_string(),
];
```

## Signal Integration

Tags are automatically included in all signal events:

```rust
use signal_system::SignalManager;

let signal_manager = Arc::new(SignalManager::new());

// Monitor tagged operations
signal_manager.add_callback(|event: &DatabaseEvent| {
    if !event.tags.is_empty() {
        println!("Tagged operation detected:");
        println!("  Table: {}", event.table_name);
        println!("  Operation: {:?}", event.event_type);
        println!("  Tags: {:?}", event.tags);

        // Business logic based on tags
        if event.tags.contains(&"audit:required".to_string()) {
            // Send to audit system
            send_to_audit_log(event);
        }

        if event.tags.iter().any(|tag| tag.starts_with("alert:")) {
            // Trigger notifications
            send_alert(event);
        }
    }
});
```

## Advanced Use Cases

### Audit Trail Implementation

```rust
// Tag all operations for comprehensive audit trail
async fn audit_user_operation(
    user_id: Uuid,
    operation: &str,
    operator: &str,
) -> Result<(), StorehausError> {
    let tags = vec![
        format!("audit:required"),
        format!("operation:{}", operation),
        format!("operator:{}", operator),
        format!("timestamp:{}", chrono::Utc::now().timestamp()),
    ];

    // All operations include audit tags
    match operation {
        "create" => user_store.create(user_data, Some(tags)).await?,
        "update" => user_store.update(&user_id, user_data, Some(tags)).await?,
        "delete" => {
            // Delete operations don't take data, but tags are captured in signals
            user_store.delete(&user_id).await?;
        }
        _ => return Err(StorehausError::ValidationError("Unknown operation".into())),
    };

    Ok(())
}
```

### Feature Flag Management

```rust
// Tag operations based on active feature flags
async fn create_user_with_features(user: User) -> Result<User, StorehausError> {
    let mut tags = vec!["user-creation".to_string()];

    // Add feature flag tags
    if is_feature_enabled("new-onboarding") {
        tags.push("feature:new-onboarding".to_string());
    }

    if is_feature_enabled("enhanced-analytics") {
        tags.push("feature:enhanced-analytics".to_string());
    }

    // Environment tag
    tags.push(format!("env:{}", get_environment()));

    user_store.create(user, Some(tags)).await
}
```

### Data Migration Tracking

```rust
// Track data migration operations
async fn migrate_user_batch(users: Vec<User>) -> Result<(), StorehausError> {
    let migration_id = Uuid::new_v4();
    let batch_tags = vec![
        "data-migration".to_string(),
        "migration:user-schema-v2".to_string(),
        format!("batch-id:{}", migration_id),
        format!("batch-size:{}", users.len()),
    ];

    let updates: Vec<(Uuid, User)> = users
        .into_iter()
        .map(|user| (user.id, user))
        .collect();

    // All users in batch get the same migration tags
    user_store.update_many_with_tags(updates, batch_tags).await?;

    Ok(())
}
```

### Campaign Tracking

```rust
// Track marketing campaign performance
#[derive(Debug)]
pub struct CampaignTracker {
    campaign_id: String,
    source: String,
    medium: String,
}

impl CampaignTracker {
    pub fn create_tags(&self) -> Vec<String> {
        vec![
            format!("campaign:{}", self.campaign_id),
            format!("source:{}", self.source),
            format!("medium:{}", self.medium),
            format!("timestamp:{}", chrono::Utc::now().format("%Y-%m-%d")),
        ]
    }
}

async fn register_user_with_campaign(
    user: User,
    campaign: CampaignTracker,
) -> Result<User, StorehausError> {
    let mut tags = campaign.create_tags();
    tags.push("user-registration".to_string());

    user_store.create(user, Some(tags)).await
}

// Query campaign results
async fn get_campaign_registrations(campaign_id: &str) -> Result<Vec<User>, StorehausError> {
    user_store.find(
        QueryBuilder::new()
            .filter_by_any_tag(vec![format!("campaign:{}", campaign_id)])
            .order_by("__created_at__", SortOrder::Desc)
    ).await
}
```

## Performance Considerations

### Tag Indexing

Tags are automatically indexed using PostgreSQL's GIN (Generalized Inverted Index):

```sql
-- Automatically created for all tables
CREATE INDEX IF NOT EXISTS idx_tablename_tags ON tablename USING GIN(__tags__);
```

This provides efficient searching for:
- `@>` (contains array)
- `&&` (overlaps with array)
- Array operations

### Query Optimization

```rust
// Efficient: Use tag-specific methods
let results = user_store.find(
    QueryBuilder::new()
        .filter_by_any_tag(vec!["premium".to_string()])
).await?;

// Less efficient: Raw array operations (avoid if possible)
let results = user_store.find(
    QueryBuilder::new()
        .filter(QueryFilter::raw("__tags__ @> ARRAY['premium']"))
).await?;
```

### Tag Management

Keep tags concise and avoid excessive nesting:

```rust
// Good: Concise, meaningful tags
vec!["reg:web", "campaign:summer24", "priority:high"]

// Avoid: Overly verbose or nested tags
vec![
    "user-registration-via-web-interface-during-summer-campaign".to_string(),
    "business:marketing:campaigns:seasonal:summer:2024".to_string(),
]
```

## Best Practices

### Consistent Naming

Establish and maintain consistent tag naming conventions:

```rust
// Use consistent separators and casing
"source:web"        // not "source_web" or "SOURCE:WEB"
"campaign:summer24"  // not "Campaign:Summer2024"
"env:prod"          // not "environment:production"
```

### Tag Categorization

Group related tags by purpose:

```rust
// Operational tags
let ops_tags = vec!["env:prod", "region:us-east", "service:api"];

// Business tags
let biz_tags = vec!["campaign:holiday", "segment:premium", "cohort:2024-q1"];

// Technical tags
let tech_tags = vec!["version:v2.1", "feature:new-ui", "experiment:ab-test"];

// Combine as needed
let all_tags = [ops_tags, biz_tags, tech_tags].concat();
```

### Tag Cleanup

Implement tag lifecycle management:

```rust
// Remove expired campaign tags
async fn cleanup_expired_tags() -> Result<(), StorehausError> {
    let cutoff_date = chrono::Utc::now() - chrono::Duration::days(90);

    // Find records with old campaign tags
    let old_records = user_store.find(
        QueryBuilder::new()
            .filter_by_any_tag(vec!["campaign:".to_string()]) // prefix match
            .filter(QueryFilter::lt("__updated_at__", json!(cutoff_date)))
    ).await?;

    // Note: Tag cleanup would require custom implementation
    // as StoreHaus doesn't currently support tag modification

    Ok(())
}
```