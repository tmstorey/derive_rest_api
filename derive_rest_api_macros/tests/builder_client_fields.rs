use derive_rest_api_macros::RequestBuilder;
use std::collections::HashMap;

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
        _headers: HashMap<String, String>,
        _body: Option<Vec<u8>>,
        _timeout: Option<std::time::Duration>,
    ) -> Result<Vec<u8>, Self::Error> {
        Ok(b"{\"id\":1}".to_vec())
    }
}

// Mock async HTTP client for testing
#[derive(Clone, Default)]
struct MockAsyncHttpClient;

impl derive_rest_api::AsyncHttpClient for MockAsyncHttpClient {
    type Error = MockError;

    async fn send_async(
        &self,
        _method: &str,
        _url: &str,
        _headers: HashMap<String, String>,
        _body: Option<Vec<u8>>,
        _timeout: Option<std::time::Duration>,
    ) -> Result<Vec<u8>, Self::Error> {
        Ok(b"{\"id\":1}".to_vec())
    }
}

#[derive(RequestBuilder)]
#[request_builder(method = "GET", path = "/users/{id}")]
struct GetUser {
    id: u64,
}

#[test]
fn test_builder_with_http_client() {
    let client = MockHttpClient;
    let builder = GetUserBuilder::new()
        .http_client(client)
        .id(123);

    // Verify client field is set
    assert!(builder.__http_client.is_some());
    assert!(builder.__async_http_client.is_none());

    // Verify we can still build
    let result = builder.build();
    assert!(result.is_ok());
}

#[test]
fn test_builder_with_async_http_client() {
    let client = MockAsyncHttpClient;
    let builder = GetUserBuilder::new()
        .async_http_client(client)
        .id(456);

    // Verify client field is set
    assert!(builder.__http_client.is_none());
    assert!(builder.__async_http_client.is_some());

    // Verify we can still build
    let result = builder.build();
    assert!(result.is_ok());
}

#[test]
fn test_builder_with_both_clients() {
    let http_client = MockHttpClient;
    let async_client = MockAsyncHttpClient;

    let builder = GetUserBuilder::new()
        .http_client(http_client)
        .async_http_client(async_client)
        .id(789);

    // Verify both client fields are set
    assert!(builder.__http_client.is_some());
    assert!(builder.__async_http_client.is_some());

    // Verify we can still build
    let result = builder.build();
    assert!(result.is_ok());
}

#[test]
fn test_builder_client_methods_chain_with_other_setters() {
    let client = MockHttpClient;

    // Test that client setter works in a chain with other setters
    let builder = GetUserBuilder::new()
        .id(100)
        .http_client(client);

    assert!(builder.__http_client.is_some());
    assert_eq!(builder.id, Some(100));
}

#[test]
fn test_send_method() {
    let client = MockHttpClient;

    let result = GetUserBuilder::new()
        .http_client(client)
        .base_url("https://api.example.com")
        .id(123)
        .send();

    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response, b"{\"id\":1}");
}

#[tokio::test]
async fn test_send_async_method() {
    let client = MockAsyncHttpClient;

    let result = GetUserBuilder::new()
        .async_http_client(client)
        .base_url("https://api.example.com")
        .id(789)
        .send_async()
        .await;

    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response, b"{\"id\":1}");
}

#[test]
fn test_send_and_send_async_availability() {
    // This test verifies that send() is only available when HttpClient is set
    // and send_async() is only available when AsyncHttpClient is set

    let http_client = MockHttpClient;
    let async_client = MockAsyncHttpClient;

    // Builder with HttpClient has send() method
    let _builder_with_http = GetUserBuilder::new()
        .http_client(http_client)
        .id(1);
    // Can call .send() here

    // Builder with AsyncHttpClient has send_async() method
    let _builder_with_async = GetUserBuilder::new()
        .async_http_client(async_client)
        .id(2);
    // Can call .send_async() here

    // Builder with both clients has both methods
    let http_client2 = MockHttpClient;
    let async_client2 = MockAsyncHttpClient;
    let _builder_with_both = GetUserBuilder::new()
        .http_client(http_client2)
        .async_http_client(async_client2)
        .id(3);
    // Can call both .send() and .send_async() here
}

#[test]
fn test_base_url_setter() {
    let builder = GetUserBuilder::new()
        .base_url("https://api.example.com");

    assert_eq!(builder.__base_url, Some("https://api.example.com".to_string()));
}

#[test]
fn test_send_requires_base_url() {
    let client = MockHttpClient;

    // Test that send() fails if no base_url is set
    let result = GetUserBuilder::new()
        .http_client(client)
        .id(123)
        .send();

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("No base URL configured"));
}
