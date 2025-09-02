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
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
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