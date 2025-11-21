//! # JSON Placeholder API Client Example
//!
//! This crate demonstrates how to use `derive_rest_api` to create a type-safe
//! REST API client for the JSON Placeholder API (https://jsonplaceholder.typicode.com).
//!
//! ## Features Demonstrated
//!
//! - Path parameters: `/posts/{id}`, `/users/{id}`
//! - Query parameters: Filtering posts by userId
//! - Request bodies: Creating and updating posts
//! - Multiple HTTP methods: GET, POST, PUT, PATCH, DELETE
//! - Nested resources: `/posts/{postId}/comments`
//! - ApiClient pattern: High-level client wrapping all endpoints
//!
//! ## Example Usage
//!
//! ```rust
//! use json_placeholder::JsonPlaceholderClient;
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let client = JsonPlaceholderClient::new();
//!
//!     // List all posts
//!     let posts = client.list_posts().send()?;
//!
//!     // Get a specific post
//!     let post = client.get_post().id(1).send()?;
//!
//!     // Create a new post using the builder pattern
//!     let new_post = client.create_post()
//!         .title("Hello World".to_string())
//!         .body("This is my first post!".to_string())
//!         .user_id(1)
//!         .send()?;
//!
//!     Ok(())
//! }
//! ```

use derive_rest_api::{ApiClient, RequestBuilder};
use serde::{Deserialize, Serialize};

// ============================================================================
// Data Models
// ============================================================================

/// Represents a blog post from JSON Placeholder
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Post {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<u32>,
    #[serde(rename = "userId")]
    pub user_id: u32,
    pub title: String,
    pub body: String,
}

/// Represents a user from JSON Placeholder
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: u32,
    pub name: String,
    pub username: String,
    pub email: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<Address>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub website: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub company: Option<Company>,
}

/// User address information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Address {
    pub street: String,
    pub suite: String,
    pub city: String,
    pub zipcode: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub geo: Option<Geo>,
}

/// Geographic coordinates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Geo {
    pub lat: String,
    pub lng: String,
}

/// Company information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Company {
    pub name: String,
    #[serde(rename = "catchPhrase")]
    pub catch_phrase: String,
    pub bs: String,
}

/// Represents a comment on a post
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Comment {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<u32>,
    #[serde(rename = "postId")]
    pub post_id: u32,
    pub name: String,
    pub email: String,
    pub body: String,
}

/// Represents a result object containing an id
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResultId {
    pub id: u32,
}

// ============================================================================
// Request Structs - Posts
// ============================================================================

/// List all posts, with optional filtering by userId
///
/// # Example
/// ```rust,ignore
/// // Get all posts
/// let all_posts = client.list_posts().send()?;
///
/// // Get posts for a specific user
/// let user_posts = client.list_posts().user_id(1).send()?;
/// ```
#[derive(RequestBuilder, Serialize)]
#[request_builder(
    method = "GET",
    path = "/posts",
    response = Vec<Post>
)]
pub struct ListPosts {
    /// Filter posts by user ID
    #[request_builder(query)]
    #[serde(rename = "userId")]
    pub user_id: Option<u32>,
}

/// Get a single post by ID
///
/// # Example
/// ```rust,ignore
/// let post = client.get_post().id(1).send()?;
/// ```
#[derive(RequestBuilder, Serialize)]
#[request_builder(
    method = "GET",
    path = "/posts/{id}",
    response = Post
)]
pub struct GetPost {
    /// The post ID
    pub id: u32,
}

/// Create a new post
///
/// # Example
/// ```rust,ignore
/// let new_post = client.create_post()
///     .title("Hello".to_string())
///     .body("World".to_string())
///     .user_id(1)
///     .send()?;
/// ```
#[derive(RequestBuilder, Serialize)]
#[request_builder(
    method = "POST",
    path = "/posts",
    response = ResultId
)]
pub struct CreatePost {
    /// The post title
    #[request_builder(body)]
    pub title: String,

    /// The post body/content
    #[request_builder(body)]
    pub body: String,

    /// The ID of the user creating the post
    #[request_builder(body)]
    #[serde(rename = "userId")]
    pub user_id: u32,
}

/// Update an existing post (PUT - full replacement)
///
/// # Example
/// ```rust,ignore
/// let updated = client.update_post()
///     .id(1)
///     .title("Updated".to_string())
///     .body("New content".to_string())
///     .user_id(1)
///     .send()?;
/// ```
#[derive(RequestBuilder, Serialize)]
#[request_builder(
    method = "PUT",
    path = "/posts/{id}",
    response = ResultId
)]
pub struct UpdatePost {
    /// The post ID to update
    pub id: u32,

    /// The post title
    #[request_builder(body)]
    pub title: String,

    /// The post body/content
    #[request_builder(body)]
    pub body: String,

    /// The ID of the user
    #[request_builder(body)]
    #[serde(rename = "userId")]
    pub user_id: u32,
}

/// Partially update a post (PATCH)
///
/// # Example
/// ```rust,ignore
/// // Only update the title
/// let patched = client.patch_post()
///     .id(1)
///     .title("New Title".to_string())
///     .send()?;
/// ```
#[derive(RequestBuilder, Serialize)]
#[request_builder(
    method = "PATCH",
    path = "/posts/{id}",
    response = Post
)]
pub struct PatchPost {
    /// The post ID to patch
    pub id: u32,

    /// The post title (optional)
    #[request_builder(body)]
    pub title: Option<String>,

    /// The post body/content (optional)
    #[request_builder(body)]
    pub body: Option<String>,

    /// The ID of the user (optional)
    #[request_builder(body)]
    #[serde(rename = "userId")]
    pub user_id: Option<u32>,
}

/// Delete a post
///
/// # Example
/// ```rust,ignore
/// client.delete_post().id(1).send()?;
/// ```
#[derive(RequestBuilder, Serialize)]
#[request_builder(
    method = "DELETE",
    path = "/posts/{id}",
)]
pub struct DeletePost {
    /// The post ID to delete
    pub id: u32,
}

// ============================================================================
// Request Structs - Users
// ============================================================================

/// List all users
///
/// # Example
/// ```rust,ignore
/// let users = client.list_users().send()?;
/// ```
#[derive(RequestBuilder)]
#[request_builder(
    method = "GET",
    path = "/users",
    response = Vec<User>
)]
pub struct ListUsers;

/// Get a single user by ID
///
/// # Example
/// ```rust,ignore
/// let user = client.get_user().id(1).send()?;
/// ```
#[derive(RequestBuilder, Serialize)]
#[request_builder(
    method = "GET",
    path = "/users/{id}",
    response = User
)]
pub struct GetUser {
    /// The user ID
    pub id: u32,
}

// ============================================================================
// Request Structs - Comments
// ============================================================================

/// Get comments for a specific post (demonstrates nested resource paths)
///
/// # Example
/// ```rust,ignore
/// let comments = client.get_post_comments().post_id(1).send()?;
/// ```
#[derive(RequestBuilder, Serialize)]
#[request_builder(
    method = "GET",
    path = "/posts/{post_id}/comments",
    response = Vec<Comment>
)]
pub struct GetPostComments {
    /// The post ID to get comments for
    pub post_id: u32,
}

/// List comments, optionally filtered by post ID
///
/// # Example
/// ```rust,ignore
/// // Get all comments
/// let all_comments = client.list_comments().send()?;
///
/// // Get comments for a specific post
/// let post_comments = client.list_comments().post_id(1).send()?;
/// ```
#[derive(RequestBuilder, Serialize)]
#[request_builder(
    method = "GET",
    path = "/comments",
    response = Vec<Comment>
)]
pub struct ListComments {
    /// Filter comments by post ID
    #[request_builder(query)]
    #[serde(rename = "postId")]
    pub post_id: Option<u32>,
}

// ============================================================================
// API Client Configuration
// ============================================================================

/// Configuration for the JSON Placeholder API client
///
/// This is a simple unit struct since JSON Placeholder doesn't require
/// authentication or any configuration. The library automatically implements
/// `NoRequestConfiguration` for empty structs like this.
#[derive(Clone, ApiClient)]
#[api_client(
    base_url = "https://jsonplaceholder.typicode.com",
    requests(
        ListPosts = "list_posts",
        GetPost = "get_post",
        CreatePost = "create_post",
        UpdatePost = "update_post",
        PatchPost = "patch_post",
        DeletePost = "delete_post",
        ListUsers = "list_users",
        GetUser = "get_user",
        GetPostComments = "get_post_comments",
        ListComments = "list_comments"
    )
)]
pub struct JsonPlaceholderConfig;

// The ApiClient macro automatically generates:
// - JsonPlaceholderClient (blocking client)
// - Methods: list_posts(), get_post(), create_post(), etc.
// - Pre-configured with base URL and HTTP client
