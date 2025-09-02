# API Reference

## Base URL
```
http://localhost:3000
```

## Authentication

All API endpoints (except health check) require authentication via JWT tokens passed in the Authorization header:

```
Authorization: Bearer <jwt-token>
```

### Authentication Types

1. **User Sessions** - Full access with subscription-based limits
2. **Anonymous Sessions** - Limited access (5 requests/day)

---

## Endpoints

### Health Check

**GET** `/health`

Check if the service is running.

**Response:**
```json
{
  "status": "healthy",
  "timestamp": "2025-01-02T12:00:00Z"
}
```

---

### Authentication Endpoints

#### Register User

**POST** `/v1/auth/register`

Create a new user account.

**Request Body:**
```json
{
  "email": "user@example.com",
  "password": "secure-password",
  "subscription_tier": "free"  // optional, defaults to "free"
}
```

**Response:**
```json
{
  "status": "success",
  "data": {
    "id": "user-id",
    "email": "user@example.com", 
    "is_anonymous": false,
    "created_at": "2025-01-02T12:00:00Z"
  }
}
```

#### Login

**POST** `/v1/auth/login`

Authenticate existing user.

**Request Body:**
```json
{
  "email": "user@example.com",
  "password": "secure-password"
}
```

**Response:**
```json
{
  "status": "success",
  "data": {
    "token": "jwt-token-here",
    "user": {
      "id": "user-id",
      "email": "user@example.com",
      "is_anonymous": false,
      "created_at": "2025-01-02T12:00:00Z"
    }
  }
}
```

#### Create Anonymous Session

**POST** `/v1/auth/anonymous`

Create a temporary anonymous session.

**Response:**
```json
{
  "status": "success", 
  "data": {
    "token": "anon-jwt-token",
    "user": {
      "id": "anon-123456789-abc12345",
      "email": "anon-123456789-abc12345@anon.local",
      "is_anonymous": true,
      "created_at": "2025-01-02T12:00:00Z"
    }
  }
}
```

---

### Core API Endpoints

#### Invoke AI Model

**POST** `/v1/invoke`

Main endpoint for AI completions and interactions.

**Request Body:**
```json
{
  "op": "chat",  // "chat" or "fim" (fill-in-middle)
  "tier": "free", // optional subscription tier
  "input": {
    "messages": [
      {
        "role": "system",
        "content": "You are a helpful assistant."
      },
      {
        "role": "user", 
        "content": "Hello, how are you?"
      }
    ],
    "model": "gpt-3.5-turbo",
    "provider": "openai"
  },
  "options": {
    "temperature": 0.7,    // 0.0 - 2.0
    "max_tokens": 150      // positive integer
  },
  "token": "jwt-token",    // optional if in header
  "enable_search": false, // optional web search
  "attachments": [        // optional file attachments
    {
      "name": "document.pdf",
      "url": "https://example.com/file.pdf",
      "content_type": "application/pdf",
      "size": 1024000
    }
  ]
}
```

**Response:**
```json
{
  "status": "success",
  "data": {
    "request_id": "req-uuid-here",
    "status": "processed", 
    "message": "AI response content here"
  }
}
```

#### Get Analytics

**GET** `/v1/analytics?hours=24`

Retrieve usage analytics.

**Query Parameters:**
- `hours` (optional): Number of hours to look back (default varies)

**Response:**
```json
{
  "status": "success",
  "data": {
    "total_requests": 150,
    "requests_by_provider": {
      "openai": 75,
      "anthropic": 50,
      "mistral": 25
    },
    "average_response_time_ms": 1250,
    "period_hours": 24
  }
}
```

---

## Request/Response Formats

### Common Response Structure

All API responses follow this structure:

```json
{
  "status": "success|error",
  "data": {}, // Present on success
  "message": "Optional success message",
  "error": "Error description if failed"
}
```

### Error Responses

**400 Bad Request**
```json
{
  "status": "error",
  "error": "Invalid request format"
}
```

**401 Unauthorized** 
```json
{
  "status": "error",
  "error": "Authentication required"
}
```

**429 Too Many Requests**
```json
{
  "status": "error", 
  "error": "Rate limit exceeded"
}
```

**500 Internal Server Error**
```json
{
  "status": "error",
  "error": "Internal server error"
}
```

---

## Supported Providers

| Provider | Code | Models Supported |
|----------|------|------------------|
| OpenAI | `openai` | GPT-3.5, GPT-4 series |
| Anthropic | `anthropic` | Claude series |
| Mistral | `mistral` | Mistral models |
| Cloudflare | `cf` | Workers AI models |
| Meta | `meta` | Llama series |  
| XAI | `xai` | Grok models |
| Groq | `groq` | Various models |
| OpenRouter | `openrouter` | Multiple providers |

---

## Rate Limits

### Anonymous Users
- **5 requests per day**
- **Resets at midnight UTC**
- **In-memory tracking**

### Registered Users
- **Varies by subscription tier**
- **Database tracking** 
- **More generous limits**

### Rate Limit Headers
```
X-RateLimit-Limit: 5
X-RateLimit-Remaining: 3  
X-RateLimit-Reset: 1641024000
```

---

## Examples

### cURL Examples

**Basic Chat:**
```bash
curl -X POST http://localhost:3000/v1/invoke \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer your-jwt-token" \
  -d '{
    "op": "chat",
    "input": {
      "messages": [{"role": "user", "content": "Hello!"}],
      "model": "gpt-3.5-turbo", 
      "provider": "openai"
    }
  }'
```

**With Search:**
```bash
curl -X POST http://localhost:3000/v1/invoke \
  -H "Authorization: Bearer your-token" \
  -d '{
    "op": "chat",
    "input": {
      "messages": [{"role": "user", "content": "Latest AI news?"}]
    },
    "enable_search": true
  }'
```

### JavaScript/TypeScript Examples

```typescript
interface InvokeRequest {
  op: 'chat' | 'fim';
  input: {
    messages: Array<{role: string, content: string}>;
    model?: string;
    provider?: string;
  };
  options?: {
    temperature?: number;
    max_tokens?: number;
  };
  enable_search?: boolean;
}

const response = await fetch('/v1/invoke', {
  method: 'POST',
  headers: {
    'Content-Type': 'application/json',
    'Authorization': `Bearer ${token}`
  },
  body: JSON.stringify({
    op: 'chat',
    input: {
      messages: [{role: 'user', content: 'Hello!'}],
      model: 'gpt-3.5-turbo',
      provider: 'openai'
    },
    options: {
      temperature: 0.7,
      max_tokens: 150
    }
  })
});

const result = await response.json();
```