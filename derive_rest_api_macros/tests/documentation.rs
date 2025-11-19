use derive_rest_api_macros::RequestBuilder;

/// A user in the system.
///
/// This struct represents a user with their basic information.
#[derive(RequestBuilder, Debug, PartialEq)]
struct User {
    /// The user's unique identifier
    id: u64,

    /// The user's display name
    name: String,

    /// Optional email address
    email: Option<String>,
}

#[test]
fn test_documented_builder() {
    // This test just verifies the code compiles with documentation
    let user = UserBuilder::new()
        .id(1)
        .name("Alice".to_string())
        .email("alice@example.com".to_string())
        .build()
        .unwrap();

    assert_eq!(user.id, 1);
    assert_eq!(user.name, "Alice");
    assert_eq!(user.email, Some("alice@example.com".to_string()));
}

/// Configuration for the application.
///
/// # Examples
///
/// ```ignore
/// let config = AppConfigBuilder::new()
///     .timeout(30)
///     .build()?;
/// ```
#[derive(RequestBuilder, Debug, PartialEq)]
#[request_builder(default)]
struct AppConfig {
    /// Request timeout in seconds
    timeout: u32,

    /// Maximum number of retries
    retries: usize,
}

#[test]
fn test_config_with_docs() {
    let config = AppConfigBuilder::new()
        .timeout(60)
        .build()
        .unwrap();

    assert_eq!(config.timeout, 60);
    assert_eq!(config.retries, 0);
}
