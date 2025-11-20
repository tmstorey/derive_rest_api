//! Example demonstrating how to use the `derive_rest_api` macro with reqwest's async client.
//!
//! Run with: `cargo run --example reqwest_async_example --features reqwest-async`

#[cfg(feature = "reqwest-async")]
mod example {
    use derive_rest_api::{ApiClient, RequestBuilder};
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

    pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
        println!("=== Reqwest Async Client Example ===\n");

        // Create a reqwest async client wrapper using the default client (auto-selected based on features)
        let client = JsonPlaceholderAsyncClient::new();

        // Example 1: GET request with path parameter and query string
        println!("1. Fetching user with ID 1...");
        let get_user_response = client.get_user()
            .id(1)
            .include_posts(true)
            .send_async()
            .await?;

        println!("   Response: {}", to_string_pretty(&get_user_response).unwrap());
        println!();

        // Example 2: POST request with body and headers
        println!("2. Creating a new post...");
        let create_post_response = client.new_post()
            .title("Hello from derive_rest_api async!")
            .body("This post was created using the async derive_rest_api macro.")
            .user_id(1)
            .authorization("Bearer fake-token-12345")
            .send_async()
            .await?;

        println!("   Response: {}", to_string_pretty(&create_post_response).unwrap());
        println!();

        // Example 3: Using a custom reqwest client with configuration
        println!("3. Using custom reqwest client with timeout...");
        let custom_reqwest_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .user_agent("derive-rest-api-async-example/1.0")
            .build()?;

        let custom_client = JsonPlaceholderAsyncClient::new()
            .with_http_client(custom_reqwest_client);

        let custom_request = custom_client.get_user()
            .id(2)
            .send_async()
            .await?;

        println!("   Response: {}", to_string_pretty(&custom_request).unwrap());
        println!();

        // Example 4: Concurrent requests using tokio
        println!("4. Fetching multiple users concurrently...");

        let user_ids = vec![1, 2, 3, 4, 5];
        let mut tasks = vec![];

        for id in user_ids {
            let client_clone = client.clone();
            let task = async move {
                client_clone.get_user()
                    .id(id)
                    .send_async()
                    .await
            };

            tasks.push(task);
        }

        let results = futures::future::join_all(tasks).await;

        for (idx, result) in results.iter().enumerate() {
            match result {
                Ok(response) => {
                    println!("   User {}: {}", idx + 1, response["name"]);
                }
                Err(e) => println!("   User {}: Error - {}", idx + 1, e),
            }
        }

        Ok(())
    }
}

#[cfg(feature = "reqwest-async")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    example::main().await
}


#[cfg(not(feature = "reqwest-async"))]
fn main() {
    println!("This example requires the 'reqwest-async' feature.");
    println!("Run with: cargo run --example reqwest_async_example --features reqwest-async");
}
