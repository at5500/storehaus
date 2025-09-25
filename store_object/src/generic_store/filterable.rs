use super::core::GenericStore;
use crate::errors::StorehausError;
use crate::table_metadata::TableMetadata;
use crate::traits::{Filterable, StoreFilter};
use async_trait::async_trait;

#[async_trait]
impl<T> Filterable for GenericStore<T>
where
    T: TableMetadata
        + for<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow>
        + sqlx::Type<sqlx::Postgres>
        + serde::Serialize
        + Unpin,
{
    async fn list_by_filter(&self, filter: &StoreFilter) -> Result<Vec<Self::Model>, StorehausError> {
        let base_query = format!("SELECT * FROM {}", T::table_name());
        let (where_clause, values) = filter.build_where_clause();
        let order_clause = " ORDER BY created_at DESC";

        let full_query = format!("{}{}{}", base_query, where_clause, order_clause);

        let mut query_builder = sqlx::query_as::<_, T>(&full_query);

        for value in values {
            query_builder = query_builder.bind(value);
        }

        let results = query_builder
            .fetch_all(&self.db_pool)
            .await
            .map_err(|e| StorehausError::DatabaseError(e.to_string()))?;

        Ok(results)
    }
}