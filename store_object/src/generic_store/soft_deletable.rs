use super::core::GenericStore;
use crate::errors::StorehausError;
use crate::table_metadata::TableMetadata;
use crate::traits::SoftDeletable;
use async_trait::async_trait;
use sqlx::Row;

#[async_trait]
impl<T> SoftDeletable for GenericStore<T>
where
    T: TableMetadata
        + for<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow>
        + sqlx::Type<sqlx::Postgres>
        + serde::Serialize
        + Unpin,
{
    async fn list_active(&self) -> Result<Vec<Self::Model>, StorehausError> {
        if !T::supports_soft_delete() {
            return Err(StorehausError::ValidationError(
                "Entity does not support soft deletion".to_string(),
            ));
        }

        let sql = format!(
            "SELECT * FROM {} WHERE is_active = true ORDER BY created_at DESC",
            T::table_name()
        );
        let results = sqlx::query_as::<_, T>(&sql)
            .fetch_all(&self.db_pool)
            .await
            .map_err(|e| StorehausError::DatabaseError(e.to_string()))?;
        Ok(results)
    }

    async fn set_active(&self, id: &Self::Id, is_active: bool) -> Result<bool, StorehausError> {
        if !T::supports_soft_delete() {
            return Err(StorehausError::ValidationError(
                "Entity does not support soft deletion".to_string(),
            ));
        }

        let sql = format!(
            "UPDATE {} SET is_active = $1, updated_at = NOW() WHERE id = $2",
            T::table_name()
        );
        let result = sqlx::query(&sql)
            .bind(is_active)
            .bind(id)
            .execute(&self.db_pool)
            .await
            .map_err(|e| StorehausError::DatabaseError(e.to_string()))?;

        Ok(result.rows_affected() > 0)
    }

    async fn count_active(&self) -> Result<i64, StorehausError> {
        if !T::supports_soft_delete() {
            return Err(StorehausError::ValidationError(
                "Entity does not support soft deletion".to_string(),
            ));
        }

        let sql = format!(
            "SELECT COUNT(*) as active FROM {} WHERE is_active = true",
            T::table_name()
        );
        let result = sqlx::query(&sql)
            .fetch_one(&self.db_pool)
            .await
            .map_err(|e| StorehausError::DatabaseError(e.to_string()))?;

        let active: i64 = result.get("active");
        Ok(active)
    }
}