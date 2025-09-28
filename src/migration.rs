//! Database migration functionality
//!
//! This module provides automatic table creation and migration utilities
//! for StoreObject types managed by StoreHaus.

use crate::core::StoreHaus;
use crate::errors::StoreHausError;
use store_object::traits::{StoreObject, TableMetadata};

impl StoreHaus {
    /// Automatically create table and indexes for a model
    /// If recreate is true, drops existing table first
    pub async fn auto_migrate<T>(&self, recreate: bool) -> Result<(), StoreHausError>
    where
        T: TableMetadata + Send + Sync,
    {
        let table_name = T::table_name();

        // Drop table if recreate is requested
        if recreate {
            let drop_sql = T::drop_table_sql();
            println!("Dropping table with SQL: {}", drop_sql);
            sqlx::query(&drop_sql).execute(self.pool()).await?;
        }

        // Create the table
        let create_table_sql = T::create_table_sql();
        println!("Creating table with SQL: {}", create_table_sql);
        sqlx::query(&create_table_sql).execute(self.pool()).await?;

        // Create __updated_at__ trigger function if it doesn't exist
        let trigger_function_sql = r#"
        CREATE OR REPLACE FUNCTION update_updated_at_column()
        RETURNS TRIGGER AS $$
        BEGIN
            NEW.__updated_at__ = NOW();
            RETURN NEW;
        END;
        $$ language 'plpgsql';
        "#;
        sqlx::query(trigger_function_sql)
            .execute(self.pool())
            .await?;

        // Create __updated_at__ trigger for this table
        let trigger_sql = format!(
            "CREATE TRIGGER update_{}_updated_at
             BEFORE UPDATE ON {}
             FOR EACH ROW
             EXECUTE FUNCTION update_updated_at_column()",
            table_name, table_name
        );
        // Use IF NOT EXISTS equivalent for triggers
        let trigger_check_sql = format!(
            "DO $$
             BEGIN
                 IF NOT EXISTS (SELECT 1 FROM pg_trigger WHERE tgname = 'update_{}_updated_at') THEN
                     EXECUTE '{}';
                 END IF;
             END $$",
            table_name, trigger_sql
        );
        sqlx::query(&trigger_check_sql).execute(self.pool()).await?;

        // Create indexes
        let indexes = T::create_indexes_sql();
        for index_sql in indexes {
            println!("Creating index with SQL: {}", index_sql);
            sqlx::query(&index_sql).execute(self.pool()).await?;
        }

        Ok(())
    }

    /// Register store and auto-migrate its table
    pub async fn register_store_with_migration<T>(
        &mut self,
        name: String,
        store: T,
        recreate: bool,
    ) -> Result<(), StoreHausError>
    where
        T: StoreObject + TableMetadata + Send + Sync + 'static,
    {
        // First, run auto-migration for this type
        self.auto_migrate::<T>(recreate).await?;

        // Then register the store
        self.register_store(name, store)
    }
}
