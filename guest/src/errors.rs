use thiserror::Error;

/// Use your own error for this package instead of relying on external error code
#[derive(Error, Debug)]
pub enum ChickError {
    #[error("{0}")]
    SerializationError(String),
    #[error("{0}")]
    InvalidArguments(String),
}
