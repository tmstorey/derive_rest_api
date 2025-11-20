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
    ) -> Result<Vec<u8>, Self::Error> {
        Ok(vec![])
    }
}

#[derive(RequestBuilder)]
#[request_builder(method = "GET", path = "/users/{id}")]
struct GetUser {
    id: u64,
}

#[test]
fn test_unit_struct_auto_no_config() {
    // Unit struct should automatically get NoRequestConfiguration
    #[derive(Clone, ApiClient)]
    #[api_client(
        base_url = "https://api.example.com",
        requests(GetUser)
    )]
    struct UnitConfig;

    let config = UnitConfig;
    let client = UnitClient::<MockClient>::new().with_config(config);

    // Should compile and work without manual NoRequestConfiguration impl
    let _builder = client.get_user();
}

#[test]
fn test_empty_struct_auto_no_config() {
    // Empty struct should automatically get NoRequestConfiguration
    #[derive(Clone, ApiClient)]
    #[api_client(
        base_url = "https://api.example.com",
        requests(GetUser)
    )]
    struct EmptyConfig {}

    let config = EmptyConfig {};
    let client = EmptyClient::<MockClient>::new().with_config(config);

    // Should compile and work without manual NoRequestConfiguration impl
    let _builder = client.get_user();
}

#[test]
fn test_empty_tuple_struct_auto_no_config() {
    // Empty tuple struct should automatically get NoRequestConfiguration
    #[derive(Clone, ApiClient)]
    #[api_client(
        base_url = "https://api.example.com",
        requests(GetUser)
    )]
    struct EmptyTupleConfig();

    let config = EmptyTupleConfig();
    let client = EmptyTupleClient::<MockClient>::new().with_config(config);

    // Should compile and work without manual NoRequestConfiguration impl
    let _builder = client.get_user();
}
