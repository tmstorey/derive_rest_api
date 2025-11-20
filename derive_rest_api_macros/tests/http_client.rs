use derive_rest_api_macros::RequestBuilder;
use serde::Serialize;
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
#[derive(Clone)]
struct MockHttpClient {
    response: Vec<u8>,
}

impl MockHttpClient {
    fn new(response: &str) -> Self {
        Self {
            response: response.as_bytes().to_vec(),
        }
    }
}

impl derive_rest_api::HttpClient for MockHttpClient {
    type Error = MockError;

    fn send(
        &self,
        _method: &str,
        _url: &str,
        _headers: HashMap<String, String>,
        _body: Option<Vec<u8>>,
    ) -> Result<Vec<u8>, Self::Error> {
        Ok(self.response.clone())
    }
}

#[test]
fn test_send_with_client_basic() {
    #[derive(RequestBuilder)]
    #[request_builder(method = "GET", path = "/api/users/{id}")]
    struct GetUser {
        id: u64,
    }

    let request = GetUserBuilder::new()
        .id(123)
        .build()
        .unwrap();

    let client = MockHttpClient::new(r#"{"id": 123, "name": "Alice"}"#);
    let response = request.send_with_client(&client, "https://api.example.com").unwrap();

    assert_eq!(response, br#"{"id": 123, "name": "Alice"}"#);
}

#[test]
fn test_send_with_client_query_params() {
    #[derive(RequestBuilder, Serialize)]
    #[request_builder(method = "GET", path = "/api/users")]
    struct ListUsers {
        #[request_builder(query)]
        page: Option<u32>,
        #[request_builder(query)]
        limit: Option<u32>,
    }

    let request = ListUsersBuilder::new()
        .page(2)
        .limit(10)
        .build()
        .unwrap();

    let client = MockHttpClient::new(r#"[{"id": 1}, {"id": 2}]"#);
    let response = request.send_with_client(&client, "https://api.example.com").unwrap();

    assert_eq!(response, br#"[{"id": 1}, {"id": 2}]"#);
}

#[test]
fn test_build_body_with_fields() {
    #[derive(RequestBuilder, Serialize)]
    #[request_builder(method = "POST", path = "/api/users")]
    struct CreateUser {
        #[request_builder(body)]
        name: String,
        #[request_builder(body)]
        email: String,
    }

    let request = CreateUserBuilder::new()
        .name("Alice".to_string())
        .email("alice@example.com".to_string())
        .build()
        .unwrap();

    let body = request.build_body().unwrap();
    assert!(body.is_some());

    let json: serde_json::Value = serde_json::from_slice(&body.unwrap()).unwrap();
    assert_eq!(json["name"], "Alice");
    assert_eq!(json["email"], "alice@example.com");
}

#[test]
#[allow(dead_code)]
fn test_build_body_no_fields() {
    #[derive(RequestBuilder)]
    #[request_builder(method = "GET", path = "/api/users/{id}")]
    struct GetUser {
        id: u64,
    }

    let request = GetUserBuilder::new()
        .id(123)
        .build()
        .unwrap();

    let body = request.build_body().unwrap();
    assert!(body.is_none());
}

#[test]
fn test_build_body_optional_fields() {
    #[derive(RequestBuilder, Serialize)]
    #[request_builder(method = "PATCH", path = "/api/users/{id}")]
    struct UpdateUser {
        id: u64,
        #[request_builder(body)]
        name: Option<String>,
        #[request_builder(body)]
        email: Option<String>,
    }

    let request = UpdateUserBuilder::new()
        .id(123)
        .name("Bob".to_string())
        .build()
        .unwrap();

    let body = request.build_body().unwrap();
    assert!(body.is_some());

    let json: serde_json::Value = serde_json::from_slice(&body.unwrap()).unwrap();
    assert_eq!(json["name"], "Bob");
    assert!(json.get("email").is_none() || json["email"].is_null());
}

#[test]
fn test_build_headers_with_fields() {
    #[derive(RequestBuilder)]
    #[request_builder(method = "GET", path = "/api/users")]
    struct GetUsers {
        #[request_builder(header)]
        authorization: String,
        #[request_builder(header)]
        x_api_key: String,
    }

    let request = GetUsersBuilder::new()
        .authorization("Bearer token123".to_string())
        .x_api_key("key456".to_string())
        .build()
        .unwrap();

    let headers = request.build_headers();
    // Header names should be converted from snake_case to Title-Case
    assert_eq!(headers.get("Authorization"), Some(&"Bearer token123".to_string()));
    assert_eq!(headers.get("X-Api-Key"), Some(&"key456".to_string()));
}

#[test]
#[allow(dead_code)]
fn test_build_headers_no_fields() {
    #[derive(RequestBuilder)]
    #[request_builder(method = "GET", path = "/api/users/{id}")]
    struct GetUser {
        id: u64,
    }

    let request = GetUserBuilder::new()
        .id(123)
        .build()
        .unwrap();

    let headers = request.build_headers();
    assert!(headers.is_empty());
}

#[test]
fn test_build_headers_optional_fields() {
    #[derive(RequestBuilder)]
    #[request_builder(method = "GET", path = "/api/users")]
    struct GetUsers {
        #[request_builder(header)]
        authorization: Option<String>,
        #[request_builder(header)]
        x_custom_header: Option<String>,
    }

    // With both headers set
    let request1 = GetUsersBuilder::new()
        .authorization("Bearer token".to_string())
        .x_custom_header("custom".to_string())
        .build()
        .unwrap();

    let headers1 = request1.build_headers();
    assert_eq!(headers1.len(), 2);
    assert_eq!(headers1.get("Authorization"), Some(&"Bearer token".to_string()));

    // With only one header set
    let request2 = GetUsersBuilder::new()
        .authorization("Bearer token".to_string())
        .build()
        .unwrap();

    let headers2 = request2.build_headers();
    assert_eq!(headers2.len(), 1);
    assert_eq!(headers2.get("Authorization"), Some(&"Bearer token".to_string()));
    assert!(headers2.get("X-Custom-Header").is_none());
}

#[test]
fn test_mixed_field_types() {
    #[derive(RequestBuilder, Serialize)]
    #[request_builder(method = "POST", path = "/api/users/{id}/posts")]
    struct CreatePost {
        id: u64,
        #[request_builder(query)]
        publish: Option<bool>,
        #[request_builder(body)]
        title: String,
        #[request_builder(body)]
        content: String,
        #[request_builder(header)]
        authorization: String,
    }

    let request = CreatePostBuilder::new()
        .id(123)
        .publish(true)
        .title("Hello World".to_string())
        .content("This is a test post".to_string())
        .authorization("Bearer token".to_string())
        .build()
        .unwrap();

    // Test URL
    let url = request.build_url().unwrap();
    assert_eq!(url, "/api/users/123/posts?publish=true");

    // Test body
    let body = request.build_body().unwrap();
    assert!(body.is_some());
    let json: serde_json::Value = serde_json::from_slice(&body.unwrap()).unwrap();
    assert_eq!(json["title"], "Hello World");
    assert_eq!(json["content"], "This is a test post");

    // Test headers
    let headers = request.build_headers();
    assert_eq!(headers.get("Authorization"), Some(&"Bearer token".to_string()));
}

#[test]
fn test_send_with_client_full_request() {
    #[derive(RequestBuilder, Serialize)]
    #[request_builder(method = "PUT", path = "/api/users/{id}")]
    struct UpdateUser {
        id: u64,
        #[request_builder(query)]
        notify: Option<bool>,
        #[request_builder(body)]
        name: String,
        #[request_builder(header)]
        authorization: String,
    }

    let request = UpdateUserBuilder::new()
        .id(42)
        .notify(true)
        .name("Updated Name".to_string())
        .authorization("Bearer secret".to_string())
        .build()
        .unwrap();

    let client = MockHttpClient::new(r#"{"success": true}"#);
    let response = request.send_with_client(&client, "https://api.example.com").unwrap();

    assert_eq!(response, br#"{"success": true}"#);
}

#[test]
fn test_custom_header_name() {
    #[derive(RequestBuilder)]
    #[request_builder(method = "GET", path = "/api/data")]
    struct GetData {
        #[request_builder(header = "X-Custom-Auth")]
        auth_token: String,
        #[request_builder(header = "X-Request-ID")]
        request_id: Option<String>,
    }

    let request = GetDataBuilder::new()
        .auth_token("secret123".to_string())
        .request_id("abc-123".to_string())
        .build()
        .unwrap();

    let headers = request.build_headers();
    // Should use custom names, not converted field names
    assert_eq!(headers.get("X-Custom-Auth"), Some(&"secret123".to_string()));
    assert_eq!(headers.get("X-Request-ID"), Some(&"abc-123".to_string()));
    // Should NOT have the converted field names
    assert!(headers.get("Auth-Token").is_none());
    assert!(headers.get("Request-Id").is_none());
}

#[test]
fn test_serde_rename_in_body() {
    #[derive(RequestBuilder, Serialize)]
    #[request_builder(method = "POST", path = "/api/users")]
    struct CreateUser {
        #[request_builder(body)]
        #[serde(rename = "userName")]
        user_name: String,
        #[request_builder(body)]
        #[serde(rename = "emailAddress")]
        email: String,
    }

    let request = CreateUserBuilder::new()
        .user_name("johndoe".to_string())
        .email("john@example.com".to_string())
        .build()
        .unwrap();

    let body = request.build_body().unwrap();
    assert!(body.is_some());

    let json: serde_json::Value = serde_json::from_slice(&body.unwrap()).unwrap();
    // Should use serde rename
    assert_eq!(json["userName"], "johndoe");
    assert_eq!(json["emailAddress"], "john@example.com");
    // Should NOT have the field names
    assert!(json.get("user_name").is_none());
    assert!(json.get("email").is_none());
}

#[test]
fn test_serde_rename_in_query() {
    #[derive(RequestBuilder, Serialize)]
    #[request_builder(path = "/api/search")]
    struct SearchRequest {
        #[request_builder(query)]
        #[serde(rename = "q")]
        search_query: Option<String>,
        #[request_builder(query)]
        #[serde(rename = "maxResults")]
        max_results: Option<u32>,
    }

    let request = SearchRequestBuilder::new()
        .search_query("rust programming".to_string())
        .max_results(50)
        .build()
        .unwrap();

    let url = request.build_url().unwrap();
    // Should use serde rename in query string
    assert!(url.contains("q=rust"));
    assert!(url.contains("maxResults=50"));
    // Should NOT have the field names
    assert!(!url.contains("search_query"));
    assert!(!url.contains("max_results"));
}
