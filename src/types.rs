use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: MessageRole,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    System,
    User,
    Assistant,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct InvokeOptions {
    #[validate(range(min = 0.0, max = 2.0))]
    pub temperature: Option<f32>,
    #[validate(range(min = 1))]
    pub max_tokens: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attachment {
    pub name: String,
    pub url: String,
    pub content_type: String,
    pub size: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct InvokeRequest {
    pub op: Operation,
    pub tier: Option<String>,
    pub input: HashMap<String, serde_json::Value>,
    pub options: Option<InvokeOptions>,
    pub token: Option<String>,
    pub enable_search: Option<bool>,
    pub attachments: Option<Vec<Attachment>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Operation {
    Chat,
    Fim,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Provider {
    #[serde(rename = "cf")]
    Cloudflare,
    Mistral,
    OpenAI,
    #[serde(rename = "xai")]
    Xai,
    Groq,
    OpenRouter,
    Meta,
    Anthropic,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteTarget {
    pub provider: Provider,
    pub model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub status: String,
    pub data: Option<T>,
    pub message: Option<String>,
    pub error: Option<String>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            status: "success".to_string(),
            data: Some(data),
            message: None,
            error: None,
        }
    }

    pub fn error(message: String) -> Self {
        Self {
            status: "error".to_string(),
            data: None,
            message: None,
            error: Some(message),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthUser {
    pub id: String,
    pub email: Option<String>,
    pub is_anonymous: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub title: String,
    pub url: String,
    pub snippet: String,
    pub score: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResponse {
    pub query: String,
    pub results: Vec<SearchResult>,
    pub provider: String,
    pub took_ms: u64,
}