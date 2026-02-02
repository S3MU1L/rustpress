use thiserror::Error;

use crate::models::UserIden;

#[derive(Error, Debug)]
pub enum GeneralError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Migration error: {0}")]
    Migration(#[from] sqlx::migrate::MigrateError),
}

#[derive(Error, Debug)]
pub enum UserError {
    #[error("User with {0:?} not found")]
    NotFound(UserIden),

    #[error("User with {0:?} already exists")]
    AlreadyExists(UserIden),

    #[error("User with {0:?} has already been deleted")]
    AlreadyDeleted(UserIden),

    #[error("User with {0:?} has been irreversibly deleted")]
    IrreversiblyDeleted(UserIden),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("An unexpected error occurred")]
    Internal,
}
