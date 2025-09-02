use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use uuid::Uuid;

use crate::config::Config;
use crate::types::Attachment;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiRequestEvent {
    pub request_id: String,
    pub user_id: Option<String>,
    pub operation: String, // 'chat' | 'fim'
    pub tier: String,
    pub provider: String,
    pub model: String,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub response_status: u16,
    pub response_time_ms: u64,
    pub input_messages: Option<u32>,
    pub input_tokens: Option<u32>,
    pub output_tokens: Option<u32>,
    pub error_message: Option<String>,
    pub user_agent: Option<String>,
    pub ip_address: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageEvent {
    pub user_id: Option<String>,
    pub provider: String,
    pub model: String,
    pub operation: String,
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub cost_usd: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageEvent {
    pub request_id: String,
    pub chat_id: Option<String>,
    pub user_id: Option<String>,
    pub message_type: String, // 'user' | 'assistant' | 'system'
    pub content: String,
    pub provider: Option<String>,
    pub model: Option<String>,
    pub token_count: Option<u32>,
    pub created_at: Option<i64>,
    pub attachments: Option<Vec<Attachment>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserAccount {
    pub email: String,
    pub password_hash: String,
    pub subscription_tier: String,
    pub api_key: String,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConvexUser {
    pub id: String,
    pub email: String,
    pub password_hash: String,
    pub subscription_tier: String,
    pub api_key: String,
    pub is_active: bool,
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemEvent {
    pub event_type: String,
    pub severity: String, // 'info' | 'warn' | 'error'
    pub message: String,
    pub metadata: Option<Value>,
    pub user_id: Option<String>,
    pub request_id: Option<String>,
}

#[derive(Clone)]
pub struct ConvexService {
    config: Config,
    // In-memory fallback store when Convex is disabled/unconfigured
    memory_users: HashMap<String, ConvexUser>, // key: email -> user
}

impl ConvexService {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            memory_users: HashMap::new(),
        }
    }

    pub async fn log_api_request(&self, event: ApiRequestEvent) -> Result<()> {
        if !self.config.convex.enabled || self.config.convex.url.is_empty() {
            return Ok(());
        }

        // TODO: Implement actual Convex HTTP client integration
        // For now, we'll just log to console
        tracing::info!("API Request Event: {:?}", event);
        Ok(())
    }

    pub async fn log_usage(&self, event: UsageEvent) -> Result<()> {
        if !self.config.convex.enabled {
            return Ok(());
        }

        // TODO: Implement actual Convex integration
        tracing::info!("Usage Event: {:?}", event);
        Ok(())
    }

    pub async fn log_message(&self, event: MessageEvent) -> Result<()> {
        if !self.config.convex.enabled {
            return Ok(());
        }

        // TODO: Implement actual Convex integration
        tracing::info!("Message Event: {:?}", event);
        Ok(())
    }

    pub async fn log_system_event(
        &self,
        event_type: &str,
        severity: &str,
        message: &str,
        user_id: Option<&str>,
        metadata: Option<Value>,
    ) -> Result<()> {
        let event = SystemEvent {
            event_type: event_type.to_string(),
            severity: severity.to_string(),
            message: message.to_string(),
            metadata,
            user_id: user_id.map(|s| s.to_string()),
            request_id: None,
        };

        if !self.config.convex.enabled {
            tracing::info!("System Event: {:?}", event);
            return Ok(());
        }

        // TODO: Implement actual Convex integration
        tracing::info!("System Event: {:?}", event);
        Ok(())
    }

    pub async fn create_user(&self, user_account: UserAccount) -> Result<String> {
        let user_id = Uuid::new_v4().to_string();
        
        if !self.config.convex.enabled {
            // Use in-memory storage as fallback
            let user = ConvexUser {
                id: user_id.clone(),
                email: user_account.email.clone(),
                password_hash: user_account.password_hash,
                subscription_tier: user_account.subscription_tier,
                api_key: user_account.api_key,
                is_active: user_account.is_active,
                created_at: Some(Utc::now()),
            };
            
            // Note: In a real implementation, we'd need to make this thread-safe
            // For now, this is just a placeholder
            tracing::info!("Creating user in memory store: {:?}", user);
            return Ok(user_id);
        }

        // TODO: Implement actual Convex integration
        tracing::info!("Creating user: {:?}", user_account);
        Ok(user_id)
    }

    pub async fn get_user(&self, email: &str) -> Result<Option<ConvexUser>> {
        if !self.config.convex.enabled {
            // Use in-memory storage as fallback
            // Note: This would return None in current implementation
            // as we're not actually storing in memory
            return Ok(None);
        }

        // TODO: Implement actual Convex integration
        tracing::info!("Getting user by email: {}", email);
        Ok(None)
    }

    pub async fn update_user_usage(
        &self,
        user_id: &str,
        input_tokens: u32,
        output_tokens: u32,
    ) -> Result<()> {
        if !self.config.convex.enabled {
            return Ok(());
        }

        // TODO: Implement actual Convex integration
        tracing::info!(
            "Updating user usage - user_id: {}, input_tokens: {}, output_tokens: {}",
            user_id, input_tokens, output_tokens
        );
        Ok(())
    }

    pub async fn get_analytics(
        &self,
        user_id: Option<&str>,
        timeframe_hours: Option<u32>,
    ) -> Result<Value> {
        if !self.config.convex.enabled {
            return Ok(serde_json::json!({
                "requests": 0,
                "tokens": 0,
                "errors": 0
            }));
        }

        // TODO: Implement actual Convex integration
        tracing::info!(
            "Getting analytics - user_id: {:?}, timeframe_hours: {:?}",
            user_id, timeframe_hours
        );
        
        Ok(serde_json::json!({
            "requests": 0,
            "tokens": 0,
            "errors": 0
        }))
    }

    pub async fn create_chat(
        &self,
        user_id: &str,
        title: Option<&str>,
    ) -> Result<String> {
        let chat_id = Uuid::new_v4().to_string();
        
        if !self.config.convex.enabled {
            return Ok(chat_id);
        }

        // TODO: Implement actual Convex integration
        tracing::info!(
            "Creating chat - user_id: {}, title: {:?}, chat_id: {}",
            user_id, title, chat_id
        );
        
        Ok(chat_id)
    }

    pub async fn get_user_chats(&self, user_id: &str) -> Result<Vec<Value>> {
        if !self.config.convex.enabled {
            return Ok(vec![]);
        }

        // TODO: Implement actual Convex integration
        tracing::info!("Getting user chats - user_id: {}", user_id);
        Ok(vec![])
    }

    pub async fn delete_chat(&self, chat_id: &str, user_id: &str) -> Result<()> {
        if !self.config.convex.enabled {
            return Ok(());
        }

        // TODO: Implement actual Convex integration
        tracing::info!("Deleting chat - chat_id: {}, user_id: {}", chat_id, user_id);
        Ok(())
    }
}