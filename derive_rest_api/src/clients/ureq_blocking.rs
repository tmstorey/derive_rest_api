//! Ureq blocking HTTP client implementation.

use crate::HttpClient;
use std::collections::HashMap;

/// Ureq client wrapper that implements HttpClient
///
/// This wrapper provides a blocking HTTP client implementation using ureq,
/// a lightweight, synchronous HTTP client library. It can be created with
/// default settings or with a custom ureq agent for advanced configuration.
///
/// # Examples
///
/// Basic usage:
/// ```no_run
/// use derive_rest_api::UreqBlockingClient;
///
/// let client = UreqBlockingClient::new();
/// ```
///
/// With custom configuration:
/// ```no_run
/// use derive_rest_api::UreqBlockingClient;
///
/// let agent = ureq::AgentBuilder::new()
///     .timeout(std::time::Duration::from_secs(30))
///     .build();
///
/// let client = UreqBlockingClient::with_agent(agent);
/// ```
#[derive(Clone)]
pub struct UreqBlockingClient {
    agent: ureq::Agent,
}

impl UreqBlockingClient {
    /// Creates a new ureq client wrapper with default settings
    pub fn new() -> Self {
        Self {
            agent: ureq::Agent::new(),
        }
    }

    /// Creates a new ureq client wrapper with a custom agent
    ///
    /// This allows you to configure the ureq agent with custom settings
    /// such as timeouts, proxy settings, etc.
    pub fn with_agent(agent: ureq::Agent) -> Self {
        Self { agent }
    }
}

impl From<ureq::Agent> for UreqBlockingClient {
    fn from(agent: ureq::Agent) -> Self {
        UreqBlockingClient::with_agent(agent)
    }
}

impl Default for UreqBlockingClient {
    fn default() -> Self {
        Self::new()
    }
}

impl HttpClient for UreqBlockingClient {
    type Error = ureq::Error;

    fn send(
        &self,
        method: &str,
        url: &str,
        headers: HashMap<String, String>,
        body: Option<Vec<u8>>,
        timeout: Option<std::time::Duration>,
    ) -> Result<Vec<u8>, Self::Error> {
        // Create the request based on the HTTP method
        let mut request = match method.to_uppercase().as_str() {
            "GET" => self.agent.get(url),
            "POST" => self.agent.post(url),
            "PUT" => self.agent.put(url),
            "DELETE" => self.agent.delete(url),
            "PATCH" => self.agent.request("PATCH", url),
            "HEAD" => self.agent.head(url),
            _ => self.agent.request(method, url),
        };

        // Add headers
        for (key, value) in headers {
            request = request.set(&key, &value);
        }

        // Add timeout if present
        if let Some(timeout_duration) = timeout {
            request = request.timeout(timeout_duration);
        }

        // Send the request with or without body
        let response = if let Some(body_data) = body {
            request.send_bytes(&body_data)?
        } else {
            request.call()?
        };

        // Read the response body into a string first, then convert to bytes
        // This is the recommended way to read ureq responses
        let body_str = response.into_string()
            .map_err(|_e| ureq::Error::Status(500,
                ureq::Response::new(500, "IO Error", "Failed to read response body").unwrap()
            ))?;

        Ok(body_str.into_bytes())
    }
}
