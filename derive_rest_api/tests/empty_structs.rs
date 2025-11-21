//! Tests for unit structs and empty named structs
//!
//! These tests verify that RequestBuilder can handle endpoints with no parameters.

use derive_rest_api::RequestBuilder;

#[test]
fn test_unit_struct_builds() {
    #[derive(RequestBuilder)]
    #[request_builder(method = "GET", path = "/health")]
    struct HealthCheck;

    let request = HealthCheckBuilder::new().build().unwrap();
    let url = request.build_url().unwrap();
    assert_eq!(url, "/health");
}

#[test]
fn test_empty_named_struct_builds() {
    #[derive(RequestBuilder)]
    #[request_builder(method = "GET", path = "/status")]
    struct StatusCheck {}

    let request = StatusCheckBuilder::new().build().unwrap();
    let url = request.build_url().unwrap();
    assert_eq!(url, "/status");
}

#[test]
fn test_unit_struct_with_base_url() {
    #[derive(RequestBuilder)]
    #[request_builder(method = "GET", path = "/ping")]
    struct Ping;

    let request = PingBuilder::new()
        .base_url("https://example.com")
        .build()
        .unwrap();

    let url = request.build_url().unwrap();
    assert_eq!(url, "/ping");
}

#[test]
fn test_unit_struct_with_dynamic_headers() {
    use derive_rest_api::RequestModifier;

    #[derive(RequestBuilder)]
    #[request_builder(method = "DELETE", path = "/cache")]
    struct ClearCache;

    // Dynamic headers set via .header() are stored in the builder
    // They don't appear in build_headers() since they're merged during send()
    let builder = ClearCacheBuilder::new()
        .header("X-Clear-All", "true");

    let request = builder.build().unwrap();

    // Unit structs have no header fields, so build_headers() returns empty map
    let headers = request.build_headers();
    assert!(headers.is_empty());

    // The URL should still build correctly
    let url = request.build_url().unwrap();
    assert_eq!(url, "/cache");
}

#[test]
fn test_empty_named_struct_with_response_type() {
    use serde::Deserialize;

    #[derive(Debug, Deserialize, PartialEq)]
    struct HealthStatus {
        status: String,
    }

    #[derive(RequestBuilder)]
    #[request_builder(method = "GET", path = "/health", response = HealthStatus)]
    struct HealthCheck {}

    let request = HealthCheckBuilder::new().build().unwrap();
    let url = request.build_url().unwrap();
    assert_eq!(url, "/health");
}

#[test]
fn test_unit_struct_builder_methods() {
    use derive_rest_api::RequestModifier;

    #[derive(RequestBuilder)]
    #[request_builder(method = "GET", path = "/ready")]
    struct ReadinessCheck;

    // Verify builder has the standard methods
    let builder = ReadinessCheckBuilder::new();
    let builder = builder.base_url("https://api.example.com");
    let builder = builder.timeout(std::time::Duration::from_secs(5));

    let request = builder.build().unwrap();
    let url = request.build_url().unwrap();
    assert_eq!(url, "/ready");
}
