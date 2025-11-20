//! Async reqwest HTTP client implementation.

use crate::AsyncHttpClient;
use std::collections::HashMap;

/// Async reqwest client wrapper that implements AsyncHttpClient
///
/// This wrapper provides an async HTTP client implementation using reqwest's
/// async API. It can be created with default settings or with a custom
/// reqwest client for advanced configuration.
///
/// # Examples
///
/// Basic usage:
/// ```no_run
/// use derive_rest_api::ReqwestAsyncClient;
///
/// # async {
/// let client = ReqwestAsyncClient::new().unwrap();
/// # };
/// ```
///
/// With custom configuration:
/// ```no_run
/// use derive_rest_api::ReqwestAsyncClient;
///
/// # async {
/// let reqwest_client = reqwest::Client::builder()
///     .timeout(std::time::Duration::from_secs(30))
///     .build()
///     .unwrap();
///
/// let client = ReqwestAsyncClient::with_client(reqwest_client);
/// # };
/// ```
#[derive(Clone)]
pub struct ReqwestAsyncClient {
    client: reqwest::Client,
}

impl ReqwestAsyncClient {
    /// Creates a new async reqwest client wrapper
    ///
    /// # Errors
    ///
    /// Returns an error if the reqwest client cannot be created
    pub fn new() -> Result<Self, reqwest::Error> {
        Ok(Self {
            client: reqwest::Client::new(),
        })
    }

    /// Creates a new async reqwest client wrapper with a custom client
    ///
    /// This allows you to configure the reqwest client with custom settings
    /// such as timeouts, user agents, etc.
    pub fn with_client(client: reqwest::Client) -> Self {
        Self { client }
    }
}

impl From<reqwest::Client> for ReqwestAsyncClient {
    fn from(client: reqwest::Client) -> Self {
        ReqwestAsyncClient::with_client(client)
    }
}

impl Default for ReqwestAsyncClient {
    fn default() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }
}

impl AsyncHttpClient for ReqwestAsyncClient {
    type Error = reqwest::Error;

    async fn send_async(
        &self,
        method: &str,
        url: &str,
        headers: HashMap<String, String>,
        body: Option<Vec<u8>>,
    ) -> Result<Vec<u8>, Self::Error> {
        let mut request = match method.to_uppercase().as_str() {
            "GET" => self.client.get(url),
            "POST" => self.client.post(url),
            "PUT" => self.client.put(url),
            "DELETE" => self.client.delete(url),
            "PATCH" => self.client.patch(url),
            "HEAD" => self.client.head(url),
            _ => {
                // For other methods, use the generic request method
                self.client.request(
                    reqwest::Method::from_bytes(method.as_bytes())
                        .unwrap_or(reqwest::Method::GET),
                    url
                )
            }
        };

        // Add headers
        for (key, value) in headers {
            request = request.header(key, value);
        }

        // Add body if present
        if let Some(body_data) = body {
            request = request.body(body_data);
        }

        let response = request.send().await?;
        let bytes = response.bytes().await?;
        Ok(bytes.to_vec())
    }
}
