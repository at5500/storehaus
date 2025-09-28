//! Generic store implementations
//!
//! This module provides generic database store functionality.

pub mod core;
pub mod filterable;
pub mod soft_deletable;
pub mod store_object;

pub use core::GenericStore;
