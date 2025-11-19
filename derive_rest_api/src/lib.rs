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
//! - `#[request_builder(query)]` - Include field in query string
//! - `#[request_builder(body)]` - Mark field as request body
//! - `#[request_builder(header)]` - Mark field as HTTP header
//! - `#[request_builder(into)]` - Enable `Into<T>` conversion for this field
//! - `#[request_builder(default)]` - Use default value if not set
//! - `#[request_builder(validate = "fn_path")]` - Specify custom validation function

// Re-export the procedural macro
pub use derive_rest_api_macros::RequestBuilder;
