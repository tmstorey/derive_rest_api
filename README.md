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

struct MyBlockingHttpClient {
    // Your client implementation
}

impl HttpClient for MyBlockingHttpClient {
    type Error = String;

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

struct MyAsyncHttpClient {
    // Your client implementation
}

impl AsyncHttpClient for MyAsyncHttpClient {
    type Error = String;

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
