//! JOIN and Aggregation Demo
//!
//! This example demonstrates the new JOIN and aggregation capabilities in StoreHaus:
//! - INNER, LEFT, RIGHT, and FULL OUTER JOINs
//! - Aggregate functions (COUNT, SUM, AVG, MIN, MAX)
//! - GROUP BY with HAVING clauses
//! - Complex queries combining JOINs, aggregations, and filtering

use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::PgPool;
use std::error::Error;
use storehaus::prelude::*;
use store_object::query_builder::{
    GroupBy, JoinClause, JoinType, QueryBuilder, QueryFilter, SelectField, SortOrder,
};
use uuid::Uuid;

#[model]
#[table(name = "users")]
pub struct User {
    #[primary_key]
    pub id: Uuid,
    #[field(create, update)]
    pub name: String,
    #[field(create, update)]
    pub email: String,
}

#[model]
#[table(name = "orders")]
pub struct Order {
    #[primary_key]
    pub id: Uuid,
    #[field(create, update)]
    pub user_id: Uuid,
    #[field(create, update)]
    pub total_amount: f64,
    #[field(create, update)]
    pub status: String,
}

#[model]
#[table(name = "order_items")]
pub struct OrderItem {
    #[primary_key]
    pub id: Uuid,
    #[field(create, update)]
    pub order_id: Uuid,
    #[field(create, update)]
    pub product_name: String,
    #[field(create, update)]
    pub quantity: i32,
    #[field(create, update)]
    pub price: f64,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("🚀 StoreHaus JOIN and Aggregation Demo\n");

    // Database configuration
    let config = DatabaseConfig::new(
        "localhost".to_string(),
        5432,
        "storehaus".to_string(),
        "postgres".to_string(),
        "password".to_string(),
        1,
        10,
        30,
        600,
        3600,
    );

    let storehaus = StoreHaus::new(config).await?;
    let pool = storehaus.pool().clone();

    // Auto-migrate tables
    println!("📊 Setting up tables...");
    storehaus.auto_migrate::<User>(true).await?;
    storehaus.auto_migrate::<Order>(true).await?;
    storehaus.auto_migrate::<OrderItem>(true).await?;

    // Create stores
    let user_store = GenericStore::<User>::new(pool.clone(), None, None);
    let order_store = GenericStore::<Order>::new(pool.clone(), None, None);
    let order_item_store = GenericStore::<OrderItem>::new(pool.clone(), None, None);

    // Create sample data
    println!("📝 Creating sample data...\n");
    let users = create_sample_users(&user_store).await?;
    let orders = create_sample_orders(&order_store, &users).await?;
    create_sample_order_items(&order_item_store, &orders).await?;

    // Example 1: Simple aggregation queries
    println!("═══════════════════════════════════════════════════");
    println!("📈 Example 1: Simple Aggregation Queries");
    println!("═══════════════════════════════════════════════════\n");

    demo_simple_aggregations(&pool).await?;

    // Example 2: JOIN queries
    println!("\n═══════════════════════════════════════════════════");
    println!("🔗 Example 2: JOIN Queries");
    println!("═══════════════════════════════════════════════════\n");

    demo_join_queries(&pool).await?;

    // Example 3: GROUP BY with aggregations
    println!("\n═══════════════════════════════════════════════════");
    println!("📊 Example 3: GROUP BY with Aggregations");
    println!("═══════════════════════════════════════════════════\n");

    demo_group_by_queries(&pool).await?;

    // Example 4: Complex queries with HAVING
    println!("\n═══════════════════════════════════════════════════");
    println!("🎯 Example 4: Complex Queries with HAVING");
    println!("═══════════════════════════════════════════════════\n");

    demo_having_queries(&pool).await?;

    println!("\n✅ Demo completed successfully!");
    Ok(())
}

async fn create_sample_users(store: &GenericStore<User>) -> Result<Vec<User>, Box<dyn Error>> {
    let users = vec![
        User::new(
            Uuid::new_v4(),
            "Alice Johnson".to_string(),
            "alice@example.com".to_string(),
        ),
        User::new(
            Uuid::new_v4(),
            "Bob Smith".to_string(),
            "bob@example.com".to_string(),
        ),
        User::new(
            Uuid::new_v4(),
            "Charlie Brown".to_string(),
            "charlie@example.com".to_string(),
        ),
    ];

    let mut created_users = Vec::new();
    for user in users {
        let created = store.create(user, None).await?;
        created_users.push(created);
    }

    Ok(created_users)
}

async fn create_sample_orders(
    store: &GenericStore<Order>,
    users: &[User],
) -> Result<Vec<Order>, Box<dyn Error>> {
    let orders = vec![
        Order::new(
            Uuid::new_v4(),
            users[0].id,
            150.0,
            "completed".to_string(),
        ),
        Order::new(Uuid::new_v4(), users[0].id, 75.5, "pending".to_string()),
        Order::new(
            Uuid::new_v4(),
            users[1].id,
            200.0,
            "completed".to_string(),
        ),
        Order::new(
            Uuid::new_v4(),
            users[1].id,
            50.0,
            "cancelled".to_string(),
        ),
        Order::new(
            Uuid::new_v4(),
            users[2].id,
            300.0,
            "completed".to_string(),
        ),
    ];

    let mut created_orders = Vec::new();
    for order in orders {
        let created = store.create(order, None).await?;
        created_orders.push(created);
    }

    Ok(created_orders)
}

async fn create_sample_order_items(
    store: &GenericStore<OrderItem>,
    orders: &[Order],
) -> Result<(), Box<dyn Error>> {
    let items = vec![
        OrderItem::new(
            Uuid::new_v4(),
            orders[0].id,
            "Laptop".to_string(),
            1,
            150.0,
        ),
        OrderItem::new(
            Uuid::new_v4(),
            orders[1].id,
            "Mouse".to_string(),
            2,
            25.5,
        ),
        OrderItem::new(
            Uuid::new_v4(),
            orders[1].id,
            "Keyboard".to_string(),
            1,
            50.0,
        ),
        OrderItem::new(
            Uuid::new_v4(),
            orders[2].id,
            "Monitor".to_string(),
            2,
            100.0,
        ),
        OrderItem::new(
            Uuid::new_v4(),
            orders[3].id,
            "Cable".to_string(),
            5,
            10.0,
        ),
        OrderItem::new(
            Uuid::new_v4(),
            orders[4].id,
            "Desk".to_string(),
            1,
            300.0,
        ),
    ];

    for item in items {
        store.create(item, None).await?;
    }

    Ok(())
}

async fn demo_simple_aggregations(_pool: &PgPool) -> Result<(), Box<dyn Error>> {
    // Count all orders
    let query = QueryBuilder::new()
        .select(SelectField::count_all().with_alias("total_orders"));

    let (select_clause, _, where_clause, _, _, _, limit_clause, where_values, _) = query.build_full();
    let sql = format!(
        "SELECT {} FROM orders {} {}",
        select_clause, where_clause, limit_clause
    );

    println!("SQL: {}", sql);
    println!("Params: {:?}\n", where_values);

    // Sum of all order totals
    let query = QueryBuilder::new()
        .select(SelectField::sum("total_amount").with_alias("total_revenue"));

    let (select_clause, _, where_clause, _, _, _, limit_clause, where_values, _) = query.build_full();
    let sql = format!(
        "SELECT {} FROM orders {} {}",
        select_clause, where_clause, limit_clause
    );

    println!("SQL: {}", sql);
    println!("Params: {:?}\n", where_values);

    // Average order amount
    let query = QueryBuilder::new()
        .select(SelectField::avg("total_amount").with_alias("avg_order_value"));

    let (select_clause, _, where_clause, _, _, _, limit_clause, where_values, _) = query.build_full();
    let sql = format!(
        "SELECT {} FROM orders {} {}",
        select_clause, where_clause, limit_clause
    );

    println!("SQL: {}", sql);
    println!("Params: {:?}", where_values);

    Ok(())
}

async fn demo_join_queries(_pool: &PgPool) -> Result<(), Box<dyn Error>> {
    // INNER JOIN: Get orders with user information
    let query = QueryBuilder::new()
        .select_fields(vec![
            SelectField::field("users.name"),
            SelectField::field("users.email"),
            SelectField::field("orders.total_amount"),
            SelectField::field("orders.status"),
        ])
        .join(JoinClause::new_on(
            JoinType::Inner,
            "orders",
            "users.id",
            "orders.user_id",
        ))
        .filter(QueryFilter::eq("orders.status", json!("completed")))
        .order_by("orders.total_amount", SortOrder::Desc);

    let (select_clause, join_clause, where_clause, _, _, order_clause, limit_clause, where_values, _) = query.build_full();
    let sql = format!(
        "SELECT {} FROM users {} {} {} {} {}",
        select_clause, join_clause, where_clause, "", order_clause, limit_clause
    );

    println!("INNER JOIN Example:");
    println!("SQL: {}", sql);
    println!("Params: {:?}\n", where_values);

    // LEFT JOIN: Get all users with their order count
    let query = QueryBuilder::new()
        .select_fields(vec![
            SelectField::field("users.name"),
            SelectField::count("orders.id").with_alias("order_count"),
        ])
        .join(JoinClause::new_on(
            JoinType::Left,
            "orders",
            "users.id",
            "orders.user_id",
        ))
        .group_by(GroupBy::single("users.name"));

    let (select_clause, join_clause, where_clause, group_by_clause, _, order_clause, limit_clause, where_values, _) = query.build_full();
    let sql = format!(
        "SELECT {} FROM users {} {} {} {} {} {}",
        select_clause, join_clause, where_clause, group_by_clause, "", order_clause, limit_clause
    );

    println!("LEFT JOIN Example:");
    println!("SQL: {}", sql);
    println!("Params: {:?}", where_values);

    Ok(())
}

async fn demo_group_by_queries(_pool: &PgPool) -> Result<(), Box<dyn Error>> {
    // Group orders by status
    let query = QueryBuilder::new()
        .select_fields(vec![
            SelectField::field("status"),
            SelectField::count_all().with_alias("order_count"),
            SelectField::sum("total_amount").with_alias("total_revenue"),
            SelectField::avg("total_amount").with_alias("avg_order_value"),
        ])
        .group_by(GroupBy::single("status"))
        .order_by("total_revenue", SortOrder::Desc);

    let (select_clause, _, where_clause, group_by_clause, _, order_clause, limit_clause, where_values, _) = query.build_full();
    let sql = format!(
        "SELECT {} FROM orders {} {} {} {}",
        select_clause, where_clause, group_by_clause, order_clause, limit_clause
    );

    println!("GROUP BY Status:");
    println!("SQL: {}", sql);
    println!("Params: {:?}\n", where_values);

    // Group by user with order statistics
    let query = QueryBuilder::new()
        .select_fields(vec![
            SelectField::field("users.name"),
            SelectField::count("orders.id").with_alias("total_orders"),
            SelectField::sum("orders.total_amount").with_alias("total_spent"),
        ])
        .join(JoinClause::new_on(
            JoinType::Inner,
            "orders",
            "users.id",
            "orders.user_id",
        ))
        .group_by(GroupBy::new(vec!["users.name".to_string()]))
        .order_by("total_spent", SortOrder::Desc);

    let (select_clause, join_clause, where_clause, group_by_clause, _, order_clause, limit_clause, where_values, _) = query.build_full();
    let sql = format!(
        "SELECT {} FROM users {} {} {} {} {}",
        select_clause, join_clause, where_clause, group_by_clause, order_clause, limit_clause
    );

    println!("GROUP BY User:");
    println!("SQL: {}", sql);
    println!("Params: {:?}", where_values);

    Ok(())
}

async fn demo_having_queries(_pool: &PgPool) -> Result<(), Box<dyn Error>> {
    // Find users with more than 1 order
    let query = QueryBuilder::new()
        .select_fields(vec![
            SelectField::field("users.name"),
            SelectField::count("orders.id").with_alias("order_count"),
            SelectField::sum("orders.total_amount").with_alias("total_spent"),
        ])
        .join(JoinClause::new_on(
            JoinType::Inner,
            "orders",
            "users.id",
            "orders.user_id",
        ))
        .group_by(
            GroupBy::new(vec!["users.name".to_string()])
                .having(QueryFilter::gt("COUNT(orders.id)", json!(1))),
        )
        .order_by("order_count", SortOrder::Desc);

    let (select_clause, join_clause, where_clause, group_by_clause, having_clause, order_clause, limit_clause, where_values, having_values) = query.build_full();
    let sql = format!(
        "SELECT {} FROM users {} {} {} {} {} {}",
        select_clause, join_clause, where_clause, group_by_clause, having_clause, order_clause, limit_clause
    );

    println!("HAVING Example - Users with > 1 order:");
    println!("SQL: {}", sql);
    println!("WHERE Params: {:?}", where_values);
    println!("HAVING Params: {:?}\n", having_values);

    // Find order statuses with high total revenue
    let query = QueryBuilder::new()
        .select_fields(vec![
            SelectField::field("status"),
            SelectField::count_all().with_alias("order_count"),
            SelectField::sum("total_amount").with_alias("total_revenue"),
        ])
        .filter(QueryFilter::ne("status", json!("cancelled")))
        .group_by(
            GroupBy::single("status")
                .having(QueryFilter::gt("SUM(total_amount)", json!(100.0))),
        )
        .order_by("total_revenue", SortOrder::Desc);

    let (select_clause, _, where_clause, group_by_clause, having_clause, order_clause, limit_clause, where_values, having_values) = query.build_full();
    let sql = format!(
        "SELECT {} FROM orders {} {} {} {} {}",
        select_clause, where_clause, group_by_clause, having_clause, order_clause, limit_clause
    );

    println!("HAVING Example - High revenue statuses:");
    println!("SQL: {}", sql);
    println!("WHERE Params: {:?}", where_values);
    println!("HAVING Params: {:?}", having_values);

    Ok(())
}