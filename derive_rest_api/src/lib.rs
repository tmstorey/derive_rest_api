//! # derive_rest_api
//!
//! A procedural macro library for generating type-safe builder patterns for REST API request structures.
//!
//! This library provides the `RequestBuilder` derive macro that automatically creates builder structs
//! with support for:
//! - URL path parameter templating
//! - Query string serialization
//! - Field validation
//! - Flexible type conversion with `Into<T>`
//! - Default value handling
//!
//! ## Example
//!
//! ```rust
//! use derive_rest_api::RequestBuilder;
//! use serde::Serialize;
//!
//! #[derive(RequestBuilder, Serialize)]
//! #[request_builder(method = "GET", path = "/api/users/{id}")]
//! struct GetUser {
//!     id: u64,
//!     #[request_builder(query)]
//!     #[serde(skip_serializing)]
//!     include_posts: Option<bool>,
//! }
//!
//! // Usage:
//! let request = GetUserBuilder::new()
//!     .id(123)
//!     .include_posts(true)
//!     .build()
//!     .unwrap();
//!
//! assert_eq!(request.id, 123);
//! assert_eq!(request.include_posts, Some(true));
//! ```
//!
//! ## Struct-level Attributes
//!
//! - `#[request_builder(into)]` - Enable `Into<T>` conversion for all setter methods
//! - `#[request_builder(default)]` - Use `Default::default()` for unset fields
//! - `#[request_builder(method = "...")]` - Specify HTTP method (GET, POST, etc.)
//! - `#[request_builder(path = "...")]` - URL path template with `{param}` placeholders
//! - `#[request_builder(response = Type)]` - Specify the response type
//! - `#[request_builder(query_config = "...")]` - Custom query string serialization config
//!
//! ## Field-level Attributes
//!
//! - `#[request_builder(path)]` - Mark field as URL path parameter
//! - `#[request_builder(query)]` or `#[request_builder(query = "name")]` - Include field in query string (with optional custom name)
//! - `#[request_builder(body)]` or `#[request_builder(body = "name")]` - Mark field as request body (with optional custom name)
//! - `#[request_builder(header)]` or `#[request_builder(header = "Header-Name")]` - Mark field as HTTP header (auto-converts snake_case to Title-Case, or use custom name)
//! - `#[request_builder(into)]` - Enable `Into<T>` conversion for this field
//! - `#[request_builder(default)]` - Use default value if not set
//! - `#[request_builder(validate = "fn_path")]` - Specify custom validation function
//!
//! ## Serde Integration
//!
//! All serde attributes (like `#[serde(rename = "...")]`, `#[serde(flatten)]`, etc.) on fields
//! marked with `body` or `query` are automatically copied to the generated serialization structs,
//! allowing full control over how fields are serialized.

use std::collections::HashMap;

/// Trait for HTTP clients that can execute REST API requests.
///
/// This trait abstracts over different HTTP client implementations (reqwest, ureq, etc.)
/// allowing the generated request builders to work with any compliant client.
///
/// # Type Parameters
///
/// - `E`: The error type returned by the client
pub trait HttpClient {
    /// The error type for this HTTP client
    type Error: std::fmt::Debug;

    /// Send an HTTP request with the given parameters
    ///
    /// # Arguments
    ///
    /// - `method`: HTTP method (GET, POST, PUT, DELETE, etc.)
    /// - `url`: Complete URL including query parameters
    /// - `headers`: HTTP headers as key-value pairs
    /// - `body`: Optional request body as bytes
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP request fails
    fn send(
        &self,
        method: &str,
        url: &str,
        headers: HashMap<String, String>,
        body: Option<Vec<u8>>,
    ) -> Result<Vec<u8>, Self::Error>;
}

// Re-export the procedural macro
pub use derive_rest_api_macros::RequestBuilder;
