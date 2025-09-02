use anyhow::Result;
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
    #[allow(dead_code)]
    memory_users: HashMap<String, ConvexUser>, // key: email -> user
}

#[allow(dead_code)]
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Config, ConvexConfig};
    
    fn create_test_config(enabled: bool) -> Config {
        let mut config = Config::from_env();
        config.convex = ConvexConfig {
            url: if enabled { "https://test.convex.dev".to_string() } else { "".to_string() },
            enabled,
        };
        config
    }
    
    #[test]
    fn test_convex_service_new() {
        let config = create_test_config(true);
        let service = ConvexService::new(config.clone());
        
        // Should create service regardless of configuration
        assert_eq!(service.config.convex.enabled, true);
        assert_eq!(service.config.convex.url, "https://test.convex.dev");
    }
    
    #[test]
    fn test_convex_service_disabled() {
        let config = create_test_config(false);
        let service = ConvexService::new(config);
        
        assert_eq!(service.config.convex.enabled, false);
        assert_eq!(service.config.convex.url, "");
    }
    
    #[tokio::test]
    async fn test_log_api_request_disabled() {
        let config = create_test_config(false);
        let service = ConvexService::new(config);
        
        let event = ApiRequestEvent {
            request_id: "test_123".to_string(),
            user_id: Some("user_456".to_string()),
            operation: "chat".to_string(),
            tier: "fast".to_string(),
            provider: "openai".to_string(),
            model: "gpt-4o-mini".to_string(),
            temperature: Some(0.7),
            max_tokens: Some(1000),
            response_status: 200,
            response_time_ms: 1500,
            input_messages: Some(2),
            input_tokens: Some(200),
            output_tokens: Some(300),
            error_message: None,
            user_agent: None,
            ip_address: None,
        };
        
        // Should not fail when Convex is disabled
        let result = service.log_api_request(event).await;
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_log_usage_disabled() {
        let config = create_test_config(false);
        let service = ConvexService::new(config);
        
        let event = UsageEvent {
            user_id: Some("user_123".to_string()),
            operation: "chat".to_string(),
            provider: "anthropic".to_string(),
            model: "claude-3.5-sonnet".to_string(),
            input_tokens: 100,
            output_tokens: 200,
            cost_usd: Some(0.005),
        };
        
        let result = service.log_usage(event).await;
        assert!(result.is_ok());
    }
    
    #[tokio::test] 
    async fn test_log_message_disabled() {
        let config = create_test_config(false);
        let service = ConvexService::new(config);
        
        let event = MessageEvent {
            request_id: "req_789".to_string(),
            user_id: Some("user_abc".to_string()),
            chat_id: Some("chat_def".to_string()),
            message_type: "user".to_string(),
            content: "Hello, world!".to_string(),
            provider: None,
            model: None,
            token_count: None,
            created_at: None,
            attachments: None,
        };
        
        let result = service.log_message(event).await;
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_get_user_disabled() {
        let config = create_test_config(false);
        let service = ConvexService::new(config);
        
        let result = service.get_user("test@example.com").await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_none()); // Should return None when disabled
    }
    
    #[tokio::test]
    async fn test_create_user_disabled() {
        let config = create_test_config(false);
        let service = ConvexService::new(config);
        
        let user = UserAccount {
            email: "test@example.com".to_string(),
            password_hash: "hash123".to_string(),
            subscription_tier: "free".to_string(),
            api_key: "key123".to_string(),
            is_active: true,
        };
        
        let result = service.create_user(user).await;
        assert!(result.is_ok());
        assert!(!result.unwrap().is_empty()); // Should return non-empty string when disabled
    }
    
    #[tokio::test]
    async fn test_log_system_event_disabled() {
        let config = create_test_config(false);
        let service = ConvexService::new(config);
        
        let result = service.log_system_event(
            "test_event",
            "info", 
            "Test message",
            Some("user_123"),
            None
        ).await;
        
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_update_user_usage_disabled() {
        let config = create_test_config(false);
        let service = ConvexService::new(config);
        
        let result = service.update_user_usage(
            "user_123",
            10,
            20
        ).await;
        
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_create_chat_disabled() {
        let config = create_test_config(false);
        let service = ConvexService::new(config);
        
        let result = service.create_chat(
            "user_123",
            Some("Test Chat Title")
        ).await;
        
        assert!(result.is_ok());
        assert!(!result.unwrap().is_empty());
    }
    
    #[tokio::test]
    async fn test_get_user_chats_disabled() {
        let config = create_test_config(false);
        let service = ConvexService::new(config);
        
        let result = service.get_user_chats("user_123").await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }
    
    #[tokio::test]
    async fn test_delete_chat_disabled() {
        let config = create_test_config(false);
        let service = ConvexService::new(config);
        
        let result = service.delete_chat("chat_123", "user_456").await;
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_convex_user_serialization() {
        let user = ConvexUser {
            id: "user_123".to_string(),
            email: "test@example.com".to_string(),
            password_hash: "bcrypt_hash_here".to_string(),
            subscription_tier: "premium".to_string(),
            api_key: "key123".to_string(),
            is_active: true,
            created_at: Some(chrono::DateTime::parse_from_rfc3339("2021-01-01T00:00:00Z").unwrap().with_timezone(&chrono::Utc)),
        };
        
        // Test serialization
        let json = serde_json::to_string(&user).unwrap();
        assert!(json.contains("user_123"));
        assert!(json.contains("test@example.com"));
        assert!(json.contains("premium"));
        
        // Test deserialization
        let deserialized: ConvexUser = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, user.id);
        assert_eq!(deserialized.email, user.email);
        assert_eq!(deserialized.subscription_tier, user.subscription_tier);
        assert_eq!(deserialized.is_active, user.is_active);
    }
    
    #[test]
    fn test_api_request_event_serialization() {
        let event = ApiRequestEvent {
            request_id: "req_abc123".to_string(),
            user_id: Some("user_456".to_string()),
            operation: "chat".to_string(),
            tier: "smart".to_string(),
            provider: "openai".to_string(),
            model: "gpt-4o".to_string(),
            temperature: Some(0.8),
            max_tokens: Some(2000),
            response_status: 200,
            response_time_ms: 3000,
            input_messages: Some(3),
            input_tokens: Some(500),
            output_tokens: Some(1000),
            error_message: None,
            user_agent: None,
            ip_address: None,
        };
        
        let json = serde_json::to_string(&event).unwrap();
        let deserialized: ApiRequestEvent = serde_json::from_str(&json).unwrap();
        
        assert_eq!(deserialized.request_id, event.request_id);
        assert_eq!(deserialized.operation, event.operation);
        assert_eq!(deserialized.provider, event.provider);
        assert_eq!(deserialized.temperature, event.temperature);
    }
    
    #[test]
    fn test_usage_event_serialization() {
        let event = UsageEvent {
            user_id: Some("user_789".to_string()),
            operation: "fim".to_string(),
            provider: "mistral".to_string(),
            model: "codestral".to_string(),
            input_tokens: 200,
            output_tokens: 100,
            cost_usd: Some(0.0025),
        };
        
        let json = serde_json::to_string(&event).unwrap();
        let deserialized: UsageEvent = serde_json::from_str(&json).unwrap();
        
        assert_eq!(deserialized.user_id, event.user_id);
        assert_eq!(deserialized.operation, event.operation);
        assert_eq!(deserialized.input_tokens, event.input_tokens);
        assert_eq!(deserialized.cost_usd, event.cost_usd);
    }
}