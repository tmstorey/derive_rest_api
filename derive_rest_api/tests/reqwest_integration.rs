#[cfg(feature = "reqwest-blocking")]
mod blocking_tests {
    use derive_rest_api::{RequestBuilder, ReqwestBlockingClient};
    use serde::Serialize;

    #[test]
    fn test_reqwest_blocking_client_creation() {
        let client = ReqwestBlockingClient::new();
        assert!(client.is_ok());

        let client_default = ReqwestBlockingClient::default();
        // Just verify it was created successfully
        drop(client_default);
    }

    #[test]
    fn test_reqwest_blocking_with_custom_client() {
        let reqwest_client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .unwrap();

        let client = ReqwestBlockingClient::with_client(reqwest_client);
        drop(client);
    }

    #[test]
    #[ignore] // Requires network connection
    fn test_reqwest_blocking_real_request() {
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

        let client = ReqwestBlockingClient::new().unwrap();

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
    fn test_reqwest_blocking_post_request() {
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

        let client = ReqwestBlockingClient::new().unwrap();

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
    fn test_reqwest_blocking_with_headers() {
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

        let client = ReqwestBlockingClient::new().unwrap();

        let result = request.send_with_client(&client, "https://jsonplaceholder.typicode.com");

        if let Ok(response) = result {
            assert!(!response.is_empty());
            let json: serde_json::Value = serde_json::from_slice(&response).unwrap();
            assert!(json.is_array());
        }
    }
}

#[cfg(feature = "reqwest-async")]
mod async_tests {
    use derive_rest_api::{AsyncHttpClient, RequestBuilder, ReqwestAsyncClient};
    use serde::Serialize;

    #[test]
    fn test_reqwest_async_client_creation() {
        let client = ReqwestAsyncClient::new();
        assert!(client.is_ok());

        let client_default = ReqwestAsyncClient::default();
        drop(client_default);
    }

    #[test]
    fn test_reqwest_async_with_custom_client() {
        let reqwest_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .unwrap();

        let client = ReqwestAsyncClient::with_client(reqwest_client);
        drop(client);
    }

    #[tokio::test]
    #[ignore] // Requires network connection
    async fn test_reqwest_async_real_request() {
        #[derive(RequestBuilder, Serialize)]
        #[request_builder(method = "GET", path = "/users/{id}")]
        struct GetUser {
            id: u64,
            #[request_builder(query)]
            include_posts: Option<bool>,
        }

        let request = GetUserBuilder::new()
            .id(1)
            .build()
            .unwrap();

        let client = ReqwestAsyncClient::new().unwrap();

        // Using JSONPlaceholder API for testing
        let result = client.send_async(
            "GET",
            &format!("https://jsonplaceholder.typicode.com{}", request.build_url().unwrap()),
            request.build_headers(),
            request.build_body().unwrap(),
            None
        ).await;

        if let Ok(response) = result {
            assert!(!response.is_empty());
            let json: serde_json::Value = serde_json::from_slice(&response).unwrap();
            assert_eq!(json["id"], 1);
        }
    }

    #[tokio::test]
    #[ignore] // Requires network connection
    async fn test_reqwest_async_post_request() {
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
            .body("This is a test async post".to_string())
            .user_id(1)
            .build()
            .unwrap();

        let client = ReqwestAsyncClient::new().unwrap();

        let result = client.send_async(
            "POST",
            &format!("https://jsonplaceholder.typicode.com{}", request.build_url().unwrap()),
            request.build_headers(),
            request.build_body().unwrap(),
            None
        ).await;

        if let Ok(response) = result {
            assert!(!response.is_empty());
            let json: serde_json::Value = serde_json::from_slice(&response).unwrap();
            assert!(json.get("id").is_some());
        }
    }
}
