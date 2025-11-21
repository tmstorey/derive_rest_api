use derive_rest_api::{ApiClient, RequestBuilder};

// Mock error type for testing
#[derive(Debug)]
struct MockError(String);

impl std::fmt::Display for MockError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for MockError {}

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

// Implement NoRequestConfiguration since this config doesn't need to modify requests
impl derive_rest_api::NoRequestConfiguration for MyApiConfig {}

#[test]
fn test_client_struct_generation() {
    // Test that the client structs are generated
    let config = MyApiConfig {
        api_key: "test_key".to_string(),
    };

    // Mock HTTP client
    #[derive(Clone, Default)]
    struct MockClient;
    impl derive_rest_api::HttpClient for MockClient {
        type Error = MockError;
        fn send(
            &self,
            _method: &str,
            _url: &str,
            _headers: std::collections::HashMap<String, String>,
            _body: Option<Vec<u8>>,
            _timeout: Option<std::time::Duration>,
        ) -> Result<Vec<u8>, Self::Error> {
            Ok(vec![])
        }
    }

    let client = MyApiClient::<MockClient>::new_with_client(MockClient).with_config(config);

    // Verify we can access the config
    assert_eq!(client.config().clone().unwrap().api_key, "test_key");
}

#[test]
fn test_async_client_struct_generation() {
    let config = MyApiConfig {
        api_key: "test_key".to_string(),
    };

    // Mock async HTTP client
    #[derive(Clone, Default)]
    struct MockAsyncClient;
    impl derive_rest_api::AsyncHttpClient for MockAsyncClient {
        type Error = MockError;
        async fn send_async(
            &self,
            _method: &str,
            _url: &str,
            _headers: std::collections::HashMap<String, String>,
            _body: Option<Vec<u8>>,
            _timeout: Option<std::time::Duration>,
        ) -> Result<Vec<u8>, Self::Error> {
            Ok(vec![])
        }
    }

    let client: MyApiAsyncClient<MockAsyncClient> = MyApiAsyncClient::<MockAsyncClient>::new_with_client(MockAsyncClient).with_config(config);

    // Verify we can access the config
    assert_eq!(client.config().clone().unwrap().api_key, "test_key");
}

#[test]
fn test_method_generation() {
    #[derive(Clone, Default)]
    struct MockClient;
    impl derive_rest_api::HttpClient for MockClient {
        type Error = MockError;
        fn send(
            &self,
            _method: &str,
            _url: &str,
            _headers: std::collections::HashMap<String, String>,
            _body: Option<Vec<u8>>,
            _timeout: Option<std::time::Duration>,
        ) -> Result<Vec<u8>, Self::Error> {
            Ok(vec![])
        }
    }

    let client: MyApiClient<MockClient> = MyApiClient::<MockClient>::new_with_client(MockClient);

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

    #[derive(Clone, Default)]
    struct MockClient;
    impl derive_rest_api::HttpClient for MockClient {
        type Error = MockError;
        fn send(
            &self,
            _method: &str,
            _url: &str,
            _headers: std::collections::HashMap<String, String>,
            _body: Option<Vec<u8>>,
            _timeout: Option<std::time::Duration>,
        ) -> Result<Vec<u8>, Self::Error> {
            Ok(vec![])
        }
    }

    let client = MyApiClient::<MockClient>::new_with_client(MockClient)
        .with_config(config)
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

    impl derive_rest_api::NoRequestConfiguration for GithubApiConfig {}

    let config = GithubApiConfig {
        token: "test_token".to_string(),
    };

    #[derive(Clone, Default)]
    struct MockClient;
    impl derive_rest_api::HttpClient for MockClient {
        type Error = MockError;
        fn send(
            &self,
            _method: &str,
            _url: &str,
            _headers: std::collections::HashMap<String, String>,
            _body: Option<Vec<u8>>,
            _timeout: Option<std::time::Duration>,
        ) -> Result<Vec<u8>, Self::Error> {
            Ok(vec![])
        }
    }

    // Should generate GithubApiClient, not GithubApiConfigClient
    let _client = GithubApiClient::<MockClient>::new_with_client(MockClient).with_config(config);
}

#[test]
fn test_default_attribute() {
    // Test that the default attribute causes the config to be initialized with Default::default()
    #[derive(Clone, Default, ApiClient)]
    #[api_client(
        base_url = "https://api.example.com",
        requests(GetUser),
        default
    )]
    struct DefaultableConfig {
        api_key: String,
    }

    impl derive_rest_api::NoRequestConfiguration for DefaultableConfig {}

    #[derive(Clone, Default)]
    struct MockClient;
    impl derive_rest_api::HttpClient for MockClient {
        type Error = MockError;
        fn send(
            &self,
            _method: &str,
            _url: &str,
            _headers: std::collections::HashMap<String, String>,
            _body: Option<Vec<u8>>,
            _timeout: Option<std::time::Duration>,
        ) -> Result<Vec<u8>, Self::Error> {
            Ok(vec![])
        }
    }

    // Test blocking client
    let blocking_client = DefaultableClient::<MockClient>::new_with_client(MockClient);

    // Config should be initialized with default value
    assert!(blocking_client.config().is_some());
    assert_eq!(blocking_client.config().as_ref().unwrap().api_key, "");

    // Test async client
    #[derive(Clone, Default)]
    struct MockAsyncClient;
    impl derive_rest_api::AsyncHttpClient for MockAsyncClient {
        type Error = MockError;
        async fn send_async(
            &self,
            _method: &str,
            _url: &str,
            _headers: std::collections::HashMap<String, String>,
            _body: Option<Vec<u8>>,
            _timeout: Option<std::time::Duration>,
        ) -> Result<Vec<u8>, Self::Error> {
            Ok(vec![])
        }
    }

    let async_client = DefaultableAsyncClient::<MockAsyncClient>::new_with_client(MockAsyncClient);
    assert!(async_client.config().is_some());
    assert_eq!(async_client.config().as_ref().unwrap().api_key, "");
}

#[allow(dead_code)]
#[test]
fn test_without_default_attribute() {
    // Test that without the default attribute, config is None by default
    #[derive(Clone, Default, ApiClient)]
    #[api_client(
        base_url = "https://api.example.com",
        requests(GetUser)
    )]
    struct NonDefaultableConfig {
        api_key: String,
    }

    impl derive_rest_api::NoRequestConfiguration for NonDefaultableConfig {}

    #[derive(Clone, Default)]
    struct MockClient;
    impl derive_rest_api::HttpClient for MockClient {
        type Error = MockError;
        fn send(
            &self,
            _method: &str,
            _url: &str,
            _headers: std::collections::HashMap<String, String>,
            _body: Option<Vec<u8>>,
            _timeout: Option<std::time::Duration>,
        ) -> Result<Vec<u8>, Self::Error> {
            Ok(vec![])
        }
    }

    // Test blocking client
    let blocking_client = NonDefaultableClient::<MockClient>::new_with_client(MockClient);

    // Config should be None without the default attribute
    assert!(blocking_client.config().is_none());

    // Test async client
    #[derive(Clone, Default)]
    struct MockAsyncClient;
    impl derive_rest_api::AsyncHttpClient for MockAsyncClient {
        type Error = MockError;
        async fn send_async(
            &self,
            _method: &str,
            _url: &str,
            _headers: std::collections::HashMap<String, String>,
            _body: Option<Vec<u8>>,
            _timeout: Option<std::time::Duration>,
        ) -> Result<Vec<u8>, Self::Error> {
            Ok(vec![])
        }
    }

    let async_client = NonDefaultableAsyncClient::<MockAsyncClient>::new_with_client(MockAsyncClient);
    assert!(async_client.config().is_none());
}
