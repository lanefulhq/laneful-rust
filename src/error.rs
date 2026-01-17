//! Error types for the Laneful SDK.

use thiserror::Error;

/// Errors that can occur when using the Laneful SDK.
#[derive(Debug, Error)]
pub enum LanefulError {
    /// HTTP request failed.
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),

    /// API returned an error response.
    #[error("API error: {0}")]
    ApiError(String),

    /// Invalid configuration.
    #[error("Invalid configuration: {0}")]
    ConfigError(String),

    /// Email validation failed.
    #[error("Validation error: {0}")]
    ValidationError(String),
}

/// Result type alias for Laneful operations.
pub type Result<T> = std::result::Result<T, LanefulError>;
