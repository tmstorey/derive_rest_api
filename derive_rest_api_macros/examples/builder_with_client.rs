/// Example demonstrating how to use the builder with embedded HTTP clients.
///
/// This shows the new capability to attach clients directly to the builder,
/// which simplifies the API for users who want to configure a client once
/// and reuse it across multiple requests.

use derive_rest_api_macros::RequestBuilder;
use serde::Serialize;

// Note: This example demonstrates the API but doesn't actually make HTTP requests
// as it would require a feature flag to include an actual HTTP client implementation

#[derive(RequestBuilder, Serialize)]
#[request_builder(method = "GET", path = "/users/{id}")]
struct GetUser {
    id: u64,
    #[request_builder(query)]
    include_posts: Option<bool>,
}

fn main() {
    println!("=== Builder with Client Fields Example ===\n");

    // Example 1: Basic usage without client (traditional approach)
    println!("1. Traditional builder usage:");
    let request = GetUserBuilder::new()
        .id(123)
        .include_posts(true)
        .build()
        .unwrap();

    println!("   User ID: {}", request.id);
    println!("   Include posts: {:?}", request.include_posts);
    println!();

    // Example 2: Builder with client attached (new capability)
    // This demonstrates the new generic builder structure
    println!("2. Builder can now store HTTP clients and base URL:");
    println!("   - Builder type: GetUserBuilder<C, A>");
    println!("   - C: Optional HttpClient implementation");
    println!("   - A: Optional AsyncHttpClient implementation");
    println!();
    println!("   Usage:");
    println!("   let builder = GetUserBuilder::new()");
    println!("       .http_client(MyClient::new())  // Sets the blocking client");
    println!("       .base_url(\"https://api.example.com\")  // Sets the base URL");
    println!("       .id(123)");
    println!("       .send()  // Sends the request");
    println!();
    println!("   The builder now has three extra fields:");
    println!("   - __http_client: Option<C>");
    println!("   - __async_http_client: Option<A>");
    println!("   - __base_url: Option<String>");
    println!();

    // Example 3: The builder maintains type safety
    println!("3. Type safety:");
    println!("   - GetUserBuilder::new() returns GetUserBuilder<(), ()>");
    println!("   - .http_client(c) returns GetUserBuilder<C, ()>");
    println!("   - .async_http_client(a) returns GetUserBuilder<(), A>");
    println!("   - Both can be set: GetUserBuilder<C, A>");
    println!();

    println!("This feature enables:");
    println!("  - .send() methods that automatically use the embedded client and base URL");
    println!("  - .send_async() methods for async clients");
    println!("  - Builder reuse with a pre-configured client and base URL");
}
