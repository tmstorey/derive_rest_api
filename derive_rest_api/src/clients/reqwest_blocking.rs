//! Blocking reqwest HTTP client implementation.

use crate::HttpClient;
use std::collections::HashMap;

/// Blocking reqwest client wrapper that implements HttpClient
///
/// This wrapper provides a blocking HTTP client implementation using reqwest's
/// blocking API. It can be created with default settings or with a custom
/// reqwest blocking client for advanced configuration.
///
/// # Examples
///
/// Basic usage:
/// ```no_run
/// use derive_rest_api::ReqwestBlockingClient;
///
/// let client = ReqwestBlockingClient::new().unwrap();
/// ```
///
/// With custom configuration:
/// ```no_run
/// use derive_rest_api::ReqwestBlockingClient;
///
/// let reqwest_client = reqwest::blocking::Client::builder()
///     .timeout(std::time::Duration::from_secs(30))
///     .build()
///     .unwrap();
///
/// let client = ReqwestBlockingClient::with_client(reqwest_client);
/// ```
#[derive(Clone)]
pub struct ReqwestBlockingClient {
    client: reqwest::blocking::Client,
}

impl ReqwestBlockingClient {
    /// Creates a new blocking reqwest client wrapper
    ///
    /// # Errors
    ///
    /// Returns an error if the reqwest client cannot be created
    pub fn new() -> Result<Self, reqwest::Error> {
        Ok(Self {
            client: reqwest::blocking::Client::new(),
        })
    }

    /// Creates a new blocking reqwest client wrapper with a custom client
    ///
    /// This allows you to configure the reqwest client with custom settings
    /// such as timeouts, user agents, etc.
    pub fn with_client(client: reqwest::blocking::Client) -> Self {
        Self { client }
    }
}

impl Default for ReqwestBlockingClient {
    fn default() -> Self {
        Self {
            client: reqwest::blocking::Client::new(),
        }
    }
}

impl HttpClient for ReqwestBlockingClient {
    type Error = reqwest::Error;

    fn send(
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

        let response = request.send()?;
        let bytes = response.bytes()?;
        Ok(bytes.to_vec())
    }
}
