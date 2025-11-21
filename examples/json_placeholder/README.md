# JSON Placeholder API Client Example

This example demonstrates how to use `derive_rest_api` to create a type-safe REST API client for the [JSON Placeholder API](https://jsonplaceholder.typicode.com).

## Features Demonstrated

This example showcases all major features of the `derive_rest_api` library:

### ✅ Path Parameters
- `GET /posts/{id}` - Single post by ID
- `GET /users/{id}` - Single user by ID
- `GET /posts/{postId}/comments` - Nested resource paths

### ✅ Query Parameters
- `GET /posts?userId={id}` - Filter posts by user
- `GET /comments?postId={id}` - Filter comments by post

### ✅ Request Bodies
- `POST /posts` - Create post with JSON body
- `PUT /posts/{id}` - Full update with JSON body
- `PATCH /posts/{id}` - Partial update with JSON body

### ✅ HTTP Methods
- `GET` - Retrieving resources
- `POST` - Creating resources
- `PUT` - Full updates
- `PATCH` - Partial updates
- `DELETE` - Deleting resources

### ✅ ApiClient Pattern
High-level client that wraps all endpoints with a clean, ergonomic API.

## Running the Example

```bash
# From the json_placeholder directory
cargo run --example demo

# Or from the workspace root
cargo run --example demo -p json_placeholder
```

## Project Structure

```
json_placeholder/
├── src/
│   └── lib.rs              # API client library
├── examples/
│   └── demo.rs             # Runnable demo
├── Cargo.toml
└── README.md
```

## Code Overview

### Data Models

The library defines strongly-typed models for JSON Placeholder resources:

```rust
pub struct Post {
    pub id: Option<u32>,
    pub user_id: u32,
    pub title: String,
    pub body: String,
}

pub struct User {
    pub id: u32,
    pub name: String,
    pub username: String,
    pub email: String,
    // ... more fields
}

pub struct Comment {
    pub id: Option<u32>,
    pub post_id: u32,
    pub name: String,
    pub email: String,
    pub body: String,
}
```

### Request Structs

Each API operation has a corresponding request struct with the `#[derive(RequestBuilder)]` attribute:

```rust
#[derive(RequestBuilder, Serialize)]
#[request_builder(
    method = "GET",
    path = "/posts/{id}",
    response = Post
)]
pub struct GetPost {
    pub id: u32,
}

#[derive(RequestBuilder, Serialize)]
#[request_builder(method = "GET", path = "/posts")]
pub struct ListPosts {
    #[request_builder(query)]
    #[serde(rename = "userId")]
    pub user_id: Option<u32>,
}
```

### API Client

The `#[derive(ApiClient)]` macro generates a high-level client:

```rust
#[derive(Clone, ApiClient)]
#[api_client(
    base_url = "https://jsonplaceholder.typicode.com",
    requests(
        ListPosts = "list_posts",
        GetPost = "get_post",
        CreatePost = "create_post",
        // ... more requests
    )
)]
pub struct JsonPlaceholderConfig;
```

This generates `JsonPlaceholderClient` with methods for each request type.

## Usage Examples

### Basic Usage

```rust
use derive_rest_api::ReqwestBlockingClient;
use json_placeholder::{JsonPlaceholderClient, JsonPlaceholderConfig};

// Create client
let client = JsonPlaceholderClient::<ReqwestBlockingClient>::with_client()
    .with_config(JsonPlaceholderConfig);

// List all posts
let posts = client.list_posts().send()?;

// Get a specific post
let post = client.get_post().id(1).send()?;
```

### Filtering with Query Parameters

```rust
// Get posts for a specific user
let user_posts = client.list_posts()
    .user_id(1)
    .send()?;

// Get comments for a specific post
let comments = client.list_comments()
    .post_id(1)
    .send()?;
```

### Creating Resources

```rust
use json_placeholder::CreatePostData;

let new_post = client.create_post()
    .data(CreatePostData {
        title: "Hello World".to_string(),
        body: "This is my first post!".to_string(),
        user_id: 1,
    })
    .send()?;
```

### Updating Resources

```rust
use json_placeholder::{UpdatePostData, PatchPostData};

// Full update (PUT)
let updated = client.update_post()
    .id(1)
    .data(UpdatePostData {
        title: "Updated".to_string(),
        body: "New content".to_string(),
        user_id: 1,
    })
    .send()?;

// Partial update (PATCH)
let patched = client.patch_post()
    .id(1)
    .data(PatchPostData {
        title: Some("New Title".to_string()),
        ..Default::default()
    })
    .send()?;
```

### Deleting Resources

```rust
client.delete_post().id(1).send()?;
```

### Nested Resources

```rust
// Get comments for a post using nested path
let comments = client.get_post_comments()
    .post_id(1)
    .send()?;
```

## API Reference

### Posts

| Method | Request Type | Description |
|--------|-------------|-------------|
| `list_posts()` | `ListPosts` | List all posts, optionally filtered by userId |
| `get_post()` | `GetPost` | Get a single post by ID |
| `create_post()` | `CreatePost` | Create a new post |
| `update_post()` | `UpdatePost` | Fully update a post (PUT) |
| `patch_post()` | `PatchPost` | Partially update a post (PATCH) |
| `delete_post()` | `DeletePost` | Delete a post |

### Users

| Method | Request Type | Description |
|--------|-------------|-------------|
| `list_users()` | `ListUsers` | List all users |
| `get_user()` | `GetUser` | Get a single user by ID |

### Comments

| Method | Request Type | Description |
|--------|-------------|-------------|
| `get_post_comments()` | `GetPostComments` | Get comments for a specific post (nested path) |
| `list_comments()` | `ListComments` | List all comments, optionally filtered by postId |

## Learning Points

### 1. No Authentication Required
JSON Placeholder is a public API that doesn't require authentication, making it perfect for learning. For examples with authentication, check out other examples in the repository.

### 2. Builder Pattern
All requests use the builder pattern for a clean, type-safe API:
```rust
client.get_post()
    .id(123)
    .send()?
```

### 3. Type Safety
The compiler ensures you provide all required parameters:
```rust
// This won't compile - missing id:
// client.get_post().send()?

// This compiles:
client.get_post().id(1).send()?
```

### 4. Serde Integration
Field names are automatically renamed using serde attributes:
```rust
#[serde(rename = "userId")]
pub user_id: u32,  // Becomes "userId" in JSON
```

### 5. Automatic Request Configuration
The `ApiClient` automatically configures:
- Base URL
- HTTP client
- Request/response serialization
- Error handling

## Next Steps

To learn more about `derive_rest_api`:

1. **Read the main README**: See the workspace root for comprehensive documentation
2. **Check other examples**: Look for examples with authentication
3. **Explore the source**: Review `src/lib.rs` to see how everything works
4. **Build your own**: Create a client for your favorite API!

## Resources

- [JSON Placeholder API Documentation](https://jsonplaceholder.typicode.com/)
- [derive_rest_api GitHub Repository](../../README.md)
- [Rust Serde Documentation](https://serde.rs/)
