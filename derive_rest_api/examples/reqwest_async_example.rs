//! Example demonstrating how to use the `derive_rest_api` macro with reqwest's async client.
//!
//! Run with: `cargo run --example reqwest_async_example --features reqwest-async`

#[cfg(feature = "reqwest-async")]
use derive_rest_api::{AsyncHttpClient, RequestBuilder, ReqwestAsyncClient};
#[cfg(feature = "reqwest-async")]
use serde::Serialize;

#[cfg(feature = "reqwest-async")]
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

#[cfg(feature = "reqwest-async")]
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

#[cfg(feature = "reqwest-async")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Reqwest Async Client Example ===\n");

    // Create a reqwest async client wrapper
    let client = ReqwestAsyncClient::new()?;

    // Example 1: GET request with path parameter and query string
    println!("1. Fetching user with ID 1...");
    let get_user_request = GetUserBuilder::new()
        .id(1)
        .include_posts(true)
        .build()?;

    let url = get_user_request.build_url()?;
    println!("   URL: {}", url);

    match client.send_async(
        "GET",
        &format!("https://jsonplaceholder.typicode.com{}", url),
        get_user_request.build_headers(),
        get_user_request.build_body()?,
    ).await {
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
        .title("Hello from derive_rest_api async!".to_string())
        .body("This post was created using the async derive_rest_api macro.".to_string())
        .user_id(1)
        .authorization("Bearer fake-token-12345".to_string())
        .build()?;

    let url = create_post_request.build_url()?;
    println!("   URL: {}", url);
    println!("   Headers: {:?}", create_post_request.build_headers());

    match client.send_async(
        "POST",
        &format!("https://jsonplaceholder.typicode.com{}", url),
        create_post_request.build_headers(),
        create_post_request.build_body()?,
    ).await {
        Ok(response) => {
            let post: serde_json::Value = serde_json::from_slice(&response)?;
            println!("   Response: {}", serde_json::to_string_pretty(&post)?);
        }
        Err(e) => println!("   Error: {}", e),
    }

    println!();

    // Example 3: Using a custom reqwest client with configuration
    println!("3. Using custom reqwest client with timeout...");
    let custom_reqwest_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .user_agent("derive-rest-api-async-example/1.0")
        .build()?;

    let custom_client = ReqwestAsyncClient::with_client(custom_reqwest_client);

    let request = GetUserBuilder::new()
        .id(2)
        .build()?;

    let url = request.build_url()?;

    match custom_client.send_async(
        "GET",
        &format!("https://jsonplaceholder.typicode.com{}", url),
        request.build_headers(),
        request.build_body()?,
    ).await {
        Ok(response) => {
            let user: serde_json::Value = serde_json::from_slice(&response)?;
            println!("   User name: {}", user["name"]);
        }
        Err(e) => println!("   Error: {}", e),
    }

    println!();

    // Example 4: Concurrent requests using tokio
    println!("4. Fetching multiple users concurrently...");

    let user_ids = vec![1, 2, 3, 4, 5];
    let mut tasks = vec![];

    for id in user_ids {
        let client_clone = &client;
        let task = async move {
            let request = GetUserBuilder::new()
                .id(id)
                .build()
                .unwrap();

            let url = request.build_url().unwrap();

            client_clone.send_async(
                "GET",
                &format!("https://jsonplaceholder.typicode.com{}", url),
                request.build_headers(),
                request.build_body().unwrap(),
            ).await
        };

        tasks.push(task);
    }

    let results = futures::future::join_all(tasks).await;

    for (idx, result) in results.iter().enumerate() {
        match result {
            Ok(response) => {
                let user: serde_json::Value = serde_json::from_slice(response)?;
                println!("   User {}: {}", idx + 1, user["name"]);
            }
            Err(e) => println!("   User {}: Error - {}", idx + 1, e),
        }
    }

    Ok(())
}

#[cfg(not(feature = "reqwest-async"))]
fn main() {
    println!("This example requires the 'reqwest-async' feature.");
    println!("Run with: cargo run --example reqwest_async_example --features reqwest-async");
}
