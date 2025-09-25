//! Traits for database operations
//!
//! This module contains all the traits that define the interface for database operations
//! in the storehaus library.

pub mod core;
pub mod soft_deletable;
pub mod filterable;
pub mod table_metadata;

// Re-export all public items for convenience
pub use core::StoreObject;
pub use soft_deletable::SoftDeletable;
pub use filterable::{StoreFilter, Filterable};
pub use table_metadata::TableMetadata;