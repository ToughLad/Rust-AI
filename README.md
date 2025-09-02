# Rust-AI

A high-performance, multi-provider AI API aggregator built in Rust. This service provides a unified interface for accessing various AI providers including OpenAI, Anthropic, Mistral, Cloudflare, Meta, XAI, Groq, and OpenRouter.

## 🚀 Features

### Multi-Provider Support
- **OpenAI** - GPT models and completions
- **Anthropic** - Claude models
- **Mistral** - Mistral AI models  
- **Cloudflare** - Workers AI models
- **Meta** - Llama models
- **XAI** - Grok models
- **Groq** - High-speed inference
- **OpenRouter** - Model routing service

### Core Capabilities
- **Unified API** - Single interface for all providers
- **Authentication System** - JWT-based auth with user management
- **Rate Limiting** - Built-in request limiting for guests and users
- **Search Integration** - Optional web search capabilities
- **File Attachments** - Support for document processing
- **Analytics** - Request tracking and usage analytics
- **Configuration Management** - Environment-based configuration

### Technical Features
- **High Performance** - Built with Rust and Axum for maximum speed
- **Async/Await** - Fully asynchronous request handling
- **Type Safety** - Comprehensive type definitions with validation
- **Error Handling** - Robust error management with detailed logging
- **CORS Support** - Cross-origin resource sharing enabled
- **Graceful Shutdown** - Proper signal handling for clean shutdowns

## 🏗️ Architecture

### Core Components

#### **Authentication Service** (`auth.rs`)
Handles user registration, login, JWT token generation and verification. Supports both registered users and anonymous sessions with different rate limits.

#### **Convex Service** (`convex_service.rs`)  
Database abstraction layer providing user management, analytics logging, and data persistence capabilities.

#### **Search Service** (`search_service.rs`)
Optional web search integration to enhance AI responses with current information.

#### **File Processor** (`file_processor.rs`)
Handles file attachments and document processing for context-aware AI interactions.

#### **Routing System** (`routing.rs`)
Intelligent request routing to appropriate AI providers based on model availability and user preferences.

### API Endpoints

#### Authentication
- `POST /v1/auth/register` - Create new user account
- `POST /v1/auth/login` - User login
- `POST /v1/auth/anonymous` - Create anonymous session

#### Core API  
- `POST /v1/invoke` - Main AI completion endpoint
- `GET /v1/analytics` - Usage analytics (hours parameter optional)
- `GET /health` - Health check

### Data Types

#### **InvokeRequest**
```rust
pub struct InvokeRequest {
    pub op: Operation,           // chat, fim (fill-in-middle)
    pub tier: Option<String>,    // subscription tier
    pub input: HashMap<String, Value>, // provider-specific input
    pub options: Option<InvokeOptions>, // temperature, max_tokens, etc.
    pub token: Option<String>,   // auth token
    pub enable_search: Option<bool>, // web search toggle
    pub attachments: Option<Vec<Attachment>>, // file attachments
}
```

#### **Supported Providers**
- `cf` (Cloudflare)
- `mistral` 
- `openai`
- `xai`
- `groq` 
- `openrouter`
- `meta`
- `anthropic`

## 🛠️ Setup & Installation

### Prerequisites
- Rust 1.70+ 
- Cargo

### Environment Configuration
Create a `.env` file with the following variables:

```bash
# Server Configuration
BIND_ADDRESS=0.0.0.0:3000

# JWT Secret
ACTION_TOKEN_SECRET=your-jwt-secret-here

# Provider API Keys
OPENAI_API_KEY=your-openai-key
ANTHROPIC_API_KEY=your-anthropic-key  
MISTRAL_API_KEY=your-mistral-key
CLOUDFLARE_API_TOKEN=your-cf-token
XAI_API_KEY=your-xai-key
GROQ_API_KEY=your-groq-key
OPENROUTER_API_KEY=your-openrouter-key

# Optional: Clerk Authentication
CLERK_SECRET_KEY=your-clerk-secret
CLERK_PUBLISHABLE_KEY=your-clerk-public-key

# Database/Service URLs
CONVEX_URL=your-convex-deployment-url
```

### Installation & Running

1. **Clone the repository**
   ```bash
   git clone https://github.com/ToughLad/Rust-AI.git
   cd Rust-AI
   ```

2. **Install dependencies**
   ```bash
   cargo build
   ```

3. **Run the server**
   ```bash
   cargo run
   ```

The server will start on the configured bind address (default: `localhost:3000`).

## 📖 Usage Examples

### Basic Chat Completion
```bash
curl -X POST http://localhost:3000/v1/invoke \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer your-jwt-token" \
  -d '{
    "op": "chat",
    "input": {
      "messages": [
        {"role": "user", "content": "Hello, how are you?"}
      ],
      "model": "gpt-3.5-turbo",
      "provider": "openai"
    },
    "options": {
      "temperature": 0.7,
      "max_tokens": 150
    }
  }'
```

### Anonymous Session
```bash
# Create anonymous session
curl -X POST http://localhost:3000/v1/auth/anonymous

# Use the returned token for requests (limited to 5 requests per day)
curl -X POST http://localhost:3000/v1/invoke \
  -H "Authorization: Bearer anon-token-here" \
  -d '{"op": "chat", "input": {...}}'
```

### With Search Enhancement
```bash
curl -X POST http://localhost:3000/v1/invoke \
  -H "Authorization: Bearer your-token" \
  -d '{
    "op": "chat",
    "input": {
      "messages": [{"role": "user", "content": "What are the latest developments in AI?"}]
    },
    "enable_search": true
  }'
```

## 🔒 Rate Limiting

- **Anonymous Users**: 5 requests per day (fallback in-memory tracking)
- **Registered Users**: Configurable based on subscription tier
- **Rate limit headers** included in responses
- **Automatic daily reset** at midnight UTC

## 📊 Monitoring

The service provides comprehensive logging and analytics:

- **Request logging** with tracing support
- **Error tracking** with detailed error messages  
- **Analytics endpoint** for usage statistics
- **Health check endpoint** for monitoring

## 🤝 Contributing

This project is licensed under CC BY-NC-SA 4.0. You may:
- Use the code for non-commercial purposes
- Modify and distribute with attribution
- Must share derivatives under the same license

**Commercial use is prohibited** without explicit permission.

## 📄 License

This project is licensed under the Creative Commons Attribution-NonCommercial-ShareAlike 4.0 International License.

- ✅ **Non-commercial use** allowed
- ✅ **Modifications** permitted  
- ✅ **Sharing** encouraged
- ❌ **Commercial use** prohibited
- 🔄 **Share-alike** required for derivatives

See the [LICENSE](LICENSE) file for full details.

## 🏷️ Project Status

**Current Version**: 0.1.0  
**Status**: Development/Alpha  
**Rust Edition**: 2021

This is an early-stage project. The core architecture is in place, but some features are still being implemented:

- ✅ Basic HTTP server with Axum
- ✅ Authentication system
- ✅ Type-safe request/response handling
- ✅ Multi-provider architecture
- 🚧 Provider implementations (in progress)
- 🚧 Complete routing logic
- 🚧 Search service integration
- 🚧 File attachment processing

---

**Made with ❤️ and ⚡ by ToughLad**