use super::core::GenericStore;
use crate::errors::StorehausError;
use crate::table_metadata::TableMetadata;
use crate::traits::StoreObject;
use async_trait::async_trait;
use sqlx::Row;

#[async_trait]
impl<T> StoreObject for GenericStore<T>
where
    T: TableMetadata
        + for<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow>
        + sqlx::Type<sqlx::Postgres>
        + serde::Serialize
        + Unpin,
{
    type Model = T;
    type Id = T::Id;
    async fn create(
        &self,
        data: Self::Model,
    ) -> Result<Self::Model, StorehausError> {
        let created = data
            .execute_create(&self.db_pool)
            .await
            .map_err(|e| StorehausError::DatabaseError(e.to_string()))?;

        // Emit create signal if signal manager is present
        if self.signal_manager.is_some() {
            let mut event = signal_system::DatabaseEvent::new(
                signal_system::EventType::Create,
                T::table_name().to_string()
            );

            // Add created record as typed PostgresValue payload
            let payload = signal_system::serialize_to_postgres_payload(&created);
            for (key, value) in payload {
                event.add_payload(key, value);
            }

            self.emit_signal(event);
        }

        Ok(created)
    }

    async fn get_by_id(&self, id: &Self::Id) -> Result<Option<Self::Model>, StorehausError> {
        let id_str = format!("{:?}", id);

        // Try cache first if cache manager is present
        if let Some(cache_manager) = &self.cache_manager {
            let cache_prefix = self.get_cache_prefix();

            // Try to get from cache
            match cache_manager.get_record::<T>(&cache_prefix, &id_str).await {
                Ok(Some(cached)) => return Ok(Some(cached)),
                Ok(None) => {}, // Not in cache, continue to database
                Err(_) => {}, // Cache error, continue to database
            }
        }

        // Get from database
        let sql = format!("SELECT * FROM {} WHERE id = $1", T::table_name());
        let result = sqlx::query_as::<_, T>(&sql)
            .bind(id)
            .fetch_optional(&self.db_pool)
            .await
            .map_err(|e| StorehausError::DatabaseError(e.to_string()))?;

        // Cache the result if found and cache manager is present
        if let (Some(record), Some(cache_manager)) = (&result, &self.cache_manager) {
            let cache_prefix = self.get_cache_prefix();
            let cache_ttl = self.get_cache_ttl();

            // Store in cache (ignore errors)
            let _ = cache_manager.set_record_with_ttl(&cache_prefix, &id_str, record, cache_ttl).await;
        }

        Ok(result)
    }

    async fn list_all(&self) -> Result<Vec<Self::Model>, StorehausError> {
        let sql = format!("SELECT * FROM {} ORDER BY created_at DESC", T::table_name());
        let results = sqlx::query_as::<_, T>(&sql)
            .fetch_all(&self.db_pool)
            .await
            .map_err(|e| StorehausError::DatabaseError(e.to_string()))?;
        Ok(results)
    }

    async fn update(&self, id: &Self::Id, data: Self::Model) -> Result<Self::Model, StorehausError> {
        let updated = data
            .execute_update(&self.db_pool)
            .await
            .map_err(|e| StorehausError::DatabaseError(e.to_string()))?;

        // Emit update signal if signal manager is present
        if self.signal_manager.is_some() {
            let mut event = signal_system::DatabaseEvent::new(
                signal_system::EventType::Update,
                T::table_name().to_string()
            ).with_record_id(format!("{:?}", id));

            // Add the updated fields as individual typed fields
            let data_payload = signal_system::serialize_to_postgres_payload(&data);
            for (key, value) in data_payload {
                event.add_payload(key, value);
            }

            // Add the full record from database as __record__ associative array
            let full_record = signal_system::serialize_to_postgres_record(&updated);
            event.add_payload("__record__".to_string(), full_record);

            self.emit_signal(event);
        }

        // Invalidate cache after update
        if let Some(cache_manager) = &self.cache_manager {
            let cache_prefix = self.get_cache_prefix();
            let id_str = format!("{:?}", id);

            // Delete the specific record from cache
            let _ = cache_manager.delete_record(&cache_prefix, &id_str).await;

            // Invalidate all query caches for this table since data changed
            let _ = cache_manager.invalidate_queries(&cache_prefix).await;
        }

        Ok(updated)
    }

    async fn update_many(&self, updates: Vec<(Self::Id, Self::Model)>) -> Result<Vec<Self::Model>, StorehausError> {
        let mut results = Vec::new();
        let mut all_updated_data = Vec::new();

        // Use transaction for batch updates
        let mut tx = self.db_pool.begin().await
            .map_err(|e| StorehausError::DatabaseError(e.to_string()))?;

        for (id, data) in updates {
            let updated = data
                .execute_update_tx(&mut tx)
                .await
                .map_err(|e| StorehausError::DatabaseError(e.to_string()))?;

            all_updated_data.push((id, data, updated.clone()));
            results.push(updated);
        }

        tx.commit().await
            .map_err(|e| StorehausError::DatabaseError(e.to_string()))?;

        // Emit single batch update signal if signal manager is present
        if self.signal_manager.is_some() && !all_updated_data.is_empty() {
            let mut event = signal_system::DatabaseEvent::new(
                signal_system::EventType::Update,
                T::table_name().to_string()
            );

            // Combine all updated fields into unified payload
            let mut combined_payload = std::collections::HashMap::new();
            for (id, data, _) in &all_updated_data {
                let data_payload = signal_system::serialize_to_postgres_payload(&data);
                for (key, value) in data_payload {
                    // Use field name with record ID to avoid conflicts
                    let combined_key = format!("{}_{:?}", key, id);
                    combined_payload.insert(combined_key, value);
                }
            }

            // Add combined fields to event
            for (key, value) in combined_payload {
                event.add_payload(key, value);
            }

            // Add all full records as __record__ array
            let all_records: Vec<signal_system::PostgresValue> = all_updated_data
                .iter()
                .map(|(_, _, updated)| signal_system::serialize_to_postgres_record(updated))
                .collect();

            event.add_payload("__record__".to_string(), signal_system::PostgresValue::Json(
                serde_json::to_value(all_records).unwrap_or_default()
            ));

            self.emit_signal(event);
        }

        Ok(results)
    }

    async fn delete(&self, id: &Self::Id) -> Result<bool, StorehausError> {
        let sql = format!("DELETE FROM {} WHERE id = $1", T::table_name());
        let result = sqlx::query(&sql)
            .bind(id)
            .execute(&self.db_pool)
            .await
            .map_err(|e| StorehausError::DatabaseError(e.to_string()))?;

        let deleted = result.rows_affected() > 0;

        // Emit delete signal if signal manager is present
        if deleted && self.signal_manager.is_some() {
            let event = signal_system::DatabaseEvent::new(
                signal_system::EventType::Delete,
                T::table_name().to_string()
            ).with_record_id(format!("{:?}", id));
            self.emit_signal(event);
        }

        Ok(deleted)
    }

    async fn delete_many(&self, ids: Vec<Self::Id>) -> Result<Vec<Self::Id>, StorehausError> {
        let mut deleted_ids = Vec::new();

        // Use transaction for batch deletes
        let mut tx = self.db_pool.begin().await
            .map_err(|e| StorehausError::DatabaseError(e.to_string()))?;

        for id in ids {
            let sql = format!("DELETE FROM {} WHERE id = $1", T::table_name());
            let result = sqlx::query(&sql)
                .bind(&id)
                .execute(tx.as_mut())
                .await
                .map_err(|e| StorehausError::DatabaseError(e.to_string()))?;

            let was_deleted = result.rows_affected() > 0;

            if was_deleted {
                deleted_ids.push(id);
            }
        }

        tx.commit().await
            .map_err(|e| StorehausError::DatabaseError(e.to_string()))?;

        // Emit single batch delete signal if signal manager is present and records were deleted
        if self.signal_manager.is_some() && !deleted_ids.is_empty() {
            let mut event = signal_system::DatabaseEvent::new(
                signal_system::EventType::Delete,
                T::table_name().to_string()
            );

            // Get the primary key field name from TableMetadata
            let primary_key_name = T::primary_key_field();

            // Convert deleted IDs to PostgresValue array
            let ids_as_postgres: Vec<signal_system::PostgresValue> = deleted_ids
                .iter()
                .map(|id| {
                    // Try to convert ID to appropriate PostgresValue type
                    let id_str = format!("{:?}", id);
                    if id_str.starts_with('"') && id_str.ends_with('"') {
                        // String/UUID
                        signal_system::PostgresValue::Text(id_str.trim_matches('"').to_string())
                    } else if let Ok(int_id) = id_str.parse::<i32>() {
                        signal_system::PostgresValue::Integer(int_id)
                    } else if let Ok(bigint_id) = id_str.parse::<i64>() {
                        signal_system::PostgresValue::BigInt(bigint_id)
                    } else {
                        // Fallback to text
                        signal_system::PostgresValue::Text(id_str)
                    }
                })
                .collect();

            event.add_payload(primary_key_name.to_string(), signal_system::PostgresValue::Json(
                serde_json::to_value(ids_as_postgres).unwrap_or_default()
            ));

            self.emit_signal(event);
        }

        Ok(deleted_ids)
    }

    async fn count(&self) -> Result<i64, StorehausError> {
        let sql = format!("SELECT COUNT(*) as total FROM {}", T::table_name());
        let result = sqlx::query(&sql)
            .fetch_one(&self.db_pool)
            .await
            .map_err(|e| StorehausError::DatabaseError(e.to_string()))?;

        let total: i64 = result.get("total");
        Ok(total)
    }

    async fn find(&self, query: crate::QueryBuilder) -> Result<Vec<Self::Model>, StorehausError> {
        let base_sql = format!("SELECT * FROM {}", T::table_name());
        let (where_clause, order_clause, limit_clause, params) = query.build();
        let full_sql = format!("{}{}{}{}", base_sql, where_clause, order_clause, limit_clause);

        let mut sqlx_query = sqlx::query_as::<_, T>(&full_sql);
        for param in params {
            sqlx_query = self.bind_param(sqlx_query, param);
        }

        let results = sqlx_query
            .fetch_all(&self.db_pool)
            .await
            .map_err(|e| StorehausError::DatabaseError(e.to_string()))?;

        Ok(results)
    }

    async fn find_one(&self, query: crate::QueryBuilder) -> Result<Option<Self::Model>, StorehausError> {
        let query_with_limit = query.limit(1);
        let mut results = self.find(query_with_limit).await?;

        Ok(results.pop())
    }

    async fn update_where(&self, query: crate::QueryBuilder, data: Self::Model) -> Result<Vec<Self::Model>, StorehausError> {
        // First find all records that match the query to get their IDs
        let matching_records = self.find(query).await?;

        if matching_records.is_empty() {
            return Ok(Vec::new());
        }

        // Extract IDs and create update pairs
        let updates: Vec<(T::Id, T)> = matching_records
            .into_iter()
            .map(|record| {
                let id = record.extract_id();
                (id, data.clone())
            })
            .collect();

        // Use existing update_many functionality
        self.update_many(updates).await
    }

    async fn delete_where(&self, query: crate::QueryBuilder) -> Result<Vec<Self::Id>, StorehausError> {
        // First find all records that match the query to get their IDs
        let matching_records = self.find(query).await?;

        if matching_records.is_empty() {
            return Ok(Vec::new());
        }

        // Extract IDs
        let ids: Vec<T::Id> = matching_records
            .into_iter()
            .map(|record| record.extract_id())
            .collect();

        // Use existing delete_many functionality
        self.delete_many(ids).await
    }

    async fn count_where(&self, query: crate::QueryBuilder) -> Result<i64, StorehausError> {
        let base_sql = format!("SELECT COUNT(*) as total FROM {}", T::table_name());
        let (where_clause, _, _, params) = query.build(); // No ORDER BY or LIMIT for COUNT
        let full_sql = format!("{}{}", base_sql, where_clause);

        let mut sqlx_query = sqlx::query(&full_sql);
        for param in params {
            sqlx_query = self.bind_param_raw(sqlx_query, param);
        }

        let result = sqlx_query
            .fetch_one(&self.db_pool)
            .await
            .map_err(|e| StorehausError::DatabaseError(e.to_string()))?;

        let total: i64 = result.get("total");
        Ok(total)
    }
}

// Helper implementation for parameter binding
impl<T: TableMetadata> GenericStore<T>
where
    T: TableMetadata
        + for<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow>
        + sqlx::Type<sqlx::Postgres>
        + serde::Serialize
        + Unpin,
{
    fn bind_param<'q>(&self, query: sqlx::query::QueryAs<'q, sqlx::Postgres, T, sqlx::postgres::PgArguments>, param: serde_json::Value) -> sqlx::query::QueryAs<'q, sqlx::Postgres, T, sqlx::postgres::PgArguments> {
        match param {
            serde_json::Value::String(s) => query.bind(s),
            serde_json::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    if i >= i32::MIN as i64 && i <= i32::MAX as i64 {
                        query.bind(i as i32)
                    } else {
                        query.bind(i)
                    }
                } else if let Some(f) = n.as_f64() {
                    query.bind(f)
                } else {
                    query.bind(n.to_string())
                }
            }
            serde_json::Value::Bool(b) => query.bind(b),
            serde_json::Value::Null => query.bind(Option::<String>::None),
            other => query.bind(other.to_string()),
        }
    }

    fn bind_param_raw<'q>(&self, query: sqlx::query::Query<'q, sqlx::Postgres, sqlx::postgres::PgArguments>, param: serde_json::Value) -> sqlx::query::Query<'q, sqlx::Postgres, sqlx::postgres::PgArguments> {
        match param {
            serde_json::Value::String(s) => query.bind(s),
            serde_json::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    if i >= i32::MIN as i64 && i <= i32::MAX as i64 {
                        query.bind(i as i32)
                    } else {
                        query.bind(i)
                    }
                } else if let Some(f) = n.as_f64() {
                    query.bind(f)
                } else {
                    query.bind(n.to_string())
                }
            }
            serde_json::Value::Bool(b) => query.bind(b),
            serde_json::Value::Null => query.bind(Option::<String>::None),
            other => query.bind(other.to_string()),
        }
    }
}