//! # derive_rest_api
//!
//! A procedural macro library for generating type-safe builder patterns for REST API request structures.
//!
//! This library provides two derive macros:
//! - `RequestBuilder` - Generates builder patterns for individual API requests
//! - `ApiClient` - Generates high-level client structs that wrap multiple requests
//!
//! ## Features
//!
//! - URL path parameter templating
//! - Query string serialization
//! - Field validation
//! - Flexible type conversion with `Into<T>`
//! - Default value handling
//! - Type-safe error handling with `thiserror`
//! - Support for multiple HTTP client backends (reqwest, ureq, or custom)
//!
//! ## Basic RequestBuilder Example
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
//! ## ApiClient Example
//!
//! For a more ergonomic API, use `#[derive(ApiClient)]` to generate a high-level client:
//!
//! ```rust,ignore
//! use derive_rest_api::{ApiClient, RequestBuilder, ReqwestBlockingClient};
//! use serde::Serialize;
//!
//! #[derive(RequestBuilder, Serialize)]
//! #[request_builder(method = "GET", path = "/users/{id}")]
//! struct GetUser {
//!     id: u64,
//! }
//!
//! #[derive(RequestBuilder, Serialize)]
//! #[request_builder(method = "POST", path = "/users")]
//! struct CreateUser {
//!     #[request_builder(body)]
//!     name: String,
//! }
//!
//! #[derive(Clone, ApiClient)]
//! #[api_client(
//!     base_url = "https://api.example.com",
//!     requests(GetUser, CreateUser = "new_user")
//! )]
//! struct MyApiConfig {
//!     api_key: String,
//! }
//!
//! // Usage:
//! let config = MyApiConfig { api_key: "key".to_string() };
//! let client = MyApiClient::new(config, ReqwestBlockingClient::new()?);
//!
//! // Methods are pre-configured with base URL and HTTP client
//! let user = client.get_user().id(123).send()?;
//! let new_user = client.new_user().name("Alice".to_string()).send()?;
//! ```
//!
//! ## Error Handling
//!
//! All operations return `Result<T, RequestError>` with specific error variants:
//!
//! ```rust,ignore
//! use derive_rest_api::{RequestBuilder, RequestError};
//!
//! match request.build() {
//!     Ok(req) => { /* ... */ },
//!     Err(RequestError::MissingField { field }) => {
//!         eprintln!("Missing field: {}", field);
//!     }
//!     Err(RequestError::ValidationError { field, message }) => {
//!         eprintln!("Validation failed for {}: {}", field, message);
//!     }
//!     Err(e) => eprintln!("Error: {}", e),
//! }
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

// Module declarations
mod traits;
mod clients;
mod error;

// Re-exports
pub use derive_rest_api_macros::{ApiClient, RequestBuilder};
pub use traits::{AsyncHttpClient, HttpClient};
pub use error::RequestError;

#[cfg(feature = "reqwest-blocking")]
pub use clients::ReqwestBlockingClient;

#[cfg(feature = "reqwest-async")]
pub use clients::ReqwestAsyncClient;

#[cfg(feature = "ureq-blocking")]
pub use clients::UreqBlockingClient;
