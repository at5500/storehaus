pub mod errors;
pub mod traits;
pub mod generic_store;
pub mod query_builder;

pub use errors::StorehausError;
pub use traits::*;
pub use generic_store::GenericStore;
pub use query_builder::{QueryBuilder, QueryFilter, QueryOperator, SortOrder};

use sqlx::PgPool;

pub type DbPool = PgPool;
