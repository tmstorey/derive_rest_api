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
/// #[derive(Clone)]
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
///     ) -> Result<Vec<u8>, Self::Error> {
///         // Your implementation here
///         Ok(vec![])
///     }
/// }
/// ```
pub trait HttpClient: Clone {
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

/// Trait for async HTTP clients that can execute REST API requests.
///
/// This trait abstracts over different async HTTP client implementations (reqwest async, hyper, etc.)
/// allowing async code to work with any compliant async client.
///
/// # Example
///
/// ```ignore
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
/// #[derive(Clone)]
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
///     ) -> Result<Vec<u8>, Self::Error> {
///         // Your async implementation here
///         Ok(vec![])
///     }
/// }
/// ```
pub trait AsyncHttpClient: Clone  {
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
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP request fails
    fn send_async(
        &self,
        method: &str,
        url: &str,
        headers: HashMap<String, String>,
        body: Option<Vec<u8>>,
    ) -> impl std::future::Future<Output = Result<Vec<u8>, Self::Error>> + Send;
}
