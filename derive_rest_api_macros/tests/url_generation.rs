use derive_rest_api_macros::RequestBuilder;
use serde::Serialize;

#[test]
fn test_simple_path() {
    #[derive(RequestBuilder)]
    #[request_builder(path = "/api/users/{id}")]
    struct GetUser {
        id: u64,
    }

    let user = GetUserBuilder::new()
        .id(123)
        .build()
        .unwrap();

    let url = user.build_url().unwrap();
    assert_eq!(url, "/api/users/123");
}

#[test]
fn test_multiple_path_params() {
    #[derive(RequestBuilder)]
    #[request_builder(path = "/api/users/{user_id}/posts/{post_id}")]
    struct GetPost {
        user_id: u64,
        post_id: u64,
    }

    let post = GetPostBuilder::new()
        .user_id(42)
        .post_id(7)
        .build()
        .unwrap();

    let url = post.build_url().unwrap();
    assert_eq!(url, "/api/users/42/posts/7");
}

#[test]
fn test_path_with_string_param() {
    #[derive(RequestBuilder)]
    #[request_builder(path = "/api/repos/{owner}/{repo}")]
    struct GetRepo {
        owner: String,
        repo: String,
    }

    let repo = GetRepoBuilder::new()
        .owner("rust-lang".to_string())
        .repo("rust".to_string())
        .build()
        .unwrap();

    let url = repo.build_url().unwrap();
    assert_eq!(url, "/api/repos/rust-lang/rust");
}

#[test]
fn test_path_with_extra_fields() {
    #[derive(RequestBuilder)]
    #[request_builder(path = "/api/users/{id}")]
    struct GetUser {
        id: u64,
        #[expect(unused)]
        include_posts: Option<bool>,
        #[expect(unused)]
        limit: Option<u32>,
    }

    let user = GetUserBuilder::new()
        .id(123)
        .include_posts(true)
        .limit(10)
        .build()
        .unwrap();

    let url = user.build_url().unwrap();
    // Only the path parameter should be substituted
    assert_eq!(url, "/api/users/123");
}

#[test]
fn test_no_path_attribute() {
    #[derive(RequestBuilder)]
    #[expect(unused)]
    struct CreateUser {
        name: String,
        email: String,
    }

    // Should compile - build_url just won't be generated
    let _builder = CreateUserBuilder::new()
        .name("Alice".to_string())
        .email("alice@example.com".to_string());

    // The build_url method shouldn't exist
    // (Can't test this directly without compile_fail test)
}

#[test]
fn test_path_param_with_into() {
    #[derive(RequestBuilder)]
    #[request_builder(path = "/api/users/{username}", into)]
    struct GetUser {
        username: String,
    }

    let user = GetUserBuilder::new()
        .username("alice")
        .build()
        .unwrap();

    let url = user.build_url().unwrap();
    assert_eq!(url, "/api/users/alice");
}

#[test]
fn test_path_with_optional_param() {
    #[derive(RequestBuilder)]
    #[request_builder(path = "/api/users/{id}")]
    struct GetUser {
        id: Option<u64>,
    }

    // With value set
    let user1 = GetUserBuilder::new()
        .id(123)
        .build()
        .unwrap();
    let url1 = user1.build_url().unwrap();
    assert_eq!(url1, "/api/users/123");

    // Without value - should error
    let user2 = GetUserBuilder::new()
        .build()
        .unwrap();
    let result = user2.build_url();
    assert!(result.is_err());
}

#[test]
fn test_query_params_simple() {
    #[derive(RequestBuilder, Serialize)]
    #[request_builder(path = "/api/users")]
    struct ListUsers {
        #[request_builder(query)]
        page: Option<u32>,
        #[request_builder(query)]
        limit: Option<u32>,
    }

    let users = ListUsersBuilder::new()
        .page(2)
        .limit(10)
        .build()
        .unwrap();

    let url = users.build_url().unwrap();
    assert!(url.starts_with("/api/users?"));
    assert!(url.contains("page=2"));
    assert!(url.contains("limit=10"));
}

#[test]
fn test_query_params_with_path_param() {
    #[derive(RequestBuilder, Serialize)]
    #[request_builder(path = "/api/users/{id}")]
    struct GetUser {
        id: u64,
        #[request_builder(query)]
        include_posts: Option<bool>,
    }

    let user = GetUserBuilder::new()
        .id(123)
        .include_posts(true)
        .build()
        .unwrap();

    let url = user.build_url().unwrap();
    assert_eq!(url, "/api/users/123?include_posts=true");
}

#[test]
fn test_query_params_none() {
    #[derive(RequestBuilder, Serialize)]
    #[request_builder(path = "/api/users")]
    struct ListUsers {
        #[request_builder(query)]
        page: Option<u32>,
    }

    let users = ListUsersBuilder::new()
        .build()
        .unwrap();

    let url = users.build_url().unwrap();
    // When all query params are None, no query string should be appended
    assert_eq!(url, "/api/users");
}

#[test]
fn test_query_params_mixed() {
    #[derive(RequestBuilder, Serialize)]
    #[request_builder(path = "/api/search")]
    struct Search {
        #[request_builder(query)]
        q: Option<String>,
        #[request_builder(query)]
        limit: Option<u32>,
        #[request_builder(query)]
        offset: Option<u32>,
    }

    let search = SearchBuilder::new()
        .q("rust".to_string())
        .limit(20)
        .build()
        .unwrap();

    let url = search.build_url().unwrap();
    assert!(url.starts_with("/api/search?"));
    assert!(url.contains("q=rust"));
    assert!(url.contains("limit=20"));
    assert!(!url.contains("offset"));
}
