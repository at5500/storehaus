//! Generic store implementations
//!
//! This module provides generic database store functionality.

use super::core::GenericStore;
use crate::errors::StorehausError;
use crate::id_type::HasUniversalId;
use crate::table_metadata::TableMetadata;
use crate::traits::table_metadata::DatabaseExecutor;
use crate::traits::StoreObject;
use async_trait::async_trait;
use sqlx::Row;

/// Helper function to efficiently convert any ID to UniversalId and then to string
#[inline]
fn id_to_string<ID: HasUniversalId>(id: ID) -> String {
    id.universal_id().to_string_fast()
}

#[async_trait]
impl<T> StoreObject for GenericStore<T>
where
    T: TableMetadata
        + DatabaseExecutor
        + for<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow>
        + serde::Serialize
        + Unpin,
{
    type Model = T;
    type Id = T::Id;
    async fn create(
        &self,
        data: Self::Model,
        tags: Option<Vec<String>>,
    ) -> Result<Self::Model, StorehausError> {
        let created = data.execute_create(&self.db_pool).await?;

        // Emit create signal if signal manager is present
        if self.signal_manager.is_some() {
            let event = if let Some(tags) = tags {
                self.create_event_with_explicit_tags(
                    signal_system::EventType::Create,
                    T::table_name().to_string(),
                    &created,
                    None,
                    tags,
                )
            } else {
                self.create_event_with_tags(
                    signal_system::EventType::Create,
                    T::table_name().to_string(),
                    &created,
                    None,
                )
            };
            self.emit_signal(event).await;
        }

        Ok(created)
    }

    async fn get_by_id(&self, id: &Self::Id) -> Result<Option<Self::Model>, StorehausError> {
        // Try cache first if cache manager is present
        if let Some(cache_manager) = self.cache_manager() {
            let cache_prefix = self.get_cache_prefix();

            // Use a pre-allocated buffer to avoid repeated allocations
            use std::fmt::Write;
            let mut id_buffer = String::with_capacity(64); // Pre-allocate reasonable size
            let _ = write!(id_buffer, "{:?}", id);

            // Try to get from cache
            match cache_manager
                .get_record::<T>(cache_prefix, T::table_name(), &id_buffer)
                .await
            {
                Ok(Some(cached)) => return Ok(Some(cached)),
                Ok(None) => {} // Not in cache, continue to database
                Err(e) => {
                    return Err(StorehausError::cache_operation(
                        "get_by_id",
                        Some(&id_buffer),
                        Box::new(e),
                    ))
                }
            }
        }

        // Get from database - use static SQL to avoid allocations
        let result = sqlx::query_as::<_, T>(T::get_by_id_sql())
            .bind(id)
            .fetch_optional(&self.db_pool)
            .await
            .map_err(|e| StorehausError::query_execution(T::table_name(), T::get_by_id_sql(), e))?;

        // Cache the result if found and cache manager is present
        if let (Some(record), Some(cache_manager)) = (&result, self.cache_manager()) {
            let cache_prefix = self.get_cache_prefix();
            let cache_ttl = self.get_cache_ttl();

            // Reuse the same buffer approach for consistency
            use std::fmt::Write;
            let mut id_buffer = String::with_capacity(64);
            let _ = write!(id_buffer, "{:?}", id);

            // Store in cache
            if let Err(e) = cache_manager
                .set_record_with_ttl(
                    cache_prefix,
                    T::table_name(),
                    &id_buffer,
                    record,
                    cache_ttl,
                )
                .await
            {
                return Err(StorehausError::cache_operation(
                    "get_by_id_cache_set",
                    Some(&id_buffer),
                    Box::new(e),
                ));
            }
        }

        Ok(result)
    }

    async fn list_all(&self) -> Result<Vec<Self::Model>, StorehausError> {
        // Use static SQL from table_derive - no allocations needed
        let results = sqlx::query_as::<_, T>(T::list_all_sql())
            .fetch_all(&self.db_pool)
            .await
            .map_err(|e| StorehausError::database_operation(T::table_name(), "list_all", e))?;
        Ok(results)
    }

    async fn update(
        &self,
        id: &Self::Id,
        data: Self::Model,
        tags: Option<Vec<String>>,
    ) -> Result<Self::Model, StorehausError> {
        let updated = data.execute_update(&self.db_pool).await?;

        // Emit update signal if signal manager is present
        if self.signal_manager.is_some() {
            let mut event = if let Some(tags) = tags {
                self.create_event_with_explicit_tags(
                    signal_system::EventType::Update,
                    T::table_name().to_string(),
                    &updated,
                    Some(id_to_string(id.clone())),
                    tags,
                )
            } else {
                self.create_event_with_tags(
                    signal_system::EventType::Update,
                    T::table_name().to_string(),
                    &updated,
                    Some(id_to_string(id.clone())),
                )
            };

            // Add the full record from database as __record__ associative array
            let full_record = signal_system::serialize_to_postgres_record(&updated);
            event.add_payload("__record__".to_string(), full_record);

            self.emit_signal(event).await;
        }

        // Invalidate cache after update
        if let Some(cache_manager) = self.cache_manager() {
            let cache_prefix = self.get_cache_prefix();
            let id_str = id_to_string(id.clone());

            // Delete the specific record from cache
            let _ = cache_manager
                .delete_record(cache_prefix, T::table_name(), &id_str)
                .await;

            // Invalidate all query caches for this table since data changed
            let _ = cache_manager
                .invalidate_queries(cache_prefix, T::table_name())
                .await;
        }

        Ok(updated)
    }

    async fn update_many(
        &self,
        updates: Vec<(Self::Id, Self::Model)>,
    ) -> Result<Vec<Self::Model>, StorehausError> {
        let mut results = Vec::new();
        let mut all_updated_data = Vec::new();

        // Use transaction for batch updates
        let mut tx = self
            .db_pool
            .begin()
            .await
            .map_err(|e| StorehausError::database_operation(T::table_name(), "query", e))?;

        for (id, data) in updates {
            let updated = data.execute_update_tx(&mut tx).await?;

            results.push(updated.clone());
            all_updated_data.push((id, data, updated));
        }

        tx.commit()
            .await
            .map_err(|e| StorehausError::database_operation(T::table_name(), "query", e))?;

        // Emit single batch update signal if signal manager is present
        if self.signal_manager.is_some() && !all_updated_data.is_empty() {
            let mut event = signal_system::DatabaseEvent::new(
                signal_system::EventType::Update,
                T::table_name().to_string(),
            );

            // Combine all updated fields into unified payload
            let mut combined_payload = std::collections::HashMap::new();
            for (id, data, _) in &all_updated_data {
                let data_payload = signal_system::serialize_to_postgres_payload(&data);
                for (key, value) in data_payload {
                    // Use field name with record ID to avoid conflicts
                    let combined_key = format!("{}_{}", key, id_to_string(id.clone()));
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

            event.add_payload(
                "__record__".to_string(),
                signal_system::PostgresValue::Json(
                    serde_json::to_value(all_records).unwrap_or_default(),
                ),
            );

            self.emit_signal(event).await;
        }

        Ok(results)
    }

    async fn delete(&self, id: &Self::Id) -> Result<bool, StorehausError> {
        // If model supports soft delete, use soft delete instead of hard delete
        if T::supports_soft_delete() {
            use crate::traits::SoftDeletable;
            return self.set_active(id, false).await;
        }

        // Hard delete for models without soft delete support
        let result = sqlx::query(T::delete_by_id_sql())
            .bind(id)
            .execute(&self.db_pool)
            .await
            .map_err(|e| StorehausError::database_operation(T::table_name(), "query", e))?;

        let deleted = result.rows_affected() > 0;

        // Emit delete signal if signal manager is present
        if deleted && self.signal_manager.is_some() {
            let event = signal_system::DatabaseEvent::new(
                signal_system::EventType::Delete,
                T::table_name().to_string(),
            )
            .with_record_id(id_to_string(id.clone()));
            self.emit_signal(event).await;
        }

        Ok(deleted)
    }

    async fn delete_many(&self, ids: Vec<Self::Id>) -> Result<Vec<Self::Id>, StorehausError> {
        // If model supports soft delete, use soft delete for all IDs
        if T::supports_soft_delete() {
            use crate::traits::SoftDeletable;
            let mut deleted_ids = Vec::new();
            for id in ids {
                if self.set_active(&id, false).await? {
                    deleted_ids.push(id);
                }
            }
            return Ok(deleted_ids);
        }

        // Hard delete for models without soft delete support
        let mut deleted_ids = Vec::new();

        // Use transaction for batch deletes
        let mut tx = self
            .db_pool
            .begin()
            .await
            .map_err(|e| StorehausError::database_operation(T::table_name(), "query", e))?;

        for id in ids {
            let result = sqlx::query(T::delete_by_id_sql())
                .bind(&id)
                .execute(tx.as_mut())
                .await
                .map_err(|e| StorehausError::database_operation(T::table_name(), "query", e))?;

            let was_deleted = result.rows_affected() > 0;

            if was_deleted {
                deleted_ids.push(id);
            }
        }

        tx.commit()
            .await
            .map_err(|e| StorehausError::database_operation(T::table_name(), "query", e))?;

        // Emit single batch delete signal if signal manager is present and records were deleted
        if self.signal_manager.is_some() && !deleted_ids.is_empty() {
            let mut event = signal_system::DatabaseEvent::new(
                signal_system::EventType::Delete,
                T::table_name().to_string(),
            );

            // Get the primary key field name from TableMetadata
            let primary_key_name = T::primary_key_field();

            // Convert deleted IDs to PostgresValue array
            let ids_as_postgres: Vec<signal_system::PostgresValue> = deleted_ids
                .iter()
                .map(|id| {
                    // Try to convert ID to appropriate PostgresValue type
                    let id_str = id_to_string(id.clone());
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

            event.add_payload(
                primary_key_name.to_string(),
                signal_system::PostgresValue::Json(
                    serde_json::to_value(ids_as_postgres).unwrap_or_default(),
                ),
            );

            self.emit_signal(event).await;
        }

        Ok(deleted_ids)
    }

    async fn count(&self) -> Result<i64, StorehausError> {
        let result = sqlx::query(T::count_all_sql())
            .fetch_one(&self.db_pool)
            .await
            .map_err(|e| StorehausError::database_operation(T::table_name(), "query", e))?;

        let total: i64 = result.get("total");
        Ok(total)
    }

    async fn find(&self, query: crate::QueryBuilder) -> Result<Vec<Self::Model>, StorehausError> {
        let (where_clause, order_clause, limit_clause, params) = query.build();
        // Avoid format! allocation by building string directly
        let base_sql = T::select_base_sql();
        let mut full_sql = String::with_capacity(
            base_sql.len() + where_clause.len() + order_clause.len() + limit_clause.len(),
        );
        full_sql.push_str(base_sql);
        if !where_clause.is_empty() {
            full_sql.push(' ');
            // If base_sql already has WHERE (soft delete), replace WHERE with AND
            if base_sql.contains(" WHERE ") && where_clause.starts_with("WHERE ") {
                full_sql.push_str("AND ");
                full_sql.push_str(&where_clause[6..]); // Skip "WHERE "
            } else {
                full_sql.push_str(&where_clause);
            }
        }
        if !order_clause.is_empty() {
            full_sql.push(' ');
            full_sql.push_str(&order_clause);
        }
        if !limit_clause.is_empty() {
            full_sql.push(' ');
            full_sql.push_str(&limit_clause);
        }

        let mut sqlx_query = sqlx::query_as::<_, T>(&full_sql);
        for param in params {
            sqlx_query = self.bind_param(sqlx_query, param);
        }

        let results = sqlx_query
            .fetch_all(&self.db_pool)
            .await
            .map_err(|e| StorehausError::database_operation(T::table_name(), "query", e))?;

        Ok(results)
    }

    async fn find_one(
        &self,
        query: crate::QueryBuilder,
    ) -> Result<Option<Self::Model>, StorehausError> {
        let query_with_limit = query.limit(1);
        let mut results = self.find(query_with_limit).await?;

        Ok(results.pop())
    }

    async fn update_where(
        &self,
        query: crate::QueryBuilder,
        data: Option<Self::Model>,
    ) -> Result<Vec<Self::Model>, StorehausError> {
        // Build the WHERE clause from the query
        let (where_clause, _, _, params) = query.build();

        // Check if query has custom update operations
        let (set_clause, update_values, num_update_params) = if let Some(updates) = query.get_updates() {
            // Use custom update operations (e.g., increment, decrement)
            let mut assignments = Vec::new();
            let mut values = Vec::new();
            let mut param_num = 1;

            // Sort operations by field name for consistent ordering
            let mut ops: Vec<_> = updates.operations.iter().collect();
            ops.sort_by(|a, b| a.0.cmp(b.0));

            for (field_name, operation) in ops {
                let sql_expr = operation.to_sql(field_name, param_num);
                assignments.push(sql_expr);
                values.push(operation.value().clone());
                param_num += 1;
            }

            let set_clause = assignments.join(", ");
            (set_clause, values, param_num - 1)
        } else {
            // Use legacy approach: extract all update fields from the model
            let update_fields = T::update_fields();
            let set_clause = update_fields
                .iter()
                .enumerate()
                .map(|(i, field)| {
                    let mut assignment = String::with_capacity(field.len() + 8);
                    assignment.push_str(field);
                    assignment.push_str(" = $");
                    assignment.push_str(&(i + 1).to_string());
                    assignment
                })
                .collect::<Vec<_>>()
                .join(", ");

            (set_clause, Vec::new(), update_fields.len())
        };

        // Adjust WHERE clause parameter numbers to come after UPDATE parameters
        let mut adjusted_where_clause = if !params.is_empty() {
            let mut where_str = where_clause.clone();
            // Replace $1, $2, ... in WHERE clause with correct numbers after UPDATE params
            for i in (1..=params.len()).rev() {
                let old_param = format!("${}", i);
                let new_param = format!("${}", num_update_params + i);
                where_str = where_str.replace(&old_param, &new_param);
            }
            where_str
        } else {
            where_clause.clone()
        };

        // Add soft delete filter for models with soft delete support
        if T::supports_soft_delete() {
            if let Some(soft_delete_field) = T::soft_delete_field() {
                if adjusted_where_clause.is_empty() {
                    adjusted_where_clause = format!("WHERE {} = TRUE", soft_delete_field);
                } else {
                    adjusted_where_clause = format!("{} AND {} = TRUE", adjusted_where_clause, soft_delete_field);
                }
            }
        }

        // Build UPDATE statement with RETURNING clause to get updated records
        let sql = format!(
            "UPDATE {} SET {}, __updated_at__ = NOW() {} RETURNING *",
            T::table_name(),
            set_clause,
            adjusted_where_clause
        );

        tracing::debug!("[UPDATE_WHERE] Table: {}", T::table_name());
        tracing::debug!("[UPDATE_WHERE] SQL: {}", sql);
        tracing::debug!("[UPDATE_WHERE] WHERE params count: {}", params.len());
        tracing::debug!("[UPDATE_WHERE] UPDATE params count: {}", num_update_params);
        tracing::debug!("[UPDATE_WHERE] Using custom updates: {}", query.has_updates());

        // Bind parameters and execute based on which mode we're using
        let updated_records = if query.has_updates() {
            // Custom updates mode: bind the update values manually
            let mut q = sqlx::query_as::<_, T>(&sql);
            for value in update_values {
                q = self.bind_param(q, value);
            }

            // Then bind WHERE clause parameters
            let mut sqlx_query = q;
            for param in params {
                sqlx_query = self.bind_param(sqlx_query, param);
            }

            // Execute the query
            sqlx_query
                .fetch_all(&self.db_pool)
                .await
                .map_err(|e| StorehausError::database_operation(T::table_name(), "query", e))?
        } else {
            // Legacy mode: use model's bind method
            let model = data.ok_or_else(|| {
                StorehausError::InvalidConfiguration {
                    message: "update_where called without data and without UpdateSet".to_string(),
                }
            })?;

            let mut sqlx_query = model.bind_update_params_owned(&sql);

            // Then bind WHERE clause parameters
            for param in params {
                sqlx_query = self.bind_param(sqlx_query, param);
            }

            // Execute the query
            sqlx_query
                .fetch_all(&self.db_pool)
                .await
                .map_err(|e| StorehausError::database_operation(T::table_name(), "query", e))?
        };

        // Emit signals for updated records if signal manager is present
        if self.signal_manager.is_some() && !updated_records.is_empty() {
            let mut event = signal_system::DatabaseEvent::new(
                signal_system::EventType::Update,
                T::table_name().to_string(),
            );

            // Add all full records as __record__ array
            let all_records: Vec<signal_system::PostgresValue> = updated_records
                .iter()
                .map(|record| signal_system::serialize_to_postgres_record(record))
                .collect();

            event.add_payload(
                "__record__".to_string(),
                signal_system::PostgresValue::Json(
                    serde_json::to_value(all_records).unwrap_or_default(),
                ),
            );

            // Add count of updated records
            event.add_payload(
                "updated_count".to_string(),
                signal_system::PostgresValue::Integer(updated_records.len() as i32),
            );

            self.emit_signal(event).await;
        }

        // Invalidate cache for all updated records
        if let Some(cache_manager) = self.cache_manager() {
            let cache_prefix = self.get_cache_prefix();

            for record in &updated_records {
                let id_str = id_to_string(record.extract_id());
                let _ = cache_manager
                    .delete_record(cache_prefix, T::table_name(), &id_str)
                    .await;
            }

            // Invalidate all query caches for this table since data changed
            let _ = cache_manager
                .invalidate_queries(cache_prefix, T::table_name())
                .await;
        }

        Ok(updated_records)
    }

    /// Update records matching query using a custom executor (for transactions)
    ///
    /// This is identical to `update_where` but accepts any executor (Pool or Transaction).
    /// Use this when you need to execute updates within a database transaction.
    ///
    /// # Example
    /// ```ignore
    /// let mut tx = pool.begin().await?;
    ///
    /// let query = QueryBuilder::new()
    ///     .filter(QueryFilter::eq("wallet_id", json!(wallet_id)))
    ///     .update(UpdateSet::new().decrement("balance", json!(amount)));
    ///
    /// store.update_where_with_executor(&mut tx, query, None).await?;
    ///
    /// tx.commit().await?;
    /// ```
    async fn update_where_with_executor<'e, E>(
        &self,
        executor: E,
        query: crate::QueryBuilder,
        data: Option<Self::Model>,
    ) -> Result<Vec<Self::Model>, StorehausError>
    where
        E: sqlx::Executor<'e, Database = sqlx::Postgres>,
    {
        // Build the WHERE clause from the query
        let (where_clause, _, _, params) = query.build();

        // Check if query has custom update operations
        let (set_clause, update_values, num_update_params) = if let Some(updates) = query.get_updates() {
            // Use custom update operations (e.g., increment, decrement)
            let mut assignments = Vec::new();
            let mut values = Vec::new();
            let mut param_num = 1;

            // Sort operations by field name for consistent ordering
            let mut ops: Vec<_> = updates.operations.iter().collect();
            ops.sort_by(|a, b| a.0.cmp(b.0));

            for (field_name, operation) in ops {
                let sql_expr = operation.to_sql(field_name, param_num);
                assignments.push(sql_expr);
                values.push(operation.value().clone());
                param_num += 1;
            }

            let set_clause = assignments.join(", ");
            (set_clause, values, param_num - 1)
        } else {
            // Use legacy approach: extract all update fields from the model
            let update_fields = T::update_fields();
            let set_clause = update_fields
                .iter()
                .enumerate()
                .map(|(i, field)| {
                    let mut assignment = String::with_capacity(field.len() + 8);
                    assignment.push_str(field);
                    assignment.push_str(" = $");
                    assignment.push_str(&(i + 1).to_string());
                    assignment
                })
                .collect::<Vec<_>>()
                .join(", ");

            (set_clause, Vec::new(), update_fields.len())
        };

        // Adjust WHERE clause parameter numbers to come after UPDATE parameters
        let mut adjusted_where_clause = if !params.is_empty() {
            let mut where_str = where_clause.clone();
            // Replace $1, $2, ... in WHERE clause with correct numbers after UPDATE params
            for i in (1..=params.len()).rev() {
                let old_param = format!("${}", i);
                let new_param = format!("${}", num_update_params + i);
                where_str = where_str.replace(&old_param, &new_param);
            }
            where_str
        } else {
            where_clause.clone()
        };

        // Add soft delete filter for models with soft delete support
        if T::supports_soft_delete() {
            if let Some(soft_delete_field) = T::soft_delete_field() {
                if adjusted_where_clause.is_empty() {
                    adjusted_where_clause = format!("WHERE {} = TRUE", soft_delete_field);
                } else {
                    adjusted_where_clause = format!("{} AND {} = TRUE", adjusted_where_clause, soft_delete_field);
                }
            }
        }

        // Build UPDATE statement with RETURNING clause to get updated records
        let sql = format!(
            "UPDATE {} SET {}, __updated_at__ = NOW() {} RETURNING *",
            T::table_name(),
            set_clause,
            adjusted_where_clause
        );

        // Bind parameters and execute based on which mode we're using
        let updated_records = if query.has_updates() {
            // Custom updates mode: bind the update values manually
            let mut q = sqlx::query_as::<_, T>(&sql);
            for value in update_values {
                q = self.bind_param(q, value);
            }

            // Then bind WHERE clause parameters
            let mut sqlx_query = q;
            for param in params {
                sqlx_query = self.bind_param(sqlx_query, param);
            }

            // Execute the query with provided executor
            sqlx_query
                .fetch_all(executor)
                .await
                .map_err(|e| StorehausError::database_operation(T::table_name(), "query", e))?
        } else {
            // Legacy mode: use model's bind method
            let model = data.ok_or_else(|| {
                StorehausError::InvalidConfiguration {
                    message: "update_where called without data and without UpdateSet".to_string(),
                }
            })?;

            let mut sqlx_query = model.bind_update_params_owned(&sql);

            // Then bind WHERE clause parameters
            for param in params {
                sqlx_query = self.bind_param(sqlx_query, param);
            }

            // Execute the query with provided executor
            sqlx_query
                .fetch_all(executor)
                .await
                .map_err(|e| StorehausError::database_operation(T::table_name(), "query", e))?
        };

        // Note: When using transactions, signals and cache invalidation should be
        // handled AFTER the transaction commits, not here

        Ok(updated_records)
    }

    async fn delete_where(
        &self,
        query: crate::QueryBuilder,
    ) -> Result<Vec<Self::Id>, StorehausError> {
        // Build the WHERE clause from the query
        let (where_clause, _, _, params) = query.build();

        // Check if table has primary key
        let has_primary_key = !T::primary_key_field().is_empty();

        // For soft delete models, we need to UPDATE rather than DELETE
        if T::supports_soft_delete() {
            let soft_delete_field =
                T::soft_delete_field().ok_or_else(|| StorehausError::InvalidConfiguration {
                    message: format!(
                        "Model {} reports supports_soft_delete=true but soft_delete_field is None",
                        T::table_name()
                    ),
                })?;

            // Build UPDATE statement to set soft delete field = false
            let sql = if has_primary_key {
                format!(
                    "UPDATE {} SET {} = false, __updated_at__ = NOW() {} RETURNING {}",
                    T::table_name(),
                    soft_delete_field,
                    where_clause,
                    T::primary_key_field()
                )
            } else {
                // For tables without PK, just execute the update without returning IDs
                format!(
                    "UPDATE {} SET {} = false, __updated_at__ = NOW() {}",
                    T::table_name(),
                    soft_delete_field,
                    where_clause
                )
            };

            let deleted_ids = if has_primary_key {
                let mut sqlx_query = sqlx::query_as::<_, (T::Id,)>(&sql);
                for param in params {
                    sqlx_query = self.bind_param_for_id_query(sqlx_query, param);
                }

                let soft_deleted_ids: Vec<(T::Id,)> = sqlx_query
                    .fetch_all(&self.db_pool)
                    .await
                    .map_err(|e| StorehausError::database_operation(T::table_name(), "query", e))?;

                soft_deleted_ids.into_iter().map(|(id,)| id).collect()
            } else {
                // For tables without PK, execute and return empty vec
                let mut sqlx_query = sqlx::query(&sql);
                for param in params {
                    sqlx_query = self.bind_param_raw(sqlx_query, param);
                }

                sqlx_query
                    .execute(&self.db_pool)
                    .await
                    .map_err(|e| StorehausError::database_operation(T::table_name(), "query", e))?;

                Vec::new()
            };

            // Emit delete signal if signal manager is present and records were deleted
            if self.signal_manager.is_some() && !deleted_ids.is_empty() {
                let mut event = signal_system::DatabaseEvent::new(
                    signal_system::EventType::Delete,
                    T::table_name().to_string(),
                );

                // Add the primary key field name from TableMetadata
                let primary_key_name = T::primary_key_field();

                // Convert deleted IDs to PostgresValue array
                let ids_as_postgres: Vec<signal_system::PostgresValue> = deleted_ids
                    .iter()
                    .map(|id| {
                        let id_str = id_to_string(id.clone());
                        if id_str.starts_with('\"') && id_str.ends_with('\"') {
                            signal_system::PostgresValue::Text(
                                id_str.trim_matches('\"').to_string(),
                            )
                        } else if let Ok(int_id) = id_str.parse::<i32>() {
                            signal_system::PostgresValue::Integer(int_id)
                        } else if let Ok(bigint_id) = id_str.parse::<i64>() {
                            signal_system::PostgresValue::BigInt(bigint_id)
                        } else {
                            signal_system::PostgresValue::Text(id_str)
                        }
                    })
                    .collect();

                event.add_payload(
                    primary_key_name.to_string(),
                    signal_system::PostgresValue::Json(
                        serde_json::to_value(ids_as_postgres).unwrap_or_default(),
                    ),
                );

                // Add count of deleted records
                event.add_payload(
                    "deleted_count".to_string(),
                    signal_system::PostgresValue::Integer(deleted_ids.len() as i32),
                );

                self.emit_signal(event).await;
            }

            Ok(deleted_ids)
        } else {
            // Hard delete - build DELETE statement
            let sql = if has_primary_key {
                format!(
                    "DELETE FROM {} {} RETURNING {}",
                    T::table_name(),
                    where_clause,
                    T::primary_key_field()
                )
            } else {
                // For tables without PK, just execute the delete without returning IDs
                format!(
                    "DELETE FROM {} {}",
                    T::table_name(),
                    where_clause
                )
            };

            let deleted_ids = if has_primary_key {
                let mut sqlx_query = sqlx::query_as::<_, (T::Id,)>(&sql);
                for param in params {
                    sqlx_query = self.bind_param_for_id_query(sqlx_query, param);
                }

                let hard_deleted_ids: Vec<(T::Id,)> = sqlx_query
                    .fetch_all(&self.db_pool)
                    .await
                    .map_err(|e| StorehausError::database_operation(T::table_name(), "query", e))?;

                hard_deleted_ids.into_iter().map(|(id,)| id).collect()
            } else {
                // For tables without PK, execute and return empty vec
                let mut sqlx_query = sqlx::query(&sql);
                for param in params {
                    sqlx_query = self.bind_param_raw(sqlx_query, param);
                }

                sqlx_query
                    .execute(&self.db_pool)
                    .await
                    .map_err(|e| StorehausError::database_operation(T::table_name(), "query", e))?;

                Vec::new()
            };

            // Emit delete signal if signal manager is present and records were deleted
            if self.signal_manager.is_some() && !deleted_ids.is_empty() {
                let mut event = signal_system::DatabaseEvent::new(
                    signal_system::EventType::Delete,
                    T::table_name().to_string(),
                );

                let primary_key_name = T::primary_key_field();
                let ids_as_postgres: Vec<signal_system::PostgresValue> = deleted_ids
                    .iter()
                    .map(|id| {
                        let id_str = id_to_string(id.clone());
                        if id_str.starts_with('\"') && id_str.ends_with('\"') {
                            signal_system::PostgresValue::Text(
                                id_str.trim_matches('\"').to_string(),
                            )
                        } else if let Ok(int_id) = id_str.parse::<i32>() {
                            signal_system::PostgresValue::Integer(int_id)
                        } else if let Ok(bigint_id) = id_str.parse::<i64>() {
                            signal_system::PostgresValue::BigInt(bigint_id)
                        } else {
                            signal_system::PostgresValue::Text(id_str)
                        }
                    })
                    .collect();

                event.add_payload(
                    primary_key_name.to_string(),
                    signal_system::PostgresValue::Json(
                        serde_json::to_value(ids_as_postgres).unwrap_or_default(),
                    ),
                );

                event.add_payload(
                    "deleted_count".to_string(),
                    signal_system::PostgresValue::Integer(deleted_ids.len() as i32),
                );

                self.emit_signal(event).await;
            }

            Ok(deleted_ids)
        }
    }

    /// Delete records matching the given query using a custom executor (for transactions).
    ///
    /// This method allows you to perform deletions within a transaction by accepting
    /// any SQLx executor (Pool, Transaction, or Connection).
    ///
    /// For models with soft delete enabled, this performs an UPDATE to set the soft delete
    /// field to false. For models without soft delete, this performs a hard DELETE.
    ///
    /// Returns the IDs of deleted records (or empty vec if table has no primary key).
    ///
    /// # Important Notes
    ///
    /// - **Signals**: When using transactions, signals should be emitted AFTER the transaction
    ///   commits successfully, not during this method call. This ensures signals are only sent
    ///   for committed changes.
    /// - **Cache**: Similarly, cache invalidation should happen after transaction commit.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use sqlx::PgPool;
    /// use storehaus::prelude::*;
    ///
    /// async fn delete_account_with_wallets(
    ///     pool: &PgPool,
    ///     account_id: Uuid,
    /// ) -> Result<(), StorehausError> {
    ///     let mut tx = pool.begin().await?;
    ///
    ///     // First delete all wallets for the account
    ///     let wallet_query = QueryBuilder::new()
    ///         .filter(QueryFilter::eq("account_id", json!(account_id)));
    ///     wallet_store.delete_where_with_executor(&mut tx, wallet_query).await?;
    ///
    ///     // Then delete the account itself
    ///     let account_query = QueryBuilder::new()
    ///         .filter(QueryFilter::eq("account_id", json!(account_id)));
    ///     account_store.delete_where_with_executor(&mut tx, account_query).await?;
    ///
    ///     tx.commit().await?;
    ///     // Emit signals and invalidate cache here, after commit
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Arguments
    ///
    /// * `executor` - Any SQLx executor (Pool, Transaction, or Connection)
    /// * `query` - QueryBuilder with filters to match records for deletion
    ///
    /// # Returns
    ///
    /// Vec of IDs of deleted records (empty if table has no primary key)
    async fn delete_where_with_executor<'e, E>(
        &self,
        executor: E,
        query: crate::QueryBuilder,
    ) -> Result<Vec<Self::Id>, StorehausError>
    where
        E: sqlx::Executor<'e, Database = sqlx::Postgres>,
    {
        // Build the WHERE clause from the query
        let (where_clause, _, _, params) = query.build();

        // Check if table has primary key
        let has_primary_key = !T::primary_key_field().is_empty();

        // For soft delete models, we need to UPDATE rather than DELETE
        if T::supports_soft_delete() {
            let soft_delete_field =
                T::soft_delete_field().ok_or_else(|| StorehausError::InvalidConfiguration {
                    message: format!(
                        "Model {} reports supports_soft_delete=true but soft_delete_field is None",
                        T::table_name()
                    ),
                })?;

            // Build UPDATE statement to set soft delete field = false
            let sql = if has_primary_key {
                format!(
                    "UPDATE {} SET {} = false, __updated_at__ = NOW() {} RETURNING {}",
                    T::table_name(),
                    soft_delete_field,
                    where_clause,
                    T::primary_key_field()
                )
            } else {
                // For tables without PK, just execute the update without returning IDs
                format!(
                    "UPDATE {} SET {} = false, __updated_at__ = NOW() {}",
                    T::table_name(),
                    soft_delete_field,
                    where_clause
                )
            };

            let deleted_ids = if has_primary_key {
                let mut sqlx_query = sqlx::query_as::<_, (Self::Id,)>(&sql);
                for param in params {
                    sqlx_query = self.bind_param_for_id_query(sqlx_query, param);
                }

                let soft_deleted_ids: Vec<(Self::Id,)> = sqlx_query
                    .fetch_all(executor)
                    .await
                    .map_err(|e| StorehausError::database_operation(T::table_name(), "delete_where_with_executor", e))?;

                soft_deleted_ids.into_iter().map(|(id,)| id).collect()
            } else {
                // For tables without PK, execute and return empty vec
                let mut sqlx_query = sqlx::query(&sql);
                for param in params {
                    sqlx_query = self.bind_param_raw(sqlx_query, param);
                }

                sqlx_query
                    .execute(executor)
                    .await
                    .map_err(|e| StorehausError::database_operation(T::table_name(), "delete_where_with_executor", e))?;

                Vec::new()
            };

            // Note: When using transactions, signals should be emitted AFTER the transaction commits, not here

            Ok(deleted_ids)
        } else {
            // Hard delete for models without soft delete support
            let sql = if has_primary_key {
                format!(
                    "DELETE FROM {} {} RETURNING {}",
                    T::table_name(),
                    where_clause,
                    T::primary_key_field()
                )
            } else {
                format!("DELETE FROM {} {}", T::table_name(), where_clause)
            };

            let deleted_ids = if has_primary_key {
                let mut sqlx_query = sqlx::query_as::<_, (Self::Id,)>(&sql);
                for param in params {
                    sqlx_query = self.bind_param_for_id_query(sqlx_query, param);
                }

                let hard_deleted_ids: Vec<(Self::Id,)> = sqlx_query
                    .fetch_all(executor)
                    .await
                    .map_err(|e| StorehausError::database_operation(T::table_name(), "delete_where_with_executor", e))?;

                hard_deleted_ids.into_iter().map(|(id,)| id).collect()
            } else {
                let mut sqlx_query = sqlx::query(&sql);
                for param in params {
                    sqlx_query = self.bind_param_raw(sqlx_query, param);
                }

                sqlx_query
                    .execute(executor)
                    .await
                    .map_err(|e| StorehausError::database_operation(T::table_name(), "delete_where_with_executor", e))?;

                Vec::new()
            };

            // Note: When using transactions, signals should be emitted AFTER the transaction commits, not here

            Ok(deleted_ids)
        }
    }

    async fn count_where(&self, query: crate::QueryBuilder) -> Result<i64, StorehausError> {
        let (where_clause, _, _, params) = query.build(); // No ORDER BY or LIMIT for COUNT
                                                          // Avoid format! allocation by building string directly
        let base_sql = T::count_base_sql();
        let mut full_sql = String::with_capacity(base_sql.len() + where_clause.len());
        full_sql.push_str(base_sql);
        if !where_clause.is_empty() {
            // If base_sql already has WHERE (soft delete), replace WHERE with AND
            if base_sql.contains(" WHERE ") && where_clause.starts_with("WHERE ") {
                full_sql.push_str(" AND ");
                full_sql.push_str(&where_clause[6..]); // Skip "WHERE "
            } else {
                full_sql.push_str(&where_clause);
            }
        }

        let mut sqlx_query = sqlx::query(&full_sql);
        for param in params {
            sqlx_query = self.bind_param_raw(sqlx_query, param);
        }

        let result = sqlx_query
            .fetch_one(&self.db_pool)
            .await
            .map_err(|e| StorehausError::database_operation(T::table_name(), "query", e))?;

        let total: i64 = result.get("total");
        Ok(total)
    }
}

// Macro for the shared parameter binding logic
macro_rules! bind_json_param {
    ($query:expr, $param:expr) => {
        match $param {
            serde_json::Value::String(s) => {
                // Try to parse as RFC3339 timestamp first
                if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(&s) {
                    $query.bind(dt.with_timezone(&chrono::Utc))
                // Try to parse as UUID
                } else if let Ok(uuid) = uuid::Uuid::parse_str(&s) {
                    $query.bind(uuid)
                } else {
                    $query.bind(s)
                }
            },
            serde_json::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    if i >= i32::MIN as i64 && i <= i32::MAX as i64 {
                        $query.bind(i as i32)
                    } else {
                        $query.bind(i)
                    }
                } else if let Some(f) = n.as_f64() {
                    $query.bind(f)
                } else {
                    $query.bind(n.to_string())
                }
            }
            serde_json::Value::Bool(b) => $query.bind(b),
            serde_json::Value::Null => $query.bind(Option::<String>::None),
            other => $query.bind(other.to_string()),
        }
    };
}

// Helper implementation for parameter binding
impl<T: TableMetadata> GenericStore<T>
where
    T: TableMetadata
        + DatabaseExecutor
        + for<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow>
        + serde::Serialize
        + Unpin,
{
    fn bind_param<'q>(
        &self,
        query: sqlx::query::QueryAs<'q, sqlx::Postgres, T, sqlx::postgres::PgArguments>,
        param: serde_json::Value,
    ) -> sqlx::query::QueryAs<'q, sqlx::Postgres, T, sqlx::postgres::PgArguments> {
        bind_json_param!(query, param)
    }

    fn bind_param_for_id_query<'q>(
        &self,
        query: sqlx::query::QueryAs<'q, sqlx::Postgres, (T::Id,), sqlx::postgres::PgArguments>,
        param: serde_json::Value,
    ) -> sqlx::query::QueryAs<'q, sqlx::Postgres, (T::Id,), sqlx::postgres::PgArguments> {
        bind_json_param!(query, param)
    }

    fn bind_param_raw<'q>(
        &self,
        query: sqlx::query::Query<'q, sqlx::Postgres, sqlx::postgres::PgArguments>,
        param: serde_json::Value,
    ) -> sqlx::query::Query<'q, sqlx::Postgres, sqlx::postgres::PgArguments> {
        bind_json_param!(query, param)
    }
}
