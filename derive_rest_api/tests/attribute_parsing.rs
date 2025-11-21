use derive_rest_api::RequestBuilder;

// Test that struct-level attributes are parsed without errors
#[test]
#[allow(dead_code)]
fn test_method_attribute() {
    #[derive(RequestBuilder)]
    #[request_builder(method = "GET")]
    #[expect(unused)]
    struct GetUsers {
        limit: Option<u32>,
    }

    // Should compile without errors
    let _builder = GetUsersBuilder::new();
}

#[test]
fn test_path_attribute() {
    #[derive(RequestBuilder)]
    #[request_builder(path = "/api/users/{id}")]
    #[expect(unused)]
    struct GetUser {
        id: u64,
    }

    let _builder = GetUserBuilder::new();
}

#[test]
fn test_response_attribute() {
    #[derive(Debug, PartialEq)]
    #[expect(unused)]
    struct User {
        name: String,
    }

    #[derive(RequestBuilder)]
    #[request_builder(response = User)]
    #[expect(unused)]
    struct GetUser {
        id: u64,
    }

    let _builder = GetUserBuilder::new();
}

#[test]
fn test_combined_struct_attributes() {
    #[derive(RequestBuilder)]
    #[request_builder(
        into,
        method = "POST",
        path = "/api/posts",
        response = String
    )]
    #[expect(unused)]
    struct CreatePost {
        title: String,
        content: String,
    }

    // Should have Into support from struct-level attribute
    let _post = CreatePostBuilder::new()
        .title("Test")
        .content("Content")
        .build()
        .unwrap();
}

#[test]
#[allow(dead_code)]
fn test_field_path_attribute() {
    #[derive(RequestBuilder)]
    #[expect(unused)]
    struct GetUser {
        #[request_builder(path)]
        id: u64,
        #[request_builder(query)]
        include_posts: Option<bool>,
    }

    let _builder = GetUserBuilder::new().id(123);
}

#[test]
#[allow(dead_code)]
fn test_field_query_attribute() {
    #[derive(RequestBuilder)]
    #[expect(unused)]
    struct SearchUsers {
        #[request_builder(query)]
        name: String,
        #[request_builder(query)]
        limit: Option<u32>,
    }

    let _builder = SearchUsersBuilder::new()
        .name("alice".to_string());
}

#[test]
fn test_field_body_attribute() {
    #[derive(RequestBuilder)]
    #[expect(unused)]
    struct CreateUser {
        #[request_builder(body)]
        name: String,
        #[request_builder(body)]
        email: String,
    }

    let _builder = CreateUserBuilder::new()
        .name("alice".to_string())
        .email("alice@example.com".to_string());
}

#[test]
#[allow(dead_code)]
fn test_field_header_attribute() {
    #[derive(RequestBuilder)]
    #[expect(unused)]
    struct AuthenticatedRequest {
        #[request_builder(header)]
        authorization: String,
        #[request_builder(query)]
        page: Option<u32>,
    }

    let _builder = AuthenticatedRequestBuilder::new()
        .authorization("Bearer token".to_string());
}

#[test]
#[allow(dead_code)]
fn test_mixed_field_attributes() {
    #[derive(RequestBuilder)]
    #[request_builder(method = "PUT", path = "/api/users/{id}")]
    struct UpdateUser {
        #[request_builder(path)]
        id: u64,
        #[request_builder(body, into)]
        name: String,
        #[request_builder(body)]
        email: Option<String>,
        #[request_builder(header, into)]
        authorization: String,
    }

    // Field with 'into' should accept &str
    let _builder = UpdateUserBuilder::new()
        .id(123)
        .name("alice")
        .authorization("Bearer token");
}

#[test]
fn test_default_attribute() {
    #[derive(RequestBuilder, Debug, PartialEq)]
    struct Config {
        name: String,
        #[request_builder(default)]
        timeout: u32,
        #[request_builder(default)]
        retries: usize,
    }

    // Build without setting fields with default attribute
    let config = ConfigBuilder::new()
        .name("production".to_string())
        .build()
        .unwrap();

    assert_eq!(config.name, "production");
    assert_eq!(config.timeout, 0); // Default for u32
    assert_eq!(config.retries, 0); // Default for usize
}

#[test]
fn test_default_attribute_with_custom_values() {
    #[derive(RequestBuilder, Debug, PartialEq)]
    struct Config {
        name: String,
        #[request_builder(default)]
        timeout: u32,
    }

    // Build with setting the default field
    let config = ConfigBuilder::new()
        .name("staging".to_string())
        .timeout(5000)
        .build()
        .unwrap();

    assert_eq!(config.name, "staging");
    assert_eq!(config.timeout, 5000); // Custom value, not default
}

#[test]
fn test_default_with_optional_field() {
    #[derive(RequestBuilder, Debug, PartialEq)]
    struct Settings {
        #[request_builder(default)]
        enabled: bool,
        description: Option<String>,
    }

    // Build with just optional field
    let settings = SettingsBuilder::new()
        .description("test".to_string())
        .build()
        .unwrap();

    assert_eq!(settings.enabled, false); // Default for bool
    assert_eq!(settings.description, Some("test".to_string()));
}

#[test]
fn test_struct_level_default() {
    #[derive(RequestBuilder, Debug, PartialEq)]
    #[request_builder(default)]
    struct ServerConfig {
        host: String,
        port: u16,
        timeout_ms: u64,
        retries: usize,
    }

    // Build without setting any fields - all use default
    let config = ServerConfigBuilder::new()
        .build()
        .unwrap();

    assert_eq!(config.host, "");           // Default for String
    assert_eq!(config.port, 0);            // Default for u16
    assert_eq!(config.timeout_ms, 0);      // Default for u64
    assert_eq!(config.retries, 0);         // Default for usize
}

#[test]
fn test_struct_level_default_with_overrides() {
    #[derive(RequestBuilder, Debug, PartialEq)]
    #[request_builder(default)]
    struct AppConfig {
        name: String,
        debug: bool,
        max_connections: usize,
    }

    // Override some fields
    let config = AppConfigBuilder::new()
        .name("MyApp".to_string())
        .debug(true)
        .build()
        .unwrap();

    assert_eq!(config.name, "MyApp");
    assert_eq!(config.debug, true);
    assert_eq!(config.max_connections, 0); // Still uses default
}

#[test]
fn test_struct_default_with_field_override() {
    #[derive(RequestBuilder, Debug, PartialEq)]
    #[request_builder(default, into)]
    struct MixedConfig {
        name: String,
        enabled: bool,
    }

    // Struct has both default and into
    let config = MixedConfigBuilder::new()
        .name("test") // Into from &str
        .build()
        .unwrap();

    assert_eq!(config.name, "test");
    assert_eq!(config.enabled, false); // Default
}
