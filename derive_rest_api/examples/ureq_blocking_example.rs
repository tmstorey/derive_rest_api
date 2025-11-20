//! Example demonstrating how to use the `derive_rest_api` macro with ureq's blocking client.
//!
//! Run with: `cargo run --example ureq_blocking_example --features ureq-blocking`

#[cfg(feature = "ureq-blocking")]
use derive_rest_api::{RequestBuilder, UreqBlockingClient};
#[cfg(feature = "ureq-blocking")]
use serde::Serialize;

#[cfg(feature = "ureq-blocking")]
#[derive(RequestBuilder, Serialize, Debug)]
#[request_builder(method = "GET", path = "/users/{id}")]
struct GetUser {
    /// User ID
    id: u64,

    /// Include user's posts in the response
    #[request_builder(query)]
    #[serde(rename = "includePosts")]
    include_posts: Option<bool>,
}

#[cfg(feature = "ureq-blocking")]
#[derive(RequestBuilder, Serialize, Debug)]
#[request_builder(method = "POST", path = "/posts")]
struct CreatePost {
    /// Post title
    #[request_builder(body)]
    title: String,

    /// Post body/content
    #[request_builder(body)]
    body: String,

    /// User ID who creates the post
    #[request_builder(body)]
    #[serde(rename = "userId")]
    user_id: u64,

    /// Authorization header
    #[request_builder(header = "Authorization")]
    authorization: Option<String>,
}

#[cfg(feature = "ureq-blocking")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Ureq Blocking Client Example ===\n");

    // Create a ureq blocking client wrapper
    let client = UreqBlockingClient::new();

    // Example 1: GET request with path parameter and query string
    println!("1. Fetching user with ID 1...");
    let get_user_request = GetUserBuilder::new()
        .id(1)
        .include_posts(true)
        .build()?;

    println!("   URL: {}", get_user_request.build_url()?);

    match get_user_request.send_with_client(&client, "https://jsonplaceholder.typicode.com") {
        Ok(response) => {
            let user: serde_json::Value = serde_json::from_slice(&response)?;
            println!("   Response: {}", serde_json::to_string_pretty(&user)?);
        }
        Err(e) => println!("   Error: {}", e),
    }

    println!();

    // Example 2: POST request with body and headers
    println!("2. Creating a new post...");
    let create_post_request = CreatePostBuilder::new()
        .title("Hello from derive_rest_api with ureq!".to_string())
        .body("This post was created using the derive_rest_api macro with ureq.".to_string())
        .user_id(1)
        .authorization("Bearer fake-token-12345".to_string())
        .build()?;

    println!("   URL: {}", create_post_request.build_url()?);
    println!("   Headers: {:?}", create_post_request.build_headers());

    match create_post_request.send_with_client(&client, "https://jsonplaceholder.typicode.com") {
        Ok(response) => {
            let post: serde_json::Value = serde_json::from_slice(&response)?;
            println!("   Response: {}", serde_json::to_string_pretty(&post)?);
        }
        Err(e) => println!("   Error: {}", e),
    }

    println!();

    // Example 3: Using a custom ureq agent with configuration
    println!("3. Using custom ureq agent with timeout...");
    let custom_agent = ureq::AgentBuilder::new()
        .timeout(std::time::Duration::from_secs(10))
        .user_agent("derive-rest-api-ureq-example/1.0")
        .build();

    let custom_client = UreqBlockingClient::with_agent(custom_agent);

    let request = GetUserBuilder::new()
        .id(2)
        .build()?;

    match request.send_with_client(&custom_client, "https://jsonplaceholder.typicode.com") {
        Ok(response) => {
            let user: serde_json::Value = serde_json::from_slice(&response)?;
            println!("   User name: {}", user["name"]);
        }
        Err(e) => println!("   Error: {}", e),
    }

    Ok(())
}

#[cfg(not(feature = "ureq-blocking"))]
fn main() {
    println!("This example requires the 'ureq-blocking' feature.");
    println!("Run with: cargo run --example ureq_blocking_example --features ureq-blocking");
}
