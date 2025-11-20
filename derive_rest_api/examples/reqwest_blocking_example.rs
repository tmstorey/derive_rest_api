//! Example demonstrating how to use the `derive_rest_api` macro with reqwest's blocking client.
//!
//! Run with: `cargo run --example reqwest_blocking_example --features reqwest-blocking`

#[cfg(feature = "reqwest-blocking")]
mod example {
    use derive_rest_api::{ApiClient, HttpClient, RequestBuilder, ReqwestBlockingClient};
    use serde::Serialize;
    use serde_json::to_string_pretty;

    #[derive(RequestBuilder, Serialize, Debug)]
    #[request_builder(method = "GET", path = "/users/{id}", response = serde_json::Value)]
    struct GetUser {
        /// User ID
        id: u64,

        /// Include user's posts in the response
        #[request_builder(query)]
        #[serde(rename = "includePosts")]
        include_posts: Option<bool>,
    }

    #[derive(RequestBuilder, Serialize, Debug)]
    #[request_builder(method = "POST", path = "/posts", response = serde_json::Value)]
    struct CreatePost {
        /// Post title
        #[request_builder(body, into)]
        title: String,

        /// Post body/content
        #[request_builder(body, into)]
        body: String,

        /// User ID who creates the post
        #[request_builder(body)]
        #[serde(rename = "userId")]
        user_id: u64,

        /// Authorization header
        #[request_builder(header = "Authorization", into)]
        authorization: Option<String>,
    }

    #[derive(Clone, ApiClient)]
    #[api_client(
        base_url = "https://jsonplaceholder.typicode.com",
        requests(GetUser, CreatePost = "new_post")
    )]
    struct JsonPlaceholder;

    pub fn main() -> Result<(), Box<dyn std::error::Error>> {
        println!("=== Reqwest Blocking Client Example ===\n");

        // Create a reqwest client wrapper
        let client = JsonPlaceholderClient::<ReqwestBlockingClient>::new();

        // Example 1: GET request with path parameter and query string
        println!("1. Fetching user with ID 1...");
        let get_user_response = client.get_user()
            .id(1)
            .include_posts(true)
            .send()?;

        println!("   Response: {}", to_string_pretty(&get_user_response).unwrap());
        println!();

        // Example 2: POST request with body and headers
        println!("2. Creating a new post...");
        let create_post_response = client.new_post()
            .title("Hello from derive_rest_api!")
            .body("This post was created using the derive_rest_api macro.")
            .user_id(1u64)
            .authorization("Bearer fake-token-12345")
            .send()?;

        println!("   Response: {}", to_string_pretty(&create_post_response).unwrap());
        println!();

        // Example 3: Using a custom reqwest client with configuration
        println!("3. Using custom reqwest client with timeout...");
        let custom_reqwest_client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .user_agent("derive-rest-api-example/1.0")
            .build()?;

        let custom_client = JsonPlaceholderClient::<ReqwestBlockingClient>::new()
            .with_http_client(custom_reqwest_client);

        let custom_request = custom_client.get_user()
            .id(2)
            .send()?;

        println!("   Response: {}", to_string_pretty(&custom_request).unwrap());

        Ok(())
    }
}

#[cfg(feature = "reqwest-blocking")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    example::main()
}

#[cfg(not(feature = "reqwest-blocking"))]
fn main() {
    println!("This example requires the 'reqwest-blocking' feature.");
    println!("Run with: cargo run --example reqwest_blocking_example --features reqwest-blocking");
}
