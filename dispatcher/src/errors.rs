use thiserror::Error;

#[derive(Error, Debug)]
pub enum DispatcherError {
    #[error("Database connection error: {0}")]
    DatabaseConnection(#[from] sqlx::Error),

    #[error("Store object not found: {0}")]
    StoreNotFound(String),

    #[error("Store object already registered: {0}")]
    StoreAlreadyRegistered(String),
}