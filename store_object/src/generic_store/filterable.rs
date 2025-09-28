//! Generic store implementations
//!
//! This module provides generic database store functionality.

use super::core::GenericStore;
use crate::errors::StorehausError;
use crate::table_metadata::TableMetadata;
use crate::traits::{Filterable, StoreFilter};
use async_trait::async_trait;

#[async_trait]
impl<T> Filterable for GenericStore<T>
where
    T: TableMetadata
        + crate::traits::table_metadata::DatabaseExecutor
        + for<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow>
        + sqlx::Type<sqlx::Postgres>
        + serde::Serialize
        + Unpin,
{
    async fn list_by_filter(
        &self,
        filter: &StoreFilter,
    ) -> Result<Vec<Self::Model>, StorehausError> {
        let table_name = T::table_name();
        let (where_clause, values) = filter.build_where_clause();
        let order_clause = " ORDER BY __created_at__ DESC";

        // Pre-allocate capacity to avoid reallocations
        let mut full_query = String::with_capacity(64 + table_name.len() + where_clause.len());
        full_query.push_str("SELECT * FROM ");
        full_query.push_str(table_name);
        full_query.push_str(&where_clause);
        full_query.push_str(order_clause);

        let mut query_builder = sqlx::query_as::<_, T>(&full_query);

        for value in values {
            query_builder = query_builder.bind(value);
        }

        let results = query_builder.fetch_all(&self.db_pool).await.map_err(|e| {
            StorehausError::database_operation(T::table_name(), "list_by_filter", e)
        })?;

        Ok(results)
    }
}
