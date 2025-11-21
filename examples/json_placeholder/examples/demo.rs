//! Comprehensive demo of the JSON Placeholder API client
//!
//! Run with:
//! ```bash
//! cargo run --example demo
//! ```

use derive_rest_api::ReqwestBlockingClient;
use json_placeholder::{
    CreatePostData, JsonPlaceholderClient, JsonPlaceholderConfig, PatchPostData, UpdatePostData,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== JSON Placeholder API Demo ===\n");

    // Create API client
    let client = JsonPlaceholderClient::<ReqwestBlockingClient>::with_client()
        .with_config(JsonPlaceholderConfig);

    // ========================================================================
    // Posts - List
    // ========================================================================
    println!("📋 Listing all posts...");
    let posts = client.list_posts().send()?;
    println!("   Found {} posts", posts.len());
    if let Some(first_post) = posts.first() {
        println!("   First post: \"{}\"", first_post.title);
    }
    println!();

    // ========================================================================
    // Posts - List with Filter
    // ========================================================================
    println!("📋 Listing posts for user 1...");
    let user_posts = client.list_posts().user_id(1).send()?;
    println!("   Found {} posts for user 1", user_posts.len());
    println!();

    // ========================================================================
    // Posts - Get Single
    // ========================================================================
    println!("📄 Getting post #1...");
    let post = client.get_post().id(1).send()?;
    println!("   Title: {}", post.title);
    println!("   Body: {}", post.body.chars().take(50).collect::<String>());
    if post.body.len() > 50 {
        println!("...");
    }
    println!();

    // ========================================================================
    // Posts - Create
    // ========================================================================
    println!("✏️  Creating a new post...");
    let new_post = client
        .create_post()
        .data(CreatePostData {
            title: "Hello from derive_rest_api!".to_string(),
            body: "This post was created using the derive_rest_api library. It makes building type-safe REST clients easy!".to_string(),
            user_id: 1,
        })
        .send()?;
    println!("   Created post with ID: {:?}", new_post.id);
    println!("   Title: {}", new_post.title);
    println!();

    // ========================================================================
    // Posts - Update (PUT)
    // ========================================================================
    println!("🔄 Updating post #1 (PUT)...");
    let updated_post = client
        .update_post()
        .id(1)
        .data(UpdatePostData {
            title: "Updated Title".to_string(),
            body: "This post has been completely updated.".to_string(),
            user_id: 1,
        })
        .send()?;
    println!("   Updated title: {}", updated_post.title);
    println!();

    // ========================================================================
    // Posts - Patch (PATCH)
    // ========================================================================
    println!("🔧 Patching post #1 (PATCH)...");
    let patched_post = client
        .patch_post()
        .id(1)
        .data(PatchPostData {
            title: Some("Partially Updated Title".to_string()),
            body: None,
            user_id: None,
        })
        .send()?;
    println!("   Patched title: {}", patched_post.title);
    println!();

    // ========================================================================
    // Posts - Delete
    // ========================================================================
    println!("🗑️  Deleting post #1...");
    client.delete_post().id(1).send()?;
    println!("   Post deleted successfully");
    println!();

    // ========================================================================
    // Users - List
    // ========================================================================
    println!("👥 Listing all users...");
    let users = client.list_users().send()?;
    println!("   Found {} users", users.len());
    if let Some(first_user) = users.first() {
        println!("   First user: {} (@{})", first_user.name, first_user.username);
    }
    println!();

    // ========================================================================
    // Users - Get Single
    // ========================================================================
    println!("👤 Getting user #1...");
    let user = client.get_user().id(1).send()?;
    println!("   Name: {}", user.name);
    println!("   Email: {}", user.email);
    println!("   Username: {}", user.username);
    if let Some(company) = &user.company {
        println!("   Company: {}", company.name);
    }
    println!();

    // ========================================================================
    // Comments - Get for Post (nested resource)
    // ========================================================================
    println!("💬 Getting comments for post #1...");
    let comments = client.get_post_comments().post_id(1).send()?;
    println!("   Found {} comments", comments.len());
    if let Some(first_comment) = comments.first() {
        println!("   First comment by: {}", first_comment.name);
        println!("   Email: {}", first_comment.email);
    }
    println!();

    // ========================================================================
    // Comments - List with Filter
    // ========================================================================
    println!("💬 Listing comments for post #2 (via query param)...");
    let filtered_comments = client.list_comments().post_id(2).send()?;
    println!("   Found {} comments for post #2", filtered_comments.len());
    println!();

    println!("✅ Demo completed successfully!");

    Ok(())
}
