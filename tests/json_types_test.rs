//! Integration tests for JSON/JSONB type support
//!
//! Tests various JSON type mappings, CRUD operations, and query filtering
//! with JSON fields in PostgreSQL.

use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::PgPool;
use storehaus::prelude::*;

/// Model with various JSON field types
#[model]
#[table(name = "json_test_model")]
pub struct JsonTestModel {
    #[primary_key]
    pub id: Uuid,

    #[field(create, update)]
    pub name: String,

    #[field(create, update)]
    pub metadata: serde_json::Value,

    #[field(create, update)]
    pub optional_data: Option<serde_json::Value>,

    #[field(create, update)]
    pub config: serde_json::Value,
}

/// Custom config struct for typed JSON
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct AppConfig {
    pub enabled: bool,
    pub max_retries: i32,
    pub endpoints: Vec<String>,
}

/// Model using sqlx::types::Json<T> for typed JSON
#[model]
#[table(name = "typed_json_test")]
pub struct TypedJsonTest {
    #[primary_key]
    pub id: Uuid,

    #[field(create, update)]
    pub name: String,

    #[field(create, update)]
    pub config: sqlx::types::Json<AppConfig>,

    #[field(create, update)]
    pub optional_config: Option<sqlx::types::Json<AppConfig>>,
}

async fn setup_pool() -> PgPool {
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set for integration tests");

    PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to database")
}

async fn cleanup_tables(pool: &PgPool) {
    let _ = sqlx::query("DROP TABLE IF EXISTS json_test_model CASCADE")
        .execute(pool)
        .await;
    let _ = sqlx::query("DROP TABLE IF EXISTS typed_json_test CASCADE")
        .execute(pool)
        .await;
}

async fn migrate_table<T: TableMetadata>(pool: &PgPool) {
    let create_sql = T::create_table_sql();
    sqlx::query(&create_sql)
        .execute(pool)
        .await
        .expect("Failed to create table");
}

#[tokio::test]
async fn test_json_table_creation() {
    let pool = setup_pool().await;
    cleanup_tables(&pool).await;

    migrate_table::<JsonTestModel>(&pool).await;

    let create_sql = JsonTestModel::create_table_sql();
    assert!(create_sql.contains("JSONB"), "metadata should be JSONB: {}", create_sql);

    cleanup_tables(&pool).await;
}

#[tokio::test]
async fn test_typed_json_table_creation() {
    let pool = setup_pool().await;
    cleanup_tables(&pool).await;

    migrate_table::<TypedJsonTest>(&pool).await;

    let create_sql = TypedJsonTest::create_table_sql();
    assert!(create_sql.contains("JSONB"), "config should be JSONB: {}", create_sql);

    cleanup_tables(&pool).await;
}

#[tokio::test]
async fn test_json_create_and_read() {
    let pool = setup_pool().await;
    cleanup_tables(&pool).await;

    migrate_table::<JsonTestModel>(&pool).await;

    let json_store = GenericStore::<JsonTestModel>::new(pool.clone(), None, None);

    let model = JsonTestModel::new(
        Uuid::new_v4(),
        "test".to_string(),
        json!({
            "key": "value",
            "nested": {
                "array": [1, 2, 3],
                "bool": true
            }
        }),
        Some(json!({"optional": "present"})),
        json!(["item1", "item2"]),
    );

    let created = json_store.create(model, None).await.unwrap();
    assert_eq!(created.metadata["key"], "value");
    assert_eq!(created.metadata["nested"]["array"][0], 1);

    let fetched = json_store.get_by_id(&created.id).await.unwrap().unwrap();
    assert_eq!(fetched.metadata, created.metadata);
    assert_eq!(fetched.optional_data, created.optional_data);
    assert_eq!(fetched.config, created.config);

    cleanup_tables(&pool).await;
}

#[tokio::test]
async fn test_json_update() {
    let pool = setup_pool().await;
    cleanup_tables(&pool).await;

    migrate_table::<JsonTestModel>(&pool).await;

    let json_store = GenericStore::<JsonTestModel>::new(pool.clone(), None, None);

    let model = JsonTestModel::new(
        Uuid::new_v4(),
        "original".to_string(),
        json!({"version": 1}),
        None,
        json!([]),
    );

    let created = json_store.create(model, None).await.unwrap();

    let mut updated_model = json_store.get_by_id(&created.id).await.unwrap().unwrap();
    updated_model.metadata = json!({
        "version": 2,
        "new_field": "added"
    });
    updated_model.optional_data = Some(json!({"now": "present"}));
    updated_model.config = json!(["new", "items"]);

    let updated = json_store.update(&created.id, updated_model, None).await.unwrap();
    assert_eq!(updated.metadata["version"], 2);
    assert_eq!(updated.metadata["new_field"], "added");
    assert!(updated.optional_data.is_some());
    assert_eq!(updated.config[0], "new");

    cleanup_tables(&pool).await;
}

#[tokio::test]
async fn test_json_query_filter() {
    let pool = setup_pool().await;
    cleanup_tables(&pool).await;

    migrate_table::<JsonTestModel>(&pool).await;

    let json_store = GenericStore::<JsonTestModel>::new(pool.clone(), None, None);

    for i in 0..3 {
        let model = JsonTestModel::new(
            Uuid::new_v4(),
            format!("item_{}", i),
            json!({"index": i, "category": if i % 2 == 0 { "even" } else { "odd" }}),
            if i > 0 { Some(json!({"has_data": true})) } else { None },
            json!([i]),
        );
        json_store.create(model, None).await.unwrap();
    }

    let query = QueryBuilder::new()
        .filter(QueryFilter::eq("name", json!("item_1")));
    let results = json_store.find(query).await.unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].metadata["index"], 1);

    cleanup_tables(&pool).await;
}

#[tokio::test]
async fn test_json_update_where() {
    let pool = setup_pool().await;
    cleanup_tables(&pool).await;

    migrate_table::<JsonTestModel>(&pool).await;

    let json_store = GenericStore::<JsonTestModel>::new(pool.clone(), None, None);

    let model = JsonTestModel::new(
        Uuid::new_v4(),
        "to_update".to_string(),
        json!({"status": "pending"}),
        None,
        json!([]),
    );
    json_store.create(model, None).await.unwrap();

    let query = QueryBuilder::new()
        .filter(QueryFilter::eq("name", json!("to_update")))
        .update(UpdateSet::new().set("config", json!({"updated": true, "items": [1, 2, 3]})));

    let updated = json_store.update_where(query, None).await.unwrap();
    assert_eq!(updated.len(), 1);
    assert_eq!(updated[0].config["updated"], true);
    assert_eq!(updated[0].config["items"][0], 1);

    cleanup_tables(&pool).await;
}

#[tokio::test]
async fn test_typed_json_crud() {
    let pool = setup_pool().await;
    cleanup_tables(&pool).await;

    migrate_table::<TypedJsonTest>(&pool).await;

    let typed_store = GenericStore::<TypedJsonTest>::new(pool.clone(), None, None);

    let config = AppConfig {
        enabled: true,
        max_retries: 5,
        endpoints: vec!["http://api1.com".to_string(), "http://api2.com".to_string()],
    };

    let model = TypedJsonTest::new(
        Uuid::new_v4(),
        "typed_test".to_string(),
        sqlx::types::Json(config.clone()),
        Some(sqlx::types::Json(AppConfig {
            enabled: false,
            max_retries: 0,
            endpoints: vec![],
        })),
    );

    let created = typed_store.create(model, None).await.unwrap();
    assert_eq!(created.config.0, config);
    assert!(created.optional_config.is_some());

    let fetched = typed_store.get_by_id(&created.id).await.unwrap().unwrap();
    assert_eq!(fetched.config.0.enabled, true);
    assert_eq!(fetched.config.0.max_retries, 5);
    assert_eq!(fetched.config.0.endpoints.len(), 2);

    cleanup_tables(&pool).await;
}

#[tokio::test]
async fn test_json_with_complex_nested_structures() {
    let pool = setup_pool().await;
    cleanup_tables(&pool).await;

    migrate_table::<JsonTestModel>(&pool).await;

    let json_store = GenericStore::<JsonTestModel>::new(pool.clone(), None, None);

    let complex_json = json!({
        "users": [
            {
                "id": 1,
                "name": "Alice",
                "roles": ["admin", "user"],
                "metadata": {
                    "created": "2024-01-01",
                    "settings": {
                        "theme": "dark",
                        "notifications": true
                    }
                }
            },
            {
                "id": 2,
                "name": "Bob",
                "roles": ["user"],
                "metadata": null
            }
        ],
        "pagination": {
            "page": 1,
            "per_page": 10,
            "total": 2
        }
    });

    let model = JsonTestModel::new(
        Uuid::new_v4(),
        "complex".to_string(),
        complex_json.clone(),
        None,
        json!([]),
    );

    let created = json_store.create(model, None).await.unwrap();

    let fetched = json_store.get_by_id(&created.id).await.unwrap().unwrap();
    assert_eq!(fetched.metadata, complex_json);
    assert_eq!(fetched.metadata["users"][0]["name"], "Alice");
    assert_eq!(fetched.metadata["users"][0]["metadata"]["settings"]["theme"], "dark");
    assert!(fetched.metadata["users"][1]["metadata"].is_null());

    cleanup_tables(&pool).await;
}

#[tokio::test]
async fn test_json_null_handling() {
    let pool = setup_pool().await;
    cleanup_tables(&pool).await;

    migrate_table::<JsonTestModel>(&pool).await;

    let json_store = GenericStore::<JsonTestModel>::new(pool.clone(), None, None);

    let model = JsonTestModel::new(
        Uuid::new_v4(),
        "null_test".to_string(),
        json!(null),
        None,
        json!({"key": null, "array": [null, 1, null]}),
    );

    let created = json_store.create(model, None).await.unwrap();

    let fetched = json_store.get_by_id(&created.id).await.unwrap().unwrap();
    assert!(fetched.metadata.is_null());
    assert!(fetched.optional_data.is_none());
    assert!(fetched.config["key"].is_null());
    assert!(fetched.config["array"][0].is_null());
    assert_eq!(fetched.config["array"][1], 1);

    cleanup_tables(&pool).await;
}

#[tokio::test]
async fn test_json_empty_structures() {
    let pool = setup_pool().await;
    cleanup_tables(&pool).await;

    migrate_table::<JsonTestModel>(&pool).await;

    let json_store = GenericStore::<JsonTestModel>::new(pool.clone(), None, None);

    let model = JsonTestModel::new(
        Uuid::new_v4(),
        "empty_test".to_string(),
        json!({}),
        Some(json!([])),
        json!([]),
    );

    let created = json_store.create(model, None).await.unwrap();

    let fetched = json_store.get_by_id(&created.id).await.unwrap().unwrap();
    assert!(fetched.metadata.as_object().unwrap().is_empty());
    assert!(fetched.optional_data.unwrap().as_array().unwrap().is_empty());
    assert!(fetched.config.as_array().unwrap().is_empty());

    cleanup_tables(&pool).await;
}