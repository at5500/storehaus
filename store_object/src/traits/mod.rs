//! Traits for database operations
//!
//! This module contains all the traits that define the interface for database operations
//! in the storehaus library.

pub mod core;
pub mod filterable;
pub mod soft_deletable;
pub mod table_metadata;

// Re-export all public items for convenience
pub use core::StoreObject;
pub use filterable::{Filterable, StoreFilter};
pub use soft_deletable::SoftDeletable;
pub use table_metadata::TableMetadata;
