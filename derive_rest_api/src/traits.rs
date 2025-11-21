//! HTTP client traits for blocking and async request execution.

use std::collections::HashMap;

/// Trait for blocking HTTP clients that can execute REST API requests.
///
/// This trait abstracts over different blocking HTTP client implementations (reqwest blocking, ureq, etc.)
/// allowing the generated request builders to work with any compliant blocking client.
///
/// # Example
///
/// ```
/// use derive_rest_api::HttpClient;
/// use std::collections::HashMap;
///
/// #[derive(Debug)]
/// struct MyError;
///
/// impl std::fmt::Display for MyError {
///     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
///         write!(f, "MyError")
///     }
/// }
///
/// impl std::error::Error for MyError {}
///
/// #[derive(Clone, Default)]
/// struct MyClient;
///
/// impl HttpClient for MyClient {
///     type Error = MyError;
///
///     fn send(
///         &self,
///         method: &str,
///         url: &str,
///         headers: HashMap<String, String>,
///         body: Option<Vec<u8>>,
///         timeout: Option<std::time::Duration>,
///     ) -> Result<Vec<u8>, Self::Error> {
///         // Your implementation here
///         Ok(vec![])
///     }
/// }
/// ```
pub trait HttpClient: Clone + Default {
    /// The error type for this HTTP client
    type Error: std::error::Error + Send + Sync + 'static;

    /// Send a blocking HTTP request with the given parameters
    ///
    /// # Arguments
    ///
    /// - `method`: HTTP method (GET, POST, PUT, DELETE, etc.)
    /// - `url`: Complete URL including query parameters
    /// - `headers`: HTTP headers as key-value pairs
    /// - `body`: Optional request body as bytes
    /// - `timeout`: Optional timeout duration for the request
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
        timeout: Option<std::time::Duration>,
    ) -> Result<Vec<u8>, Self::Error>;
}

/// Trait for async HTTP clients that can execute REST API requests.
///
/// This trait abstracts over different async HTTP client implementations (reqwest async, hyper, etc.)
/// allowing async code to work with any compliant async client.
///
/// # Example
///
/// ```
/// use derive_rest_api::AsyncHttpClient;
/// use std::collections::HashMap;
///
/// #[derive(Debug)]
/// struct MyError;
///
/// impl std::fmt::Display for MyError {
///     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
///         write!(f, "MyError")
///     }
/// }
///
/// impl std::error::Error for MyError {}
///
/// #[derive(Clone, Default)]
/// struct MyAsyncClient;
///
/// impl AsyncHttpClient for MyAsyncClient {
///     type Error = MyError;
///
///     async fn send_async(
///         &self,
///         method: &str,
///         url: &str,
///         headers: HashMap<String, String>,
///         body: Option<Vec<u8>>,
///         timeout: Option<std::time::Duration>,
///     ) -> Result<Vec<u8>, Self::Error> {
///         // Your async implementation here
///         Ok(vec![])
///     }
/// }
/// ```
pub trait AsyncHttpClient: Clone + Default  {
    /// The error type for this HTTP client
    type Error: std::error::Error + Send + Sync + 'static;

    /// Send an async HTTP request with the given parameters
    ///
    /// # Arguments
    ///
    /// - `method`: HTTP method (GET, POST, PUT, DELETE, etc.)
    /// - `url`: Complete URL including query parameters
    /// - `headers`: HTTP headers as key-value pairs
    /// - `body`: Optional request body as bytes
    /// - `timeout`: Optional timeout duration for the request
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP request fails
    #[cfg(not(target_arch = "wasm32"))]
    fn send_async(
        &self,
        method: &str,
        url: &str,
        headers: HashMap<String, String>,
        body: Option<Vec<u8>>,
        timeout: Option<std::time::Duration>,
    ) -> impl std::future::Future<Output = Result<Vec<u8>, Self::Error>> + Send;

    /// Send an async HTTP request with the given parameters (WASM version)
    ///
    /// # Arguments
    ///
    /// - `method`: HTTP method (GET, POST, PUT, DELETE, etc.)
    /// - `url`: Complete URL including query parameters
    /// - `headers`: HTTP headers as key-value pairs
    /// - `body`: Optional request body as bytes
    /// - `timeout`: Optional timeout duration for the request
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP request fails
    #[cfg(target_arch = "wasm32")]
    fn send_async(
        &self,
        method: &str,
        url: &str,
        headers: HashMap<String, String>,
        body: Option<Vec<u8>>,
        timeout: Option<std::time::Duration>,
    ) -> impl std::future::Future<Output = Result<Vec<u8>, Self::Error>>;
}

impl HttpClient for crate::clients::UnimplementedClient {
    type Error = std::convert::Infallible;

    fn send(
        &self,
        _method: &str,
        _url: &str,
        _headers: HashMap<String, String>,
        _body: Option<Vec<u8>>,
        _timeout: Option<std::time::Duration>,
    ) -> Result<Vec<u8>, Self::Error> {
        unimplemented!("No blocking client found.")
    }
}

impl AsyncHttpClient for crate::clients::UnimplementedClient {
    type Error = std::convert::Infallible;

    async fn send_async(
        &self,
        _method: &str,
        _url: &str,
        _headers: HashMap<String, String>,
        _body: Option<Vec<u8>>,
        _timeout: Option<std::time::Duration>,
    ) -> Result<Vec<u8>, Self::Error> {
        unimplemented!("No async client found.")
    }
}

/// Trait for modifying request builders with common operations.
///
/// This trait is automatically implemented by all generated request builders,
/// allowing configuration structs to uniformly modify requests.
///
/// # Example
///
/// ```rust,ignore
/// use derive_rest_api::RequestModifier;
///
/// fn add_auth<M: RequestModifier>(modifier: M, token: &str) -> M {
///     modifier.header("Authorization", format!("Bearer {}", token))
/// }
/// ```
pub trait RequestModifier: Sized {
    /// Adds an HTTP header to the request.
    ///
    /// # Arguments
    ///
    /// * `name` - The header name
    /// * `value` - The header value
    fn header(self, name: impl Into<String>, value: impl Into<String>) -> Self;

    /// Sets the timeout duration for the request.
    ///
    /// # Arguments
    ///
    /// * `timeout` - The timeout duration
    fn timeout(self, timeout: std::time::Duration) -> Self;
}

/// Trait for configuration structs to modify request builders.
///
/// Implement this trait on your API configuration struct to automatically
/// apply settings (like authentication headers) to all requests.
///
/// # Example
///
/// ```rust,ignore
/// use derive_rest_api::{ConfigureRequest, RequestModifier};
///
/// struct MyApiConfig {
///     api_key: String,
/// }
///
/// impl ConfigureRequest for MyApiConfig {
///     fn configure<M: RequestModifier>(&self, modifier: M) -> M {
///         modifier
///             .header("X-API-Key", &self.api_key)
///             .header("User-Agent", "my-app/1.0")
///     }
/// }
/// ```
pub trait ConfigureRequest {
    /// Configures a request builder with settings from this configuration.
    ///
    /// # Arguments
    ///
    /// * `modifier` - The request builder to modify
    ///
    /// # Returns
    ///
    /// The modified request builder
    fn configure<M: RequestModifier>(&self, modifier: M) -> M;
}

/// Marker trait to indicate a type does not need request configuration.
///
/// Implement this trait (with an empty impl block) if your config struct
/// doesn't need to modify requests.
pub trait NoRequestConfiguration {}

/// Blanket implementation of `ConfigureRequest` for types that don't need configuration.
impl<T: NoRequestConfiguration> ConfigureRequest for T {
    fn configure<M: RequestModifier>(&self, modifier: M) -> M {
        modifier
    }
}
