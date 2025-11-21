//! Error types for request building and execution.

use std::error::Error as StdError;

/// Errors that can occur during request building and execution.
#[derive(Debug, thiserror::Error)]
pub enum RequestError {
    /// A required field was not set in the builder.
    #[error("Missing required field: {field}")]
    MissingField { field: String },

    /// A required path parameter was not provided.
    #[error("Missing required path parameter: {param}")]
    MissingPathParameter { param: String },

    /// Query string serialization failed.
    #[error("Failed to serialize query parameters: {source}")]
    QuerySerializationError {
        #[source]
        source: serde_qs::Error,
    },

    /// Request body serialization failed.
    #[error("Failed to serialize request body: {source}")]
    BodySerializationError {
        #[source]
        source: serde_json::Error,
    },

    /// Request body serialization failed.
    #[error("Failed to deserialize response body: {source}")]
    ResponseDeserializationError {
        #[source]
        source: serde_json::Error,
    },

    /// Field validation failed.
    #[error("Validation failed for field '{field}': {message}")]
    ValidationError { field: String, message: String },

    /// No base URL was configured.
    #[error("No base URL configured. Use .base_url() to set one.")]
    MissingBaseUrl,

    /// URL building failed.
    #[error("Failed to build URL: {source}")]
    UrlBuildError {
        #[source]
        source: Box<RequestError>,
    },

    /// HTTP request failed with a client-specific error.
    ///
    /// This wraps errors from the underlying HTTP client implementation.
    #[error("HTTP request failed: {0}")]
    HttpError(Box<dyn StdError + Send + Sync>),
}

impl RequestError {
    /// Creates a new `MissingField` error.
    pub fn missing_field(field: impl Into<String>) -> Self {
        Self::MissingField {
            field: field.into(),
        }
    }

    /// Creates a new `MissingPathParameter` error.
    pub fn missing_path_parameter(param: impl Into<String>) -> Self {
        Self::MissingPathParameter {
            param: param.into(),
        }
    }

    /// Creates a new `ValidationError`.
    pub fn validation_error(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self::ValidationError {
            field: field.into(),
            message: message.into(),
        }
    }

    /// Creates a new `HttpError` from any error type.
    pub fn http_error(error: impl StdError + Send + Sync + 'static) -> Self {
        Self::HttpError(Box::new(error))
    }
}
