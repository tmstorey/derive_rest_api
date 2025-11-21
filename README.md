# derive_rest_api

A Rust procedural macro library for generating type-safe builder patterns for REST API request structures with support for multiple HTTP client backends.

## Features

- ✅ Automatic builder pattern generation
- ✅ URL path parameter templating with `{param}` syntax
- ✅ Query string serialization with `serde_qs`
- ✅ Request body serialization with `serde_json`
- ✅ HTTP header management with auto-conversion to Title-Case
- ✅ Field validation with custom validator functions
- ✅ Flexible type conversion with `Into<T>`
- ✅ Default value handling
- ✅ Full serde attribute support (`#[serde(rename = "...")]`, etc.)
- ✅ Generic HTTP client trait for pluggable backends
- ✅ Built-in reqwest support (blocking and async)
- ✅ Built-in ureq support (lightweight blocking client)
- ✅ High-level API client generation with `#[derive(ApiClient)]`
- ✅ Type-safe error handling with `thiserror`

## Quick Start

Add this to your `Cargo.toml`:

```toml
[dependencies]
derive_rest_api = { path = "derive_rest_api" }
serde = { version = "1.0", features = ["derive"] }

# Optional: for reqwest blocking client
# derive_rest_api = { path = "derive_rest_api", features = ["reqwest-blocking"] }

# Optional: for reqwest async client
# derive_rest_api = { path = "derive_rest_api", features = ["reqwest-async"] }
```

## Usage

### Basic Example

```rust
use derive_rest_api::RequestBuilder;
use serde::Serialize;

#[derive(RequestBuilder, Serialize)]
#[request_builder(method = "GET", path = "/users/{id}")]
struct GetUser {
    id: u64,

    #[request_builder(query)]
    include_posts: Option<bool>,
}

fn main() {
    let request = GetUserBuilder::new()
        .id(123)
        .include_posts(true)
        .build()
        .unwrap();

    let url = request.build_url().unwrap();
    println!("URL: {}", url); // "/users/123?include_posts=true"
}
```

### With Reqwest (Blocking)

```rust
use derive_rest_api::{RequestBuilder, ReqwestBlockingClient};
use serde::Serialize;

#[derive(RequestBuilder, Serialize)]
#[request_builder(method = "POST", path = "/posts")]
struct CreatePost {
    #[request_builder(body)]
    title: String,

    #[request_builder(body)]
    #[serde(rename = "userId")]
    user_id: u64,

    #[request_builder(header = "Authorization")]
    authorization: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = ReqwestBlockingClient::new()?;

    let request = CreatePostBuilder::new()
        .title("Hello World".to_string())
        .user_id(1)
        .authorization("Bearer token123".to_string())
        .build()?;

    let response = request.send_with_client(
        &client,
        "https://api.example.com"
    )?;

    Ok(())
}
```

### With Reqwest (Async)

```rust
use derive_rest_api::{AsyncHttpClient, RequestBuilder, ReqwestAsyncClient};
use serde::Serialize;

#[derive(RequestBuilder, Serialize)]
#[request_builder(method = "GET", path = "/users/{id}")]
struct GetUser {
    id: u64,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = ReqwestAsyncClient::new()?;

    let request = GetUserBuilder::new()
        .id(123)
        .build()?;

    let url = request.build_url()?;
    let response = client.send_async(
        "GET",
        &format!("https://api.example.com{}", url),
        request.build_headers(),
        request.build_body()?,
    ).await?;

    Ok(())
}
```

### With Ureq (Blocking)

```rust
use derive_rest_api::{RequestBuilder, UreqBlockingClient};
use serde::Serialize;

#[derive(RequestBuilder, Serialize)]
#[request_builder(method = "POST", path = "/posts")]
struct CreatePost {
    #[request_builder(body)]
    title: String,
    #[request_builder(body)]
    user_id: u64,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = UreqBlockingClient::new();

    let request = CreatePostBuilder::new()
        .title("Hello World".to_string())
        .user_id(1)
        .build()?;

    let response = request.send_with_client(
        &client,
        "https://api.example.com"
    )?;

    Ok(())
}
```

### High-Level API Client

For a more ergonomic experience, use the `ApiClient` derive macro to generate a high-level client that wraps your configuration and request types:

```rust
use derive_rest_api::{ApiClient, RequestBuilder, ReqwestBlockingClient};
use serde::Serialize;

// Define your request types
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

// Define your API configuration
#[derive(Clone, ApiClient)]
#[api_client(
    base_url = "https://api.example.com",
    requests(GetUser, CreateUser)
)]
struct MyApiConfig {
    api_key: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create your config
    let config = MyApiConfig {
        api_key: "your-api-key".to_string(),
    };

    // Create the client
    let client = MyApiClient::new(config, ReqwestBlockingClient::new()?);

    // Use the generated methods - they return pre-configured builders
    let user = client.get_user()
        .id(123)
        .send()?;

    let new_user = client.create_user()
        .name("Alice".to_string())
        .send()?;

    Ok(())
}
```

The `ApiClient` macro generates:
- `MyApiClient` - Blocking client with methods for each request type
- `MyApiAsyncClient` - Async client for async request types
- Pre-configured builders with the base URL and HTTP client already set
- Methods named after your request structs (snake_case)
- Custom method names via `requests(CreateUser = "new_user")`

### Configuring Requests Automatically

Use the `ConfigureRequest` trait to automatically apply settings (like auth headers) to all requests:

```rust
use derive_rest_api::{ApiClient, ConfigureRequest, RequestBuilder, RequestModifier};
use serde::Serialize;

#[derive(RequestBuilder, Serialize)]
#[request_builder(method = "GET", path = "/users/{id}")]
struct GetUser {
    id: u64,
}

#[derive(Clone, ApiClient)]
#[api_client(
    base_url = "https://api.example.com",
    requests(GetUser)
)]
struct MyApiConfig {
    api_key: String,
    user_agent: String,
}

// Implement ConfigureRequest to modify all requests
impl ConfigureRequest for MyApiConfig {
    fn configure<M: RequestModifier>(&self, modifier: M) -> M {
        modifier
            .header("X-API-Key", &self.api_key)
            .header("User-Agent", &self.user_agent)
    }
}

// All requests automatically include the configured headers!
let config = MyApiConfig {
    api_key: "secret_key_123".to_string(),
    user_agent: "my-app/1.0".to_string(),
};

let client = MyApiClient::new().with_config(config);
client.get_user().id(123).send()?; // X-API-Key and User-Agent are auto-applied
```

**Note**: If your config struct has no fields (unit struct, empty struct, or empty tuple struct), `NoRequestConfiguration` is automatically implemented for you! For example:

```rust
#[derive(Clone, ApiClient)]
#[api_client(base_url = "...", requests(...))]
struct SimpleConfig;  // Automatically gets NoRequestConfiguration!
```

If your config has fields but doesn't need to modify requests, manually implement `NoRequestConfiguration`:

```rust
struct SimpleConfig {
    timeout: u64,
}

impl derive_rest_api::NoRequestConfiguration for SimpleConfig {}
```

## Error Handling

The library uses `thiserror` for type-safe error handling. All operations that can fail return a `Result<T, RequestError>`:

```rust
use derive_rest_api::{RequestBuilder, RequestError};

#[derive(RequestBuilder)]
#[request_builder(method = "GET", path = "/users/{id}")]
struct GetUser {
    id: u64,
}

fn example() -> Result<(), RequestError> {
    let request = GetUserBuilder::new()
        .id(123)
        .build()?;

    match request.build_url() {
        Ok(url) => println!("URL: {}", url),
        Err(RequestError::MissingPathParameter { param }) => {
            eprintln!("Missing parameter: {}", param);
        }
        Err(RequestError::QuerySerializationError { source }) => {
            eprintln!("Query error: {}", source);
        }
        Err(e) => {
            eprintln!("Other error: {}", e);
        }
    }

    Ok(())
}
```

Available error variants:
- `MissingField` - Required builder field not set
- `MissingPathParameter` - Path parameter not provided
- `QuerySerializationError` - Query string serialization failed
- `BodySerializationError` - JSON body serialization failed
- `ValidationError` - Field validation failed
- `MissingBaseUrl` - No base URL configured
- `UrlBuildError` - URL building failed
- `HttpError` - HTTP client error

## Attributes

### Struct-level Attributes

| Attribute | Description | Example |
|-----------|-------------|---------|
| `into` | Enable `Into<T>` for all setters | `#[request_builder(into)]` |
| `default` | Use `Default::default()` for all fields | `#[request_builder(default)]` |
| `method = "..."` | HTTP method (GET, POST, etc.) | `#[request_builder(method = "POST")]` |
| `path = "..."` | URL path template | `#[request_builder(path = "/users/{id}")]` |
| `response = Type` | Response type | `#[request_builder(response = User)]` |
| `query_config = "..."` | Custom query string config | `#[request_builder(query_config = "custom_config()")]` |

### Field-level Attributes

| Attribute | Description | Example |
|-----------|-------------|---------|
| `path` | Mark field as path parameter | `#[request_builder(path)]` |
| `query` | Include field in query string | `#[request_builder(query)]` |
| `query = "name"` | Include with custom name | `#[request_builder(query = "q")]` |
| `body` | Mark field as request body | `#[request_builder(body)]` |
| `body = "name"` | Body field with custom name | `#[request_builder(body = "userName")]` |
| `header` | Mark field as HTTP header (auto Title-Case) | `#[request_builder(header)]` |
| `header = "Name"` | Header with custom name | `#[request_builder(header = "X-API-Key")]` |
| `into` | Enable `Into<T>` for this field | `#[request_builder(into)]` |
| `default` | Use default value if not set | `#[request_builder(default)]` |
| `validate = "fn"` | Custom validation function | `#[request_builder(validate = "validate_email")]` |

### ApiClient Attributes

| Attribute | Description | Example |
|-----------|-------------|---------|
| `base_url = "..."` | Base URL for all requests | `#[api_client(base_url = "https://api.example.com")]` |
| `requests(...)` | Request types to include | `#[api_client(requests(GetUser, CreateUser))]` |
| Custom method name | Rename generated method | `requests(CreateUser = "new_user")` |

## Serde Integration

All `#[serde(...)]` attributes on fields marked with `query` or `body` are automatically preserved in the generated serialization code:

```rust
#[derive(RequestBuilder, Serialize)]
#[request_builder(method = "POST", path = "/users")]
struct CreateUser {
    #[request_builder(body)]
    #[serde(rename = "userName")]
    user_name: String,

    #[request_builder(body)]
    #[serde(skip_serializing_if = "Option::is_none")]
    email: Option<String>,
}
```

## Custom HTTP Clients

### Blocking HTTP Client

Implement the `HttpClient` trait for your own blocking HTTP client:

```rust
use derive_rest_api::HttpClient;
use std::collections::HashMap;

#[derive(Debug)]
struct MyError(String);

impl std::fmt::Display for MyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for MyError {}

#[derive(Clone)]
struct MyBlockingHttpClient {
    // Your client implementation
}

impl HttpClient for MyBlockingHttpClient {
    type Error = MyError;

    fn send(
        &self,
        method: &str,
        url: &str,
        headers: HashMap<String, String>,
        body: Option<Vec<u8>>,
    ) -> Result<Vec<u8>, Self::Error> {
        // Your blocking implementation
        Ok(vec![])
    }
}
```

### Async HTTP Client

Implement the `AsyncHttpClient` trait for your own async HTTP client:

```rust
use derive_rest_api::AsyncHttpClient;
use std::collections::HashMap;

#[derive(Debug)]
struct MyError(String);

impl std::fmt::Display for MyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for MyError {}

#[derive(Clone)]
struct MyAsyncHttpClient {
    // Your client implementation
}

impl AsyncHttpClient for MyAsyncHttpClient {
    type Error = MyError;

    async fn send_async(
        &self,
        method: &str,
        url: &str,
        headers: HashMap<String, String>,
        body: Option<Vec<u8>>,
    ) -> Result<Vec<u8>, Self::Error> {
        // Your async implementation
        Ok(vec![])
    }
}
```

**Note**: Error types must implement `std::error::Error + Send + Sync + 'static`.

## Features

This library provides optional feature flags for different HTTP client backends:

- `reqwest-blocking`: Enable reqwest blocking client support
- `reqwest-async`: Enable reqwest async client support
- `ureq-blocking`: Enable ureq blocking client support (lightweight alternative)

By default, no HTTP client is included, allowing you to choose only what you need.

## Examples

Run the examples with:

```bash
# Reqwest blocking client example
cargo run --example reqwest_blocking_example --features reqwest-blocking

# Reqwest async client example
cargo run --example reqwest_async_example --features reqwest-async

# Ureq blocking client example
cargo run --example ureq_blocking_example --features ureq-blocking
```

## License

[Choose your license]

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
