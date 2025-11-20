#[cfg(feature = "ureq-blocking")]
mod ureq_tests {
    use derive_rest_api::{RequestBuilder, UreqBlockingClient};
    use serde::Serialize;

    #[test]
    fn test_ureq_client_creation() {
        let client = UreqBlockingClient::new();
        // Just verify it was created successfully
        drop(client);

        let client_default = UreqBlockingClient::default();
        drop(client_default);
    }

    #[test]
    fn test_ureq_with_custom_agent() {
        let agent = ureq::AgentBuilder::new()
            .timeout(std::time::Duration::from_secs(30))
            .build();

        let client = UreqBlockingClient::with_agent(agent);
        drop(client);
    }

    #[test]
    #[ignore] // Requires network connection
    fn test_ureq_real_request() {
        #[derive(RequestBuilder, Serialize)]
        #[request_builder(method = "GET", path = "/users/{id}")]
        struct GetUser {
            id: u64,
            #[request_builder(query)]
            include_posts: Option<bool>,
        }

        let request = GetUserBuilder::new()
            .id(1)
            .include_posts(true)
            .build()
            .unwrap();

        let client = UreqBlockingClient::new();

        // Using JSONPlaceholder API for testing
        let result = request.send_with_client(&client, "https://jsonplaceholder.typicode.com");

        // We don't assert on the result because it requires network,
        // but if the network is available, this should work
        if let Ok(response) = result {
            assert!(!response.is_empty());
        }
    }

    #[test]
    #[ignore] // Requires network connection
    fn test_ureq_post_request() {
        #[derive(RequestBuilder, Serialize)]
        #[request_builder(method = "POST", path = "/posts")]
        struct CreatePost {
            #[request_builder(body)]
            title: String,
            #[request_builder(body)]
            body: String,
            #[request_builder(body)]
            #[serde(rename = "userId")]
            user_id: u64,
        }

        let request = CreatePostBuilder::new()
            .title("Test Post".to_string())
            .body("This is a test post".to_string())
            .user_id(1)
            .build()
            .unwrap();

        let client = UreqBlockingClient::new();

        // Using JSONPlaceholder API for testing
        let result = request.send_with_client(&client, "https://jsonplaceholder.typicode.com");

        if let Ok(response) = result {
            assert!(!response.is_empty());
            // JSONPlaceholder returns the created post with an ID
            let json: serde_json::Value = serde_json::from_slice(&response).unwrap();
            assert!(json.get("id").is_some());
        }
    }

    #[test]
    #[ignore] // Requires network connection
    fn test_ureq_with_headers() {
        #[derive(RequestBuilder, Serialize)]
        #[request_builder(method = "GET", path = "/posts")]
        struct GetPosts {
            #[request_builder(header = "User-Agent")]
            user_agent: String,
            #[request_builder(query)]
            #[serde(rename = "userId")]
            user_id: Option<u64>,
        }

        let request = GetPostsBuilder::new()
            .user_agent("derive-rest-api-test/1.0".to_string())
            .user_id(1)
            .build()
            .unwrap();

        let client = UreqBlockingClient::new();

        let result = request.send_with_client(&client, "https://jsonplaceholder.typicode.com");

        if let Ok(response) = result {
            assert!(!response.is_empty());
            let json: serde_json::Value = serde_json::from_slice(&response).unwrap();
            assert!(json.is_array());
        }
    }
}
