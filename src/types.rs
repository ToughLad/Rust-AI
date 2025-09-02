//! Type Definitions and Data Structures
//! 
//! This module contains all the shared type definitions used across
//! the Rust-AI application including:
//! - API request/response structures with validation
//! - Chat message and conversation types
//! - Provider and routing enumerations
//! - File attachment and search result types
//! - User authentication data structures
//! 
//! All types are designed to be serializable for API communication
//! and include proper validation where needed.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use validator::Validate;

/// Chat message structure for conversations
/// 
/// Represents a single message in a chat conversation with role-based
/// content organization (system, user, assistant).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    /// Role of the message sender (system, user, or assistant)
    pub role: MessageRole,
    /// Text content of the message
    pub content: String,
}

/// Message roles in chat conversations
/// 
/// Defines who sent each message in a conversation:
/// - System: Instructions or context for the AI
/// - User: Messages from the human user
/// - Assistant: Responses from the AI
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    /// System messages (prompts, instructions)
    System,
    /// User messages (human input)
    User,
    /// Assistant messages (AI responses)
    Assistant,
}

/// AI model generation options with validation
/// 
/// Controls the behavior of AI model inference with validated ranges
/// to ensure reasonable and safe parameter values.
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct InvokeOptions {
    /// Temperature for response randomness (0.0 = deterministic, 2.0 = very random)
    #[validate(range(min = 0.0, max = 2.0))]
    pub temperature: Option<f32>,
    /// Maximum tokens to generate in response
    #[validate(range(min = 1))]
    pub max_tokens: Option<u32>,
}

/// File attachment metadata for multimodal requests
/// 
/// Represents uploaded files that can be processed alongside text input.
/// Supports various content types including images, documents, and code.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attachment {
    /// Original filename of the uploaded file
    pub name: String,
    /// URL or data URI containing the file content
    pub url: String,
    /// MIME type of the file content
    pub content_type: String,
    /// File size in bytes (optional)
    pub size: Option<u64>,
}

/// Main API invocation request structure
/// 
/// This is the primary request format for all AI operations.
/// Contains all necessary information for routing, processing, and
/// generating AI responses.
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct InvokeRequest {
    /// Type of operation to perform (chat, code completion, etc.)
    pub op: Operation,
    /// User subscription tier for rate limiting and features
    pub tier: Option<String>,
    /// Input data specific to the operation type
    pub input: HashMap<String, serde_json::Value>,
    /// AI model generation options (temperature, max_tokens, etc.)
    pub options: Option<InvokeOptions>,
    /// Authentication token (JWT or API key)
    pub token: Option<String>,
    /// Whether to enhance response with web search results
    pub enable_search: Option<bool>,
    /// File attachments for multimodal processing
    pub attachments: Option<Vec<Attachment>>,
}

/// Operation types supported by the AI system
/// 
/// Defines the different types of AI operations that can be performed:
/// - Chat: Conversational interactions
/// - FIM: Fill-in-middle code completion
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Operation {
    /// Chat-based conversational AI
    Chat,
    /// Fill-in-middle code completion
    Fim,
}

/// AI provider enumeration
/// 
/// Lists all supported AI providers with their API identifiers.
/// Used for routing requests to specific providers and models.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Provider {
    /// Cloudflare Workers AI
    #[serde(rename = "cf")]
    Cloudflare,
    /// Mistral AI
    Mistral,
    /// OpenAI (GPT models)
    OpenAI,
    /// xAI (Grok models)
    #[serde(rename = "xai")]
    Xai,
    /// Groq (fast inference)
    Groq,
    /// OpenRouter (multi-provider aggregation)
    OpenRouter,
    /// Meta (Llama models)
    Meta,
    /// Anthropic (Claude models)
    Anthropic,
}

/// Route target for provider routing
/// 
/// Specifies which provider and model to use for a request.
/// Used by the routing system to direct requests appropriately.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteTarget {
    /// AI provider to use
    pub provider: Provider,
    /// Specific model name at the provider
    pub model: String,
}

/// Standardized API response wrapper
/// 
/// Provides consistent response format across all API endpoints
/// with success/error status and optional data payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    /// Response status ("success" or "error")
    pub status: String,
    /// Response data (present on success)
    pub data: Option<T>,
    /// Optional success message
    pub message: Option<String>,
    /// Error message (present on error)
    pub error: Option<String>,
}

impl<T> ApiResponse<T> {
    /// Create a successful response with data
    /// 
    /// # Arguments
    /// * `data` - The response data to include
    /// 
    /// # Returns
    /// ApiResponse marked as successful with the provided data
    pub fn success(data: T) -> Self {
        Self {
            status: "success".to_string(),
            data: Some(data),
            message: None,
            error: None,
        }
    }

    /// Create an error response with message
    /// 
    /// # Arguments
    /// * `message` - Error message describing what went wrong
    /// 
    /// # Returns
    /// ApiResponse marked as error with the provided message
    #[allow(dead_code)]
    pub fn error(message: String) -> Self {
        Self {
            status: "error".to_string(),
            data: None,
            message: None,
            error: Some(message),
        }
    }
}

/// Authenticated user information
/// 
/// Contains user data returned after successful authentication.
/// Used in API responses and for session management.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthUser {
    /// Unique user identifier
    pub id: String,
    /// User's email address (optional for anonymous users)
    pub email: Option<String>,
    /// Whether this is an anonymous/guest user
    pub is_anonymous: bool,
    /// Account creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Web search result item
/// 
/// Represents a single search result from web search providers.
/// Used to enhance AI responses with current information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// Title of the search result
    pub title: String,
    /// URL of the search result
    pub url: String,
    /// Text snippet/excerpt from the content
    pub snippet: String,
    /// Relevance score (provider-dependent, optional)
    pub score: Option<f32>,
}

/// Web search response container
/// 
/// Contains search results from web search operations along with
/// metadata about the search query and performance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResponse {
    /// Original search query
    pub query: String,
    /// List of search results
    pub results: Vec<SearchResult>,
    /// Search provider that generated results
    pub provider: String,
    /// Search execution time in milliseconds
    pub took_ms: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;
    use chrono::Utc;

    #[test]
    fn test_provider_serialization() {
        assert_eq!(serde_json::to_string(&Provider::OpenAI).unwrap(), "\"openai\"");
        assert_eq!(serde_json::to_string(&Provider::Anthropic).unwrap(), "\"anthropic\"");
        assert_eq!(serde_json::to_string(&Provider::Mistral).unwrap(), "\"mistral\"");
        assert_eq!(serde_json::to_string(&Provider::Cloudflare).unwrap(), "\"cf\"");
        assert_eq!(serde_json::to_string(&Provider::Xai).unwrap(), "\"xai\"");
        assert_eq!(serde_json::to_string(&Provider::Groq).unwrap(), "\"groq\"");
    }

    #[test]
    fn test_provider_deserialization() {
        assert_eq!(serde_json::from_str::<Provider>("\"openai\"").unwrap(), Provider::OpenAI);
        assert_eq!(serde_json::from_str::<Provider>("\"anthropic\"").unwrap(), Provider::Anthropic);
        assert_eq!(serde_json::from_str::<Provider>("\"mistral\"").unwrap(), Provider::Mistral);
        assert_eq!(serde_json::from_str::<Provider>("\"cf\"").unwrap(), Provider::Cloudflare);
        assert_eq!(serde_json::from_str::<Provider>("\"xai\"").unwrap(), Provider::Xai);
        assert_eq!(serde_json::from_str::<Provider>("\"groq\"").unwrap(), Provider::Groq);
    }

    #[test]
    fn test_message_role_serialization() {
        assert_eq!(serde_json::to_string(&MessageRole::System).unwrap(), "\"system\"");
        assert_eq!(serde_json::to_string(&MessageRole::User).unwrap(), "\"user\"");
        assert_eq!(serde_json::to_string(&MessageRole::Assistant).unwrap(), "\"assistant\"");
    }

    #[test]
    fn test_operation_serialization() {
        assert_eq!(serde_json::to_string(&Operation::Chat).unwrap(), "\"chat\"");
        assert_eq!(serde_json::to_string(&Operation::Fim).unwrap(), "\"fim\"");
    }

    #[test]
    fn test_route_target_creation() {
        let route = RouteTarget {
            provider: Provider::OpenAI,
            model: "gpt-4o-mini".to_string(),
        };
        
        assert_eq!(route.provider, Provider::OpenAI);
        assert_eq!(route.model, "gpt-4o-mini");
    }

    #[test]
    fn test_route_target_serialization() {
        let route = RouteTarget {
            provider: Provider::Anthropic,
            model: "claude-3-5-sonnet".to_string(),
        };
        
        let json = serde_json::to_string(&route).unwrap();
        let deserialized: RouteTarget = serde_json::from_str(&json).unwrap();
        
        assert_eq!(deserialized.provider, Provider::Anthropic);
        assert_eq!(deserialized.model, "claude-3-5-sonnet");
    }

    #[test]
    fn test_chat_message_creation() {
        let message = ChatMessage {
            role: MessageRole::User,
            content: "Hello, AI!".to_string(),
        };
        
        assert_eq!(message.role, MessageRole::User);
        assert_eq!(message.content, "Hello, AI!");
    }

    #[test]
    fn test_chat_message_serialization() {
        let message = ChatMessage {
            role: MessageRole::Assistant,
            content: "Hello! How can I help you today?".to_string(),
        };
        
        let json = serde_json::to_string(&message).unwrap();
        let deserialized: ChatMessage = serde_json::from_str(&json).unwrap();
        
        assert_eq!(deserialized.role, MessageRole::Assistant);
        assert_eq!(deserialized.content, "Hello! How can I help you today?");
    }

    #[test]
    fn test_attachment_creation() {
        let attachment = Attachment {
            name: "test.txt".to_string(),
            url: "data:text/plain;base64,SGVsbG8gV29ybGQ=".to_string(),
            content_type: "text/plain".to_string(),
            size: Some(11),
        };
        
        assert_eq!(attachment.name, "test.txt");
        assert_eq!(attachment.content_type, "text/plain");
        assert!(attachment.url.contains("SGVsbG8gV29ybGQ="));
    }

    #[test]
    fn test_attachment_serialization() {
        let attachment = Attachment {
            name: "document.pdf".to_string(),
            url: "data:application/pdf;base64,base64encodeddata".to_string(),
            content_type: "application/pdf".to_string(),
            size: None,
        };
        
        let json = serde_json::to_string(&attachment).unwrap();
        let deserialized: Attachment = serde_json::from_str(&json).unwrap();
        
        assert_eq!(deserialized.name, attachment.name);
        assert_eq!(deserialized.content_type, attachment.content_type);
        assert_eq!(deserialized.url, attachment.url);
    }

    #[test]
    fn test_invoke_request_minimal() {
        let mut input = HashMap::new();
        input.insert("messages".to_string(), serde_json::json!([
            {
                "role": "user",
                "content": "Hello"
            }
        ]));
        
        let request = InvokeRequest {
            op: Operation::Chat,
            tier: Some("fast".to_string()),
            input,
            options: None,
            token: None,
            enable_search: None,
            attachments: None,
        };
        
        assert_eq!(request.op, Operation::Chat);
        assert_eq!(request.tier, Some("fast".to_string()));
        assert!(request.input.contains_key("messages"));
        assert!(request.attachments.is_none());
        assert!(request.options.is_none());
    }

    #[test]
    fn test_invoke_request_complete() {
        let mut input = HashMap::new();
        input.insert("messages".to_string(), serde_json::json!([
            {
                "role": "system",
                "content": "You are a helpful coding assistant."
            },
            {
                "role": "user",
                "content": "Complete this function"
            }
        ]));
        
        let request = InvokeRequest {
            op: Operation::Fim,
            tier: Some("smart".to_string()),
            input,
            options: Some(InvokeOptions {
                temperature: Some(0.7),
                max_tokens: Some(1000),
            }),
            token: None,
            enable_search: None,
            attachments: Some(vec![
                Attachment {
                    name: "code.py".to_string(),
                    url: "data:text/plain;base64,ZGVmIGhlbGxvKCk6".to_string(),
                    content_type: "text/plain".to_string(),
                    size: None,
                }
            ]),
        };
        
        assert_eq!(request.op, Operation::Fim);
        assert_eq!(request.tier, Some("smart".to_string()));
        assert!(request.input.contains_key("messages"));
        assert!(request.attachments.is_some());
        assert_eq!(request.attachments.as_ref().unwrap().len(), 1);
        assert_eq!(request.options.as_ref().unwrap().temperature, Some(0.7));
        assert_eq!(request.options.as_ref().unwrap().max_tokens, Some(1000));
    }

    #[test]
    fn test_invoke_request_serialization() {
        let mut input = HashMap::new();
        input.insert("messages".to_string(), serde_json::json!([
            {
                "role": "user",
                "content": "Test message"
            }
        ]));
        
        let request = InvokeRequest {
            op: Operation::Chat,
            tier: Some("fast".to_string()),
            input,
            options: Some(InvokeOptions {
                temperature: Some(0.8),
                max_tokens: Some(500),
            }),
            token: None,
            enable_search: None,
            attachments: None,
        };
        
        let json = serde_json::to_string(&request).unwrap();
        let deserialized: InvokeRequest = serde_json::from_str(&json).unwrap();
        
        assert_eq!(deserialized.op, request.op);
        assert_eq!(deserialized.tier, request.tier);
        assert!(deserialized.input.contains_key("messages"));
        assert_eq!(deserialized.options.as_ref().unwrap().temperature, Some(0.8));
        assert_eq!(deserialized.options.as_ref().unwrap().max_tokens, Some(500));
    }

    #[test]
    fn test_api_response_success() {
        let response = ApiResponse::success(serde_json::json!({"result": "success"}));
        
        assert_eq!(response.status, "success");
        assert!(response.data.is_some());
        assert!(response.message.is_none());
        assert!(response.error.is_none());
    }

    #[test]
    fn test_api_response_success_with_message() {
        let mut response = ApiResponse::success(serde_json::json!({"id": "123"}));
        response.message = Some("Operation completed successfully".to_string());
        
        assert_eq!(response.status, "success");
        assert!(response.data.is_some());
        assert_eq!(response.message.as_ref().unwrap(), "Operation completed successfully");
        assert!(response.error.is_none());
    }

    #[test]
    fn test_api_response_error() {
        let response = ApiResponse::<()>::error("Something went wrong".to_string());
        
        assert_eq!(response.status, "error");
        assert!(response.data.is_none());
        assert!(response.message.is_none());
        assert_eq!(response.error.as_ref().unwrap(), "Something went wrong");
    }

    #[test]
    fn test_api_response_serialization() {
        let mut response = ApiResponse::success(
            serde_json::json!({"user_id": "user_123", "name": "John Doe"})
        );
        response.message = Some("User retrieved successfully".to_string());
        
        let json = serde_json::to_string(&response).unwrap();
        let deserialized: ApiResponse<serde_json::Value> = serde_json::from_str(&json).unwrap();
        
        assert_eq!(deserialized.status, "success");
        assert!(deserialized.data.is_some());
        assert_eq!(deserialized.message.as_ref().unwrap(), "User retrieved successfully");
        assert!(deserialized.error.is_none());
        
        // Check data content
        let data = deserialized.data.unwrap();
        assert_eq!(data["user_id"], "user_123");
        assert_eq!(data["name"], "John Doe");
    }

    #[test]
    fn test_auth_user_creation() {
        let user = AuthUser {
            id: "user_123".to_string(),
            email: Some("test@example.com".to_string()),
            is_anonymous: false,
            created_at: Utc::now(),
        };
        
        assert_eq!(user.id, "user_123");
        assert_eq!(user.email.as_ref().unwrap(), "test@example.com");
        assert!(!user.is_anonymous);
    }

    #[test]
    fn test_auth_user_anonymous() {
        let user = AuthUser {
            id: "anon_456".to_string(),
            email: None,
            is_anonymous: true,
            created_at: Utc::now(),
        };
        
        assert_eq!(user.id, "anon_456");
        assert!(user.email.is_none());
        assert!(user.is_anonymous);
    }

    #[test]
    fn test_auth_user_serialization() {
        let user = AuthUser {
            id: "user_789".to_string(),
            email: Some("user@test.com".to_string()),
            is_anonymous: false,
            created_at: chrono::DateTime::parse_from_rfc3339("2024-01-01T00:00:00Z").unwrap().with_timezone(&Utc),
        };
        
        let json = serde_json::to_string(&user).unwrap();
        let deserialized: AuthUser = serde_json::from_str(&json).unwrap();
        
        assert_eq!(deserialized.id, user.id);
        assert_eq!(deserialized.email, user.email);
        assert_eq!(deserialized.is_anonymous, user.is_anonymous);
    }

    #[test]
    fn test_search_result_creation() {
        let result = SearchResult {
            title: "Test Article".to_string(),
            url: "https://example.com/article".to_string(),
            snippet: "This is a test article snippet with relevant information.".to_string(),
            score: Some(0.95),
        };
        
        assert_eq!(result.title, "Test Article");
        assert_eq!(result.url, "https://example.com/article");
        assert_eq!(result.snippet, "This is a test article snippet with relevant information.");
        assert_eq!(result.score, Some(0.95));
    }

    #[test]
    fn test_search_result_without_score() {
        let result = SearchResult {
            title: "Another Article".to_string(),
            url: "https://example.com/another".to_string(),
            snippet: "Another snippet".to_string(),
            score: None,
        };
        
        assert!(result.score.is_none());
    }

    #[test]
    fn test_search_result_serialization() {
        let result = SearchResult {
            title: "Serialization Test".to_string(),
            url: "https://test.com/serialize".to_string(),
            snippet: "Testing serialization functionality".to_string(),
            score: Some(0.88),
        };
        
        let json = serde_json::to_string(&result).unwrap();
        let deserialized: SearchResult = serde_json::from_str(&json).unwrap();
        
        assert_eq!(deserialized.title, result.title);
        assert_eq!(deserialized.url, result.url);
        assert_eq!(deserialized.snippet, result.snippet);
        assert_eq!(deserialized.score, result.score);
    }

    #[test]
    fn test_search_response_creation() {
        let response = SearchResponse {
            query: "rust programming".to_string(),
            results: vec![
                SearchResult {
                    title: "Rust Programming Language".to_string(),
                    url: "https://rust-lang.org".to_string(),
                    snippet: "A systems programming language focused on safety, speed, and concurrency.".to_string(),
                    score: Some(0.98),
                },
                SearchResult {
                    title: "Learn Rust".to_string(),
                    url: "https://doc.rust-lang.org/book/".to_string(),
                    snippet: "The Rust Programming Language book teaches you Rust.".to_string(),
                    score: Some(0.92),
                }
            ],
            provider: "tavily".to_string(),
            took_ms: 245,
        };
        
        assert_eq!(response.query, "rust programming");
        assert_eq!(response.results.len(), 2);
        assert_eq!(response.provider, "tavily");
        assert_eq!(response.took_ms, 245);
    }

    #[test]
    fn test_search_response_empty() {
        let response = SearchResponse {
            query: "nonexistent query".to_string(),
            results: vec![],
            provider: "brave".to_string(),
            took_ms: 123,
        };
        
        assert!(response.results.is_empty());
    }

    #[test]
    fn test_search_response_serialization() {
        let response = SearchResponse {
            query: "test search".to_string(),
            results: vec![
                SearchResult {
                    title: "Test Result".to_string(),
                    url: "https://test.example.com".to_string(),
                    snippet: "Test snippet content".to_string(),
                    score: Some(0.85),
                }
            ],
            provider: "searxng".to_string(),
            took_ms: 456,
        };
        
        let json = serde_json::to_string(&response).unwrap();
        let deserialized: SearchResponse = serde_json::from_str(&json).unwrap();
        
        assert_eq!(deserialized.query, response.query);
        assert_eq!(deserialized.results.len(), 1);
        assert_eq!(deserialized.provider, response.provider);
        assert_eq!(deserialized.took_ms, response.took_ms);
        assert_eq!(deserialized.results[0].title, "Test Result");
    }

    #[test]
    fn test_enum_edge_cases() {
        // Test that enums handle case variations properly in deserialization
        let provider_json = "\"OPENAI\"";
        let result = serde_json::from_str::<Provider>(provider_json);
        // This should fail because our enum expects lowercase
        assert!(result.is_err());
        
        // Test proper case
        let provider_json = "\"openai\"";
        let provider = serde_json::from_str::<Provider>(provider_json).unwrap();
        assert_eq!(provider, Provider::OpenAI);
    }
}