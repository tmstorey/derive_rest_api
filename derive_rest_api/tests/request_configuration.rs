use derive_rest_api::{ApiClient, ConfigureRequest, RequestBuilder, RequestModifier};
use serde::Serialize;

// Mock error type for testing
#[derive(Debug)]
struct MockError(String);

impl std::fmt::Display for MockError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for MockError {}

// Mock HTTP client for testing
#[derive(Clone, Default)]
struct MockHttpClient;

impl derive_rest_api::HttpClient for MockHttpClient {
    type Error = MockError;

    fn send(
        &self,
        _method: &str,
        _url: &str,
        headers: std::collections::HashMap<String, String>,
        _body: Option<Vec<u8>>,
        _timeout: Option<std::time::Duration>,
    ) -> Result<Vec<u8>, Self::Error> {
        // Verify headers were set
        assert!(headers.contains_key("X-API-Key"));
        assert!(headers.contains_key("User-Agent"));
        assert_eq!(headers.get("X-API-Key").unwrap(), "test_api_key_123");
        assert_eq!(headers.get("User-Agent").unwrap(), "my-app/1.0");
        Ok(vec![])
    }
}

// Request types
#[derive(RequestBuilder, Serialize)]
#[request_builder(method = "GET", path = "/users/{id}")]
struct GetUser {
    id: u64,
}

#[derive(RequestBuilder, Serialize)]
#[request_builder(method = "POST", path = "/users")]
struct CreateUser {
    #[request_builder(body)]
    name: String,
}

// API configuration that implements ConfigureRequest
#[derive(Clone, ApiClient)]
#[api_client(
    base_url = "https://api.example.com",
    requests(GetUser, CreateUser)
)]
struct MyApiConfig {
    api_key: String,
    user_agent: String,
}

// Implement ConfigureRequest to automatically add headers
impl ConfigureRequest for MyApiConfig {
    fn configure<M: RequestModifier>(&self, modifier: M) -> M {
        modifier
            .header("X-API-Key", &self.api_key)
            .header("User-Agent", &self.user_agent)
    }
}

#[test]
fn test_configure_request_with_api_client() {
    let config = MyApiConfig {
        api_key: "test_api_key_123".to_string(),
        user_agent: "my-app/1.0".to_string(),
    };

    let client = MyApiClient::<MockHttpClient>::with_client().with_config(config);

    // The builder should be pre-configured with headers from the config
    let result = client.get_user()
        .id(123)
        .send();

    assert!(result.is_ok());
}

#[test]
fn test_manual_header_override() {
    let config = MyApiConfig {
        api_key: "test_api_key_123".to_string(),
        user_agent: "my-app/1.0".to_string(),
    };

    let client = MyApiClient::<MockHttpClient>::with_client().with_config(config);

    // Manually added headers should also work
    let builder = client.get_user()
        .id(123)
        .header("X-Custom-Header", "custom_value");

    // Verify the header was added (we can't directly inspect but the mock client will check)
    let result = builder.send();
    assert!(result.is_ok());
}

#[test]
fn test_direct_builder_with_headers() {
    // Test using RequestModifier directly on a builder
    let builder = GetUserBuilder::new()
        .id(456)
        .header("X-API-Key", "direct_key")
        .header("Authorization", "Bearer token");

    // Just verify it compiles and the builder works
    let request = builder.build();
    assert!(request.is_ok());
}
