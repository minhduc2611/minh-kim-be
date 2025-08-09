# Rust

## Rust Version Management

This project uses specific Rust version management to ensure consistency across development environments:

### 🔧 `rust-toolchain.toml`
- **Purpose**: Specifies the exact Rust version to use (like `.nvmrc` for Node.js)
- **Auto-switching**: `rustup` automatically uses the specified version when you're in this directory
- **Team consistency**: Everyone gets the same Rust version, components, and targets
- **Current version**: `1.88.0` with `rustfmt` and `clippy` components

### 📋 `rust-version` in Cargo.toml
- **Purpose**: Defines the minimum Rust version required for this project
- **Compatibility**: Used by `cargo` for version compatibility checking
- **Current minimum**: `1.75.0`

### 🚀 Getting Started
When you clone this project and run any `rustup` command, it will automatically:
1. Detect the `rust-toolchain.toml` file
2. Install Rust 1.88.0 if you don't have it
3. Switch to using that version for this project

## Update Rust

```bash
rustup update
```

## Run

```bash
cargo run
```

## Vertex AI Verbose Mode

The Vertex AI service supports verbose mode for debugging API requests. When enabled, it will print out the equivalent curl command that would be used to make the same request.

### Enabling Verbose Mode

You can enable verbose mode in several ways:

#### Option 1: During instantiation
```rust
use services::vertex_ai_service::VertexAIService;
use services::vertex_ai_service_trait::VertexAIConfig;

let config = VertexAIConfig {
    project_id: "your-project-id".to_string(),
    location: "us-central1".to_string(),
    verbose: true, // Enable verbose mode
};

let vertex_ai_service = VertexAIService::new(Some(config));
```

#### Option 2: Using the builder pattern
```rust
let vertex_ai_service = VertexAIService::new(None)
    .with_verbose(true);
```

### Example Output

When verbose mode is enabled, you'll see output like this:

```
=== VERTEX AI VERBOSE MODE ===
CURL equivalent request:
curl -X POST \
  'https://us-central1-aiplatform.googleapis.com/v1/projects/your-project/locations/us-central1/publishers/google/models/gemini-2.0-flash-001:generateContent' \
  -H 'Content-Type: application/json' \
  -H 'Authorization: Bearer $(gcloud auth print-access-token)' \
  -d '{
  "model": "projects/your-project/locations/us-central1/publishers/google/models/gemini-2.0-flash-001",
  "contents": [
    {
      "role": "user",
      "parts": [
        {
          "text": "Hello, how are you?"
        }
      ]
    }
  ],
  "generationConfig": {
    "temperature": 0.2,
    "topP": 1.0,
    "topK": 40.0,
    "maxOutputTokens": 4096
  }
}'
=== END VERBOSE MODE ===
```

This is useful for:
- Debugging API requests
- Understanding the exact payload being sent
- Testing requests manually with curl
- Troubleshooting authentication issues

## terminate port
```
lsof -ti:8080 | xargs kill -9
```