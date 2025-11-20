use derive_rest_api_macros::{ApiClient, RequestBuilder};

// Mock request structs
#[derive(RequestBuilder)]
#[request_builder(method = "GET", path = "/users/{id}")]
struct GetUser {
    id: u64,
}

#[derive(RequestBuilder)]
#[request_builder(method = "POST", path = "/users")]
#[allow(dead_code)]
struct CreateUser {
    name: String,
}

#[derive(RequestBuilder)]
#[request_builder(method = "DELETE", path = "/users/{id}")]
struct DeleteUser {
    id: u64,
}

// API configuration
#[derive(Clone, ApiClient)]
#[api_client(
    base_url = "https://api.example.com",
    requests(GetUser, CreateUser = "new_user", DeleteUser)
)]
struct MyApiConfig {
    api_key: String,
}

#[test]
fn test_client_struct_generation() {
    // Test that the client structs are generated
    let config = MyApiConfig {
        api_key: "test_key".to_string(),
    };

    // Mock HTTP client
    #[derive(Clone)]
    struct MockClient;
    impl derive_rest_api::HttpClient for MockClient {
        type Error = String;
        fn send(
            &self,
            _method: &str,
            _url: &str,
            _headers: std::collections::HashMap<String, String>,
            _body: Option<Vec<u8>>,
        ) -> Result<Vec<u8>, Self::Error> {
            Ok(vec![])
        }
    }

    let client = MyApiClient::new(config.clone(), MockClient);

    // Verify we can access the config
    assert_eq!(client.config().api_key, "test_key");
}

#[test]
fn test_async_client_struct_generation() {
    let config = MyApiConfig {
        api_key: "test_key".to_string(),
    };

    // Mock async HTTP client
    #[derive(Clone)]
    struct MockAsyncClient;
    impl derive_rest_api::AsyncHttpClient for MockAsyncClient {
        type Error = String;
        async fn send_async(
            &self,
            _method: &str,
            _url: &str,
            _headers: std::collections::HashMap<String, String>,
            _body: Option<Vec<u8>>,
        ) -> Result<Vec<u8>, Self::Error> {
            Ok(vec![])
        }
    }

    let client = MyApiAsyncClient::new(config.clone(), MockAsyncClient);

    // Verify we can access the config
    assert_eq!(client.config().api_key, "test_key");
}

#[test]
fn test_method_generation() {
    let config = MyApiConfig {
        api_key: "test_key".to_string(),
    };

    #[derive(Clone)]
    struct MockClient;
    impl derive_rest_api::HttpClient for MockClient {
        type Error = String;
        fn send(
            &self,
            _method: &str,
            _url: &str,
            _headers: std::collections::HashMap<String, String>,
            _body: Option<Vec<u8>>,
        ) -> Result<Vec<u8>, Self::Error> {
            Ok(vec![])
        }
    }

    let client = MyApiClient::new(config, MockClient);

    // Test that methods exist and return builders
    let _get_user_builder = client.get_user();
    let _create_user_builder = client.new_user(); // Custom name
    let _delete_user_builder = client.delete_user();
}

#[test]
fn test_with_base_url() {
    let config = MyApiConfig {
        api_key: "test_key".to_string(),
    };

    #[derive(Clone)]
    struct MockClient;
    impl derive_rest_api::HttpClient for MockClient {
        type Error = String;
        fn send(
            &self,
            _method: &str,
            _url: &str,
            _headers: std::collections::HashMap<String, String>,
            _body: Option<Vec<u8>>,
        ) -> Result<Vec<u8>, Self::Error> {
            Ok(vec![])
        }
    }

    let client = MyApiClient::new(config, MockClient)
        .with_base_url("https://custom.example.com");

    // Base URL should be updated
    assert_eq!(client.base_url, "https://custom.example.com");
}

#[test]
#[allow(dead_code)]
fn test_config_suffix_stripping() {
    // Test that "Config" suffix is stripped from struct names
    #[derive(Clone, ApiClient)]
    #[api_client(
        base_url = "https://github.com/api",
        requests(GetUser)
    )]
    struct GithubApiConfig {
        token: String,
    }

    let config = GithubApiConfig {
        token: "test_token".to_string(),
    };

    #[derive(Clone)]
    struct MockClient;
    impl derive_rest_api::HttpClient for MockClient {
        type Error = String;
        fn send(
            &self,
            _method: &str,
            _url: &str,
            _headers: std::collections::HashMap<String, String>,
            _body: Option<Vec<u8>>,
        ) -> Result<Vec<u8>, Self::Error> {
            Ok(vec![])
        }
    }

    // Should generate GithubApiClient, not GithubApiConfigClient
    let _client = GithubApiClient::new(config, MockClient);
}
