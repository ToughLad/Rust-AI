# Development Guide

## Getting Started

### Prerequisites
- **Rust 1.70+** - Install from [rustup.rs](https://rustup.rs/)
- **Git** - Version control
- **IDE/Editor** with Rust support (VS Code + rust-analyzer recommended)

### Quick Setup

1. **Clone Repository**
   ```bash
   git clone https://github.com/ToughLad/Rust-AI.git
   cd Rust-AI
   ```

2. **Install Dependencies**
   ```bash
   cargo build
   ```

3. **Environment Configuration**
   ```bash
   cp .env.example .env
   # Edit .env with your API keys and configuration
   ```

4. **Run Development Server**
   ```bash
   cargo run
   ```

The server will start at `http://localhost:3000` by default.

---

## Development Workflow

### Code Structure

```
src/
├── main.rs              # Application entry point & HTTP server
├── auth.rs              # Authentication service
├── config.rs            # Configuration management
├── convex_service.rs    # Database abstraction layer
├── file_processor.rs    # File handling utilities
├── routing.rs           # Provider routing logic
├── search_service.rs    # Web search integration
├── types.rs             # Type definitions & serialization
└── lib.rs               # Library exports
```

### Adding a New Provider

1. **Update Types** (`types.rs`)
   ```rust
   #[derive(Debug, Clone, Serialize, Deserialize)]
   #[serde(rename_all = "lowercase")]
   pub enum Provider {
       // ... existing providers
       #[serde(rename = "newprovider")]
       NewProvider,
   }
   ```

2. **Implement Provider Logic** (`routing.rs`)
   ```rust
   impl ProviderRouter {
       async fn route_to_new_provider(&self, request: &InvokeRequest) -> Result<Value> {
           // Implement provider-specific logic
           todo!("Implement NewProvider integration")
       }
   }
   ```

3. **Add Configuration** (`config.rs`)
   ```rust
   pub struct Config {
       // ... existing fields
       pub new_provider_api_key: String,
   }
   ```

4. **Environment Variable**
   ```bash
   NEW_PROVIDER_API_KEY=your-api-key-here
   ```

### Testing

#### Unit Tests
```bash
# Run all tests
cargo test

# Run specific test
cargo test test_auth_service

# Run with output
cargo test -- --nocapture
```

#### Integration Tests
```bash
# Run integration tests
cargo test --test integration_tests
```

#### Manual Testing
```bash
# Health check
curl http://localhost:3000/health

# Create anonymous session
curl -X POST http://localhost:3000/v1/auth/anonymous

# Test invoke endpoint
curl -X POST http://localhost:3000/v1/invoke \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer <token>" \
  -d '{"op": "chat", "input": {"messages": [{"role": "user", "content": "Hello"}]}}'
```

---

## Code Guidelines

### Rust Best Practices

#### Error Handling
```rust
// Use Result<T> for fallible operations
pub async fn create_user(&self, request: CreateUserRequest) -> Result<AuthResult> {
    match self.convex_service.get_user(&request.email).await {
        Ok(Some(_)) => Ok(AuthResult::error("User exists".to_string())),
        Ok(None) => {
            // Create user...
        }
        Err(e) => Err(anyhow!("Database error: {}", e)),
    }
}
```

#### Type Safety
```rust
// Use strong types instead of primitives
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserId(String);

#[derive(Debug, Clone, Serialize, Deserialize)]  
pub struct ApiKey(String);

// Instead of: fn get_user(id: String) -> User
pub fn get_user(id: UserId) -> User {
    // Implementation
}
```

#### Async/Await
```rust
// Use async/await for I/O operations
pub async fn call_provider(&self, request: &InvokeRequest) -> Result<Value> {
    let response = reqwest::Client::new()
        .post(&self.provider_url)
        .json(request)
        .send()
        .await?;
    
    let result: Value = response.json().await?;
    Ok(result)
}
```

### Code Style

#### Formatting
- Use `cargo fmt` for consistent formatting
- Line length: 100 characters
- Use snake_case for functions and variables
- Use PascalCase for types and structs

#### Documentation
```rust
/// Creates a new user account with the provided credentials.
/// 
/// # Arguments
/// * `request` - User registration information
/// 
/// # Returns
/// * `Result<AuthResult>` - Success with user data or error
/// 
/// # Errors
/// * Returns error if email already exists
/// * Returns error if password validation fails
pub async fn create_user(&self, request: CreateUserRequest) -> Result<AuthResult> {
    // Implementation...
}
```

#### Logging
```rust
use tracing::{info, warn, error, debug};

// Info for normal operations
info!("User {} logged in successfully", user.email);

// Warn for unusual but handled situations  
warn!("Rate limit exceeded for user {}", user.id);

// Error for actual errors
error!("Failed to connect to provider: {}", error);

// Debug for detailed debugging info
debug!("Processing request: {:?}", request);
```

---

## Database Schema

### User Account Structure
```json
{
  "_id": "user-uuid",
  "email": "user@example.com",
  "password_hash": "$2b$12$...",
  "subscription_tier": "free|pro|enterprise", 
  "api_key": "ak_xxxxxxxxxxxxxxxx",
  "is_active": true,
  "created_at": "2025-01-02T12:00:00Z",
  "updated_at": "2025-01-02T12:00:00Z"
}
```

### Analytics Event Structure
```json
{
  "_id": "event-uuid",
  "event_type": "user_login|invoke_request|error",
  "user_id": "user-uuid",
  "timestamp": "2025-01-02T12:00:00Z",
  "metadata": {
    "provider": "openai",
    "model": "gpt-3.5-turbo", 
    "tokens_used": 150,
    "response_time_ms": 1200
  }
}
```

---

## Environment Configuration

### Required Variables
```bash
# Server
BIND_ADDRESS=0.0.0.0:3000

# JWT Secret (generate with: openssl rand -base64 32)
ACTION_TOKEN_SECRET=your-secret-key-here

# Database
CONVEX_URL=https://your-deployment.convex.cloud
```

### Provider API Keys
```bash
# OpenAI
OPENAI_API_KEY=sk-...

# Anthropic  
ANTHROPIC_API_KEY=sk-ant-...

# Mistral
MISTRAL_API_KEY=...

# Cloudflare
CLOUDFLARE_API_TOKEN=...
CLOUDFLARE_ACCOUNT_ID=...

# XAI
XAI_API_KEY=...

# Groq
GROQ_API_KEY=gsk_...

# OpenRouter
OPENROUTER_API_KEY=sk-or-...
```

### Optional Configuration
```bash
# Clerk Authentication (optional)
CLERK_SECRET_KEY=sk_live_...
CLERK_PUBLISHABLE_KEY=pk_live_...

# Logging Level
RUST_LOG=info

# Development Mode
RUST_ENV=development
```

---

## Deployment

### Docker Deployment
```dockerfile
FROM rust:1.75-slim as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates
COPY --from=builder /app/target/release/rust-ai /usr/local/bin/
EXPOSE 3000
CMD ["rust-ai"]
```

### Production Considerations

#### Performance
- Use `--release` flag for production builds
- Configure appropriate worker pool sizes
- Enable HTTP/2 support
- Use load balancer for multiple instances

#### Security
- Use HTTPS in production
- Rotate JWT secrets regularly
- Secure API keys in environment
- Enable request logging
- Set up rate limiting at load balancer level

#### Monitoring
- Set up structured logging
- Configure metrics collection
- Monitor response times
- Track error rates
- Set up alerting

---

## Contributing

### Pull Request Process

1. **Fork the Repository**
2. **Create Feature Branch**
   ```bash
   git checkout -b feature/new-provider
   ```

3. **Make Changes**
   - Follow code guidelines
   - Add tests for new functionality
   - Update documentation

4. **Test Changes**
   ```bash
   cargo test
   cargo clippy
   cargo fmt --check
   ```

5. **Submit Pull Request**
   - Clear description of changes
   - Link related issues
   - Include test results

### Code Review Guidelines

- **Functionality**: Does the code work as intended?
- **Performance**: Are there any performance implications?
- **Security**: Are there security vulnerabilities?
- **Maintainability**: Is the code readable and maintainable?
- **Testing**: Are there adequate tests?

---

## Debugging

### Common Issues

#### "Connection refused" errors
- Check if server is running: `cargo run`
- Verify port availability: `netstat -an | grep 3000`
- Check bind address in configuration

#### JWT token errors  
- Verify `ACTION_TOKEN_SECRET` is set
- Check token expiration
- Validate token format

#### Provider API errors
- Verify API keys are correct
- Check API key permissions
- Monitor provider rate limits

### Debugging Tools

#### Logging
```bash
# Enable debug logging
RUST_LOG=debug cargo run

# Enable trace logging
RUST_LOG=trace cargo run

# Log specific modules
RUST_LOG=rust_ai::auth=debug cargo run
```

#### HTTP Debugging
```bash
# Use curl with verbose output
curl -v http://localhost:3000/health

# Use httpie for pretty output  
http POST localhost:3000/v1/invoke
```

#### Rust Debugging
```rust
// Add debug prints
dbg!(&user_request);

// Use tracing for structured logging
tracing::debug!("Processing user: {:?}", user);
```