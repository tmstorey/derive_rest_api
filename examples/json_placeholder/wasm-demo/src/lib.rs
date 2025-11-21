//! WASM demo for the JSON Placeholder API client
//!
//! This demo showcases async usage in a WebAssembly environment.

use json_placeholder::JsonPlaceholderAsyncClient;
use wasm_bindgen::prelude::*;

// Set up panic hook for better error messages in the browser console
#[wasm_bindgen(start)]
pub fn init() {
    console_error_panic_hook::set_once();
}

/// Run the JSON Placeholder demo
///
/// This function demonstrates all the API operations in a WASM context.
/// Call it from JavaScript: `run_demo()`
#[wasm_bindgen]
pub async fn run_demo() -> Result<String, JsValue> {
    log("=== JSON Placeholder API WASM Demo ===\n");

    // Create async API client
    let client = JsonPlaceholderAsyncClient::new();

    // ========================================================================
    // Posts - List
    // ========================================================================
    log("📋 Listing all posts...");
    let posts = client
        .list_posts()
        .send_async()
        .await
        .map_err(|e| JsValue::from_str(&format!("Error listing posts: {}", e)))?;
    log(&format!("   Found {} posts", posts.len()));
    if let Some(first_post) = posts.first() {
        log(&format!("   First post: \"{}\"", first_post.title));
    }
    log("");

    // ========================================================================
    // Posts - Get Single
    // ========================================================================
    log("📄 Getting post #1...");
    let post = client
        .get_post()
        .id(1)
        .send_async()
        .await
        .map_err(|e| JsValue::from_str(&format!("Error getting post: {}", e)))?;
    log(&format!("   Title: {}", post.title));
    log(&format!(
        "   Body: {}{}",
        post.body.chars().take(50).collect::<String>(),
        if post.body.len() > 50 { "..." } else { "" }
    ));
    log("");

    // ========================================================================
    // Posts - Create
    // ========================================================================
    log("✏️  Creating a new post...");
    let new_post = client
        .create_post()
        .title("Hello from WASM!".to_string())
        .body(
            "This post was created from WebAssembly using derive_rest_api!".to_string(),
        )
        .user_id(1)
        .send_async()
        .await
        .map_err(|e| JsValue::from_str(&format!("Error creating post: {}", e)))?;
    log(&format!("   Created post with ID: {}", new_post.id));
    log("");

    // ========================================================================
    // Posts - Update (PUT)
    // ========================================================================
    log("🔄 Updating post #1 (PUT)...");
    let updated_post = client
        .update_post()
        .id(1)
        .title("Updated from WASM".to_string())
        .body("This post has been updated from WebAssembly.".to_string())
        .user_id(1)
        .send_async()
        .await
        .map_err(|e| JsValue::from_str(&format!("Error updating post: {}", e)))?;
    log(&format!("   Updated id: {}", updated_post.id));
    log("");

    // ========================================================================
    // Posts - Patch (PATCH)
    // ========================================================================
    log("🔧 Patching post #1 (PATCH)...");
    let patched_post = client
        .patch_post()
        .id(1)
        .title("Patched from WASM".to_string())
        .send_async()
        .await
        .map_err(|e| JsValue::from_str(&format!("Error patching post: {}", e)))?;
    log(&format!("   Patched title: {}", patched_post.title));
    log("");

    // ========================================================================
    // Users - List
    // ========================================================================
    log("👥 Listing all users...");
    let users = client
        .list_users()
        .send_async()
        .await
        .map_err(|e| JsValue::from_str(&format!("Error listing users: {}", e)))?;
    log(&format!("   Found {} users", users.len()));
    if let Some(first_user) = users.first() {
        log(&format!(
            "   First user: {} (@{})",
            first_user.name, first_user.username
        ));
    }
    log("");

    // ========================================================================
    // Users - Get Single
    // ========================================================================
    log("👤 Getting user #1...");
    let user = client
        .get_user()
        .id(1)
        .send_async()
        .await
        .map_err(|e| JsValue::from_str(&format!("Error getting user: {}", e)))?;
    log(&format!("   Name: {}", user.name));
    log(&format!("   Email: {}", user.email));
    log(&format!("   Username: {}", user.username));
    if let Some(company) = &user.company {
        log(&format!("   Company: {}", company.name));
    }
    log("");

    // ========================================================================
    // Comments - Get for Post (nested resource)
    // ========================================================================
    log("💬 Getting comments for post #1...");
    let comments = client
        .get_post_comments()
        .post_id(1)
        .send_async()
        .await
        .map_err(|e| JsValue::from_str(&format!("Error getting comments: {}", e)))?;
    log(&format!("   Found {} comments", comments.len()));
    if let Some(first_comment) = comments.first() {
        log(&format!("   First comment by: {}", first_comment.name));
        log(&format!("   Email: {}", first_comment.email));
    }
    log("");

    log("✅ WASM demo completed successfully!");

    Ok("Demo completed successfully!".to_string())
}

/// Simple test function that returns a greeting
///
/// This is useful for verifying that WASM is loaded correctly.
/// Call it from JavaScript: `greet("World")`
#[wasm_bindgen]
pub fn greet(name: &str) -> String {
    format!("Hello from WASM, {}!", name)
}

// Helper function to log to browser console
fn log(s: &str) {
    web_sys::console::log_1(&JsValue::from_str(s));
}
