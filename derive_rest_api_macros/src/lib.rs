//! Procedural macros for the derive_rest_api crate.
//!
//! This crate provides derive macros for generating REST API request builders.

mod api_client;
mod request_builder;

/// Derive macro for generating a builder pattern for REST API requests.
///
/// This macro generates:
/// - A builder struct with optional fields
/// - Setter methods for each field
/// - A `build()` method that validates and constructs the original struct
/// - HTTP methods (`build_url`, `build_body`, `build_headers`, `send_with_client`)
/// - Convenience methods (`send`, `send_async`) when clients are embedded
///
/// # Example
///
/// ```ignore
/// use derive_rest_api::RequestBuilder;
/// use serde::Serialize;
///
/// #[derive(RequestBuilder, Serialize)]
/// #[request_builder(method = "GET", path = "/users/{id}")]
/// struct GetUser {
///     id: u64,
///     #[request_builder(query)]
///     include_posts: Option<bool>,
/// }
/// ```
#[proc_macro_derive(RequestBuilder, attributes(request_builder))]
pub fn derive_request_builder(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);

    match request_builder::generate_request_builder(input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

/// Derive macro for generating API client structs.
///
/// This macro generates two client structs (blocking and async) that wrap
/// a configuration struct and provide methods for making API requests.
///
/// # Example
///
/// ```ignore
/// use derive_rest_api::ApiClient;
///
/// #[derive(Clone, ApiClient)]
/// #[api_client(
///     base_url = "https://api.example.com",
///     requests(GetUser, CreateUser = "new_user")
/// )]
/// struct MyApiConfig {
///     api_key: String,
/// }
/// ```
///
/// This generates:
/// - `MyApiClient<C: HttpClient>` - Blocking client
/// - `MyApiAsyncClient<A: AsyncHttpClient>` - Async client
#[proc_macro_derive(ApiClient, attributes(api_client))]
pub fn derive_api_client(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);

    match api_client::generate_api_client(input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}
