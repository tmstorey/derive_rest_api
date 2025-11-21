# JSON Placeholder WASM Demo

This demo showcases the JSON Placeholder API client running in WebAssembly in the browser.

## Prerequisites

Before you can build and run this demo, you need to install:

1. **Rust** (if not already installed):
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **wasm-pack** - A tool for building Rust-generated WebAssembly:
   ```bash
   cargo install wasm-pack
   ```

3. **A simple HTTP server** - You can use any of these:
   ```bash
   # Option 1: Python (usually pre-installed)
   python3 -m http.server 8080

   # Option 2: Install a dedicated server
   cargo install basic-http-server
   # or
   npm install -g http-server
   ```

## Building the WASM Module

From this directory (`examples/json_placeholder/wasm-demo`), run:

```bash
wasm-pack build --target web
```

This will:
- Compile your Rust code to WebAssembly
- Generate JavaScript bindings
- Create a `pkg/` directory with all the necessary files

### Build Options

- **Development build** (faster compilation, larger files):
  ```bash
  wasm-pack build --target web --dev
  ```

- **Release build** (slower compilation, optimized):
  ```bash
  wasm-pack build --target web --release
  ```

## Running the Demo

1. **Build the WASM module** (if you haven't already):
   ```bash
   wasm-pack build --target web
   ```

2. **Start a local web server** from this directory:
   ```bash
   # Using Python
   python3 -m http.server 8080

   # OR using basic-http-server
   basic-http-server .

   # OR using http-server
   http-server -p 8080
   ```

3. **Open your browser** and navigate to:
   ```
   http://localhost:8080
   ```

4. **Open the browser console** (F12 or Cmd+Option+I) to see detailed output

5. **Click the buttons**:
   - "Test WASM Connection" - Verifies the WASM module loaded correctly
   - "Run API Demo" - Executes all the API calls and logs results

## What the Demo Does

The WASM demo performs the same operations as the regular async demo:

- ✅ Lists all posts
- ✅ Gets a specific post
- ✅ Creates a new post
- ✅ Updates a post (PUT)
- ✅ Patches a post (PATCH)
- ✅ Lists all users
- ✅ Gets a specific user
- ✅ Gets comments for a post
- ✅ Lists comments with filters

All API calls are made directly from the browser using the `derive_rest_api` library compiled to WebAssembly!

## Project Structure

```
wasm-demo/
├── Cargo.toml          # WASM-specific dependencies
├── src/
│   └── lib.rs          # WASM demo implementation
├── index.html          # Test page with UI
├── README.md           # This file
└── pkg/                # Generated WASM files (after build)
    ├── json_placeholder_wasm.js
    ├── json_placeholder_wasm_bg.wasm
    └── ...
```

## Troubleshooting

### CORS Errors

If you see CORS errors in the console, make sure you're:
1. Using a proper HTTP server (not opening `index.html` directly as `file://`)
2. The JSON Placeholder API (jsonplaceholder.typicode.com) allows CORS from any origin

### Module Not Found

If you see "Module not found" errors:
1. Make sure you've run `wasm-pack build --target web`
2. Check that the `pkg/` directory exists and contains the WASM files
3. Verify your HTTP server is serving from the correct directory

### Build Errors

If you get compilation errors:
1. Make sure you're in the `wasm-demo` directory
2. Try cleaning and rebuilding:
   ```bash
   cargo clean
   wasm-pack build --target web
   ```

### Network Errors

If API calls fail:
1. Check your internet connection
2. Open the browser console to see the full error message
3. The JSON Placeholder API might be temporarily unavailable

## Learn More

- [wasm-pack documentation](https://rustwasm.github.io/wasm-pack/)
- [wasm-bindgen guide](https://rustwasm.github.io/wasm-bindgen/)
- [Rust and WebAssembly book](https://rustwasm.github.io/book/)
- [JSON Placeholder API docs](https://jsonplaceholder.typicode.com/)

## Browser Compatibility

This demo works in all modern browsers that support WebAssembly:
- Chrome/Edge 57+
- Firefox 52+
- Safari 11+
- Opera 44+
