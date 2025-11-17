use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Entity not found: {entity}")]
    NotFound { entity: String },

    #[error("Uniqueness conflict: {field}")]
    Uniqueness { field: String },

    #[error("Forbidden action")]
    Forbidden,

    #[error("Invalid state transition from {from} to {to}")]
    InvalidStateTransition { from: String, to: String },

    #[error("Validation error: {message}")]
    Validation { message: String },
}
