use derive_rest_api_macros::RequestBuilder;

#[derive(RequestBuilder, Debug, PartialEq)]
struct GetUser {
    id: u64,
    include_posts: bool,
}

#[test]
fn test_builder_struct_exists() {
    // This test will compile if the builder struct is generated
    let _builder: GetUserBuilder<(), ()> = GetUserBuilder {
        id: Some(123),
        include_posts: Some(true),
        __http_client: None,
        __async_http_client: None,
        __base_url: None,
    };
}

#[test]
fn test_builder_with_optional_field() {
    #[derive(RequestBuilder)]
    #[expect(unused)]
    struct SearchUsers {
        query: String,
        limit: Option<u32>,
    }

    // query should be Option<String>, limit should be Option<u32> (not double-wrapped)
    let _builder: SearchUsersBuilder<(), ()> = SearchUsersBuilder {
        query: Some("test".to_string()),
        limit: Some(10),
        __http_client: None,
        __async_http_client: None,
        __base_url: None,
    };
}

#[test]
fn test_optional_field_setter() {
    #[derive(RequestBuilder)]
    #[expect(unused)]
    struct CreatePost {
        title: String,
        tags: Option<Vec<String>>,
    }

    // Setter for optional field should take inner type (Vec<String>), not Option<Vec<String>>
    let builder = CreatePostBuilder::new()
        .title("My Post".to_string())
        .tags(vec!["rust".to_string(), "programming".to_string()]);

    assert_eq!(builder.title, Some("My Post".to_string()));
    assert_eq!(builder.tags, Some(vec!["rust".to_string(), "programming".to_string()]));
}

#[test]
fn test_optional_field_not_set() {
    #[derive(RequestBuilder)]
    #[expect(unused)]
    struct UpdateUser {
        id: u64,
        email: Option<String>,
    }

    let builder = UpdateUserBuilder::new()
        .id(123);

    assert_eq!(builder.id, Some(123));
    assert_eq!(builder.email, None); // Optional field not set
}

#[test]
fn test_builder_new_method() {
    let builder = GetUserBuilder::new();

    // All fields should be None initially
    assert!(builder.id.is_none());
    assert!(builder.include_posts.is_none());
}

#[test]
fn test_builder_setter_methods() {
    let builder = GetUserBuilder::new()
        .id(123)
        .include_posts(true);

    // Fields should be set
    assert_eq!(builder.id, Some(123));
    assert_eq!(builder.include_posts, Some(true));
}

#[test]
fn test_builder_partial_setters() {
    let builder = GetUserBuilder::new()
        .id(456);

    // Only id should be set
    assert_eq!(builder.id, Some(456));
    assert!(builder.include_posts.is_none());
}

#[test]
fn test_build_success() {
    let result = GetUserBuilder::new()
        .id(123)
        .include_posts(true)
        .build();

    assert!(result.is_ok());
    let user = result.unwrap();
    assert_eq!(user.id, 123);
    assert_eq!(user.include_posts, true);
}

#[test]
fn test_build_missing_required_field() {
    // Missing include_posts (required field)
    let result = GetUserBuilder::new()
        .id(123)
        .build();

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("include_posts"));
}

#[test]
fn test_build_with_optional_fields() {
    #[derive(RequestBuilder)]
    struct CreateUser {
        username: String,
        email: Option<String>,
    }

    // Build with optional field set
    let result1 = CreateUserBuilder::new()
        .username("alice".to_string())
        .email("alice@example.com".to_string())
        .build();

    assert!(result1.is_ok());
    let user1 = result1.unwrap();
    assert_eq!(user1.username, "alice");
    assert_eq!(user1.email, Some("alice@example.com".to_string()));

    // Build without optional field
    let result2 = CreateUserBuilder::new()
        .username("bob".to_string())
        .build();

    assert!(result2.is_ok());
    let user2 = result2.unwrap();
    assert_eq!(user2.username, "bob");
    assert_eq!(user2.email, None);
}

#[test]
fn test_struct_level_into() {
    #[derive(RequestBuilder, Debug, PartialEq)]
    #[request_builder(into)]
    struct CreatePost {
        title: String,
        content: String,
    }

    // Should accept &str with Into
    let post = CreatePostBuilder::new()
        .title("Hello")
        .content("World")
        .build()
        .unwrap();

    assert_eq!(post.title, "Hello");
    assert_eq!(post.content, "World");
}

#[test]
fn test_field_level_into() {
    #[derive(RequestBuilder, Debug, PartialEq)]
    struct UpdatePost {
        id: u64,
        #[request_builder(into)]
        title: String,
    }

    // id should NOT use Into, title should
    let post = UpdatePostBuilder::new()
        .id(42)
        .title("New Title")  // &str converts to String
        .build()
        .unwrap();

    assert_eq!(post.id, 42);
    assert_eq!(post.title, "New Title");
}

#[test]
fn test_into_with_optional_field() {
    #[derive(RequestBuilder, Debug, PartialEq)]
    #[request_builder(into)]
    struct SearchPosts {
        query: String,
        author: Option<String>,
    }

    let search = SearchPostsBuilder::new()
        .query("rust")
        .author("alice")
        .build()
        .unwrap();

    assert_eq!(search.query, "rust");
    assert_eq!(search.author, Some("alice".to_string()));
}

#[test]
fn test_builder_client_fields() {
    // Test that builder has client fields and can be constructed
    let builder = GetUserBuilder::new();

    // Verify client fields are None initially
    assert!(builder.__http_client.is_none());
    assert!(builder.__async_http_client.is_none());
    assert!(builder.__base_url.is_none());

    // Verify we can still build normally
    let result = builder
        .id(123)
        .include_posts(true)
        .build();

    assert!(result.is_ok());
}
