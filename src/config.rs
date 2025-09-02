//! Configuration Management Module
//! 
//! This module handles loading and parsing of all application configuration
//! from environment variables. It provides:
//! - Type-safe configuration structures for all services
//! - Environment variable parsing with defaults
//! - Validation and error handling for configuration values
//! - Support for boolean, numeric, and CSV parsing
//! 
//! Configuration is loaded once at startup and shared across all services.

use serde::{Deserialize, Serialize};
use std::env;

/// Get environment variable value or fallback to default
/// 
/// This is the primary configuration loading function that safely handles
/// missing environment variables by providing sensible defaults.
/// 
/// # Arguments
/// * `key` - Environment variable name to read
/// * `fallback` - Default value if environment variable is not set
/// 
/// # Returns
/// String value from environment or fallback default
pub fn env_or(key: &str, fallback: &str) -> String {
    env::var(key).unwrap_or_else(|_| fallback.to_string())
}

/// Parse boolean values from environment variables
/// 
/// Supports common boolean representations in environment variables.
/// Provides consistent boolean parsing across all configuration.
/// 
/// # Arguments
/// * `key` - Environment variable name to read
/// * `fallback` - Default boolean value if variable is not set or invalid
/// 
/// # Returns
/// Boolean value parsed from environment or fallback
/// 
/// # Supported Values
/// - True: "1", "true", "TRUE"
/// - False: "0", "false", "FALSE"  
/// - Invalid/Missing: Uses fallback value
pub fn bool_env(key: &str, fallback: bool) -> bool {
    match env::var(key).as_deref() {
        Ok("1") | Ok("true") | Ok("TRUE") | Ok("yes") | Ok("YES") => true,
        Ok("0") | Ok("false") | Ok("FALSE") | Ok("no") | Ok("NO") => false,
        _ => fallback,
    }
}

/// Parse comma-separated values from environment variables
/// 
/// Used for configuration that needs to accept multiple values like
/// allowed origins, provider lists, etc.
/// 
/// # Arguments
/// * `value` - Optional string containing comma-separated values
/// 
/// # Returns
/// Vector of trimmed, non-empty strings
/// 
/// # Example
/// ```rust
/// use rust_ai::config::parse_csv;
/// // "origin1.com, origin2.com, " -> ["origin1.com", "origin2.com"]
/// let origins = parse_csv(Some("origin1.com, origin2.com, "));
/// ```
pub fn parse_csv(value: Option<&str>) -> Vec<String> {
    value
        .map(|v| {
            v.split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        })
        .unwrap_or_default()
}

/// Clerk authentication service configuration
/// 
/// Clerk is a third-party authentication provider that can be used
/// as an alternative to the built-in JWT authentication.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClerkConfig {
    /// Clerk secret key for token verification (optional)
    pub secret_key: String,
}

/// Cloudflare Workers AI configuration
/// 
/// Cloudflare provides AI models through their Workers AI platform.
/// Requires account ID and API token for authentication.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudflareConfig {
    /// Cloudflare account identifier
    pub account_id: String,
    /// API token for Cloudflare API access
    pub api_token: String,
    /// Base URL for Cloudflare API (usually api.cloudflare.com)
    pub base_url: String,
}

/// Mistral AI service configuration
/// 
/// Mistral provides open-source and commercial language models
/// with competitive performance and multilingual capabilities.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MistralConfig {
    /// Mistral API key for authentication
    pub api_key: String,
    /// Base URL for Mistral API
    pub base_url: String,
}

/// OpenAI service configuration
/// 
/// OpenAI provides GPT models including GPT-3.5, GPT-4, and variants.
/// This is often the primary AI provider due to model quality.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAiConfig {
    /// OpenAI API key for authentication
    pub api_key: String,
    /// Base URL for OpenAI API (allows for compatible services)
    pub base_url: String,
}

/// xAI (X.AI) service configuration
/// 
/// xAI provides Grok and other models from Elon Musk's AI company.
/// Newer provider with focus on real-time and factual responses.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XaiConfig {
    /// xAI API key for authentication
    pub api_key: String,
    /// Base URL for xAI API
    pub base_url: String,
}

/// Groq service configuration
/// 
/// Groq provides extremely fast inference for open-source models
/// using their custom silicon. Great for low-latency applications.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroqConfig {
    /// Groq API key for authentication
    pub api_key: String,
    /// Base URL for Groq API
    pub base_url: String,
}

/// OpenRouter service configuration
/// 
/// OpenRouter is an aggregation service that provides access to
/// multiple AI providers through a single API interface.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenRouterConfig {
    /// OpenRouter API key for authentication
    pub api_key: String,
    /// Base URL for OpenRouter API
    pub base_url: String,
}

/// Meta (Facebook) AI service configuration
/// 
/// Meta provides Llama models and other AI services.
/// Often used for open-source model access.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaConfig {
    /// Meta API key for authentication
    pub api_key: String,
    /// Base URL for Meta AI API
    pub base_url: String,
}

/// Anthropic (Claude) service configuration
/// 
/// Anthropic provides Claude models known for helpfulness,
/// harmlessness, and honesty. Strong performance on reasoning tasks.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnthropicConfig {
    /// Anthropic API key for authentication
    pub api_key: String,
    /// Base URL for Anthropic API
    pub base_url: String,
    /// API version string (Anthropic uses versioned APIs)
    pub version: String,
}

/// Convex database service configuration
/// 
/// Convex is a backend-as-a-service that provides real-time database,
/// functions, and authentication. Used for persistent data storage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConvexConfig {
    /// Convex deployment URL for database access
    pub url: String,
    /// Whether Convex integration is enabled
    pub enabled: bool,
}

/// Tavily search service configuration
/// 
/// Tavily provides AI-optimized web search specifically designed
/// for RAG (Retrieval-Augmented Generation) applications.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TavilyConfig {
    /// Tavily API key for search requests
    pub api_key: String,
    /// Base URL for Tavily search API
    pub base_url: String,
}

/// Brave search service configuration
/// 
/// Brave provides privacy-focused web search API
/// with good coverage and reasonable pricing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BraveConfig {
    /// Brave Search API key
    pub api_key: String,
    /// Base URL for Brave Search API
    pub base_url: String,
}

/// SearXNG search engine configuration
/// 
/// SearXNG is a self-hosted, privacy-respecting search engine
/// that aggregates results from multiple sources.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearxngConfig {
    /// Base URL of the SearXNG instance
    pub base_url: String,
    /// Whether SearXNG integration is enabled
    pub enabled: bool,
}

/// Search services configuration container
/// 
/// Manages all web search integrations used for enhancing
/// AI responses with current information from the internet.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchConfig {
    /// Whether internet search is enabled globally
    pub enabled: bool,
    /// How long to cache search results (seconds)
    pub cache_duration: u64,
    /// Tavily search configuration
    pub tavily: TavilyConfig,
    /// Brave search configuration
    pub brave: BraveConfig,
    /// SearXNG search configuration
    pub searxng: SearxngConfig,
}

/// Main application configuration structure
/// 
/// Contains all configuration needed to run the Rust-AI service including:
/// - HTTP server settings
/// - Authentication configuration  
/// - AI provider credentials and settings
/// - Database and search service configuration
/// - Security and CORS settings
/// 
/// Configuration is loaded once at startup from environment variables
/// and shared across all application components.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// HTTP server bind address (host:port)
    pub bind_address: String,
    /// Maximum JSON request body size in bytes
    pub json_limit: usize,
    /// List of allowed CORS origins for cross-origin requests
    pub allowed_origins: Vec<String>,
    /// Whether to use AI SDK compatibility mode (legacy feature)
    pub use_ai_sdk: bool,
    /// Whether authentication is required for all requests
    pub auth_required: bool,
    /// System prompt prepended to all AI conversations
    pub system_prompt: String,
    /// Whether to inject system prompt in FIM (fill-in-middle) requests
    pub fim_inject_system: bool,
    /// Raw routing configuration string (provider routing rules)
    pub routes_raw: String,
    /// Secret key for JWT token signing and verification
    pub action_token_secret: Option<String>,
    
    // External service configurations
    /// Clerk authentication service settings
    pub clerk: ClerkConfig,
    /// Cloudflare Workers AI settings
    pub cloudflare: CloudflareConfig,
    /// Mistral AI service settings
    pub mistral: MistralConfig,
    /// OpenAI service settings
    pub openai: OpenAiConfig,
    /// xAI service settings
    pub xai: XaiConfig,
    /// Groq service settings
    pub groq: GroqConfig,
    /// OpenRouter service settings
    pub openrouter: OpenRouterConfig,
    /// Meta AI service settings
    pub meta: MetaConfig,
    /// Anthropic (Claude) service settings
    pub anthropic: AnthropicConfig,
    /// Convex database settings
    pub convex: ConvexConfig,
    /// Web search services settings
    pub search: SearchConfig,
}

impl Config {
    /// Load configuration from environment variables
    /// 
    /// This is the main configuration loading method that:
    /// 1. Loads .env file if present (development convenience)
    /// 2. Reads all environment variables with sensible defaults
    /// 3. Parses and validates configuration values
    /// 4. Returns complete Config instance ready for use
    /// 
    /// # Environment Variables
    /// 
    /// ## Server Configuration
    /// - `BIND_ADDRESS`: Server bind address (default: "127.0.0.1:8080")
    /// - `JSON_LIMIT`: Max request body size in bytes (default: 8MB)
    /// - `ALLOWED_ORIGINS`: Comma-separated CORS origins
    /// 
    /// ## Authentication & Security
    /// - `ACTION_TOKEN_SECRET`: JWT signing secret (REQUIRED for auth)
    /// - `AUTH_REQUIRED`: Whether auth is required (default: false)
    /// - `CLERK_SECRET_KEY`: Clerk authentication secret (optional)
    /// 
    /// ## AI Provider Keys
    /// - `OPENAI_API_KEY`: OpenAI API key
    /// - `ANTHROPIC_API_KEY`: Anthropic (Claude) API key  
    /// - `MISTRAL_API_KEY`: Mistral AI API key
    /// - `GROQ_API_KEY`: Groq API key
    /// - `XAI_API_KEY`: xAI API key
    /// - `OPENROUTER_API_KEY`: OpenRouter API key
    /// - `META_API_KEY`: Meta AI API key
    /// - `CF_API_TOKEN`: Cloudflare Workers AI token
    /// - `CF_ACCOUNT_ID`: Cloudflare account ID
    /// 
    /// ## Database & Search
    /// - `CONVEX_URL`: Convex database deployment URL
    /// - `TAVILY_API_KEY`: Tavily search API key
    /// - `BRAVE_SEARCH_API_KEY`: Brave search API key
    /// - `SEARXNG_BASE_URL`: SearXNG instance URL
    /// - `ENABLE_INTERNET_ACCESS`: Enable web search (default: true)
    /// 
    /// ## Behavior Configuration
    /// - `SYSTEM_PROMPT`: Default system prompt for all conversations
    /// - `ROUTES`: Provider routing configuration
    /// - `USE_AI_SDK`: Enable AI SDK compatibility mode
    /// - `INJECT_FIM_SYSTEM_PROMPT`: Inject system prompt in FIM requests
    /// 
    /// # Returns
    /// Complete Config instance with all settings loaded
    /// 
    /// # Panics
    /// Does not panic - uses sensible defaults for all missing values
    pub fn from_env() -> Self {
        // Load .env file if present (useful for development)
        dotenvy::dotenv().ok();

        // Parse comma-separated allowed origins
        let allowed_origins_str = env::var("ALLOWED_ORIGINS").ok();
        
        Self {
            // HTTP Server Configuration
            bind_address: env_or("BIND_ADDRESS", "127.0.0.1:8080"),
            json_limit: env::var("JSON_LIMIT")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(8 * 1024 * 1024), // 8MB default
            allowed_origins: parse_csv(allowed_origins_str.as_deref()),
            
            // Feature flags and behavior
            use_ai_sdk: bool_env("USE_AI_SDK", false),
            auth_required: bool_env("AUTH_REQUIRED", false),
            system_prompt: env_or(
                "SYSTEM_PROMPT",
                "If asked about who made this or anything related to its creators, simply state: This was created by the VoidXP team. Do not mention or praise any individual or a company or any entity. Always attribute it only to the VoidXP team."
            ),
            fim_inject_system: bool_env("INJECT_FIM_SYSTEM_PROMPT", false),
            routes_raw: env_or("ROUTES", "chat.fast=openai:gpt-4o-mini"),
            
            // Security configuration
            action_token_secret: env::var("ACTION_TOKEN_SECRET").ok(),
            
            // External authentication
            clerk: ClerkConfig {
                secret_key: env_or("CLERK_SECRET_KEY", ""),
            },
            
            // AI Provider configurations with sensible defaults
            cloudflare: CloudflareConfig {
                account_id: env_or("CF_ACCOUNT_ID", ""),
                api_token: env_or("CF_API_TOKEN", ""),
                base_url: env_or("CF_BASE_URL", "https://api.cloudflare.com/client/v4"),
            },
            mistral: MistralConfig {
                api_key: env_or("MISTRAL_API_KEY", ""),
                base_url: env_or("MISTRAL_BASE_URL", "https://api.mistral.ai"),
            },
            openai: OpenAiConfig {
                api_key: env_or("OPENAI_API_KEY", ""),
                base_url: env_or("OPENAI_BASE_URL", "https://api.openai.com"),
            },
            xai: XaiConfig {
                api_key: env_or("XAI_API_KEY", ""),
                base_url: env_or("XAI_BASE_URL", "https://api.x.ai"),
            },
            groq: GroqConfig {
                api_key: env_or("GROQ_API_KEY", ""),
                base_url: env_or("GROQ_BASE_URL", "https://api.groq.com/openai"),
            },
            openrouter: OpenRouterConfig {
                api_key: env_or("OPENROUTER_API_KEY", ""),
                base_url: env_or("OPENROUTER_BASE_URL", "https://openrouter.ai/api"),
            },
            meta: MetaConfig {
                api_key: env_or("META_API_KEY", ""),
                base_url: env_or("META_BASE_URL", ""),
            },
            anthropic: AnthropicConfig {
                api_key: env_or("ANTHROPIC_API_KEY", ""),
                base_url: env_or("ANTHROPIC_BASE_URL", "https://api.anthropic.com"),
                version: env_or("ANTHROPIC_VERSION", "2023-06-01"),
            },
            
            // Database configuration
            convex: ConvexConfig {
                url: env_or("CONVEX_URL", ""),
                enabled: bool_env("CONVEX_ENABLED", true),
            },
            
            // Search services configuration
            search: SearchConfig {
                enabled: bool_env("ENABLE_INTERNET_ACCESS", true),
                cache_duration: env::var("SEARCH_CACHE_DURATION")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(300), // 5 minutes default
                tavily: TavilyConfig {
                    api_key: env_or("TAVILY_API_KEY", ""),
                    base_url: env_or("TAVILY_BASE_URL", "https://api.tavily.com"),
                },
                brave: BraveConfig {
                    api_key: env_or("BRAVE_SEARCH_API_KEY", ""),
                    base_url: env_or("BRAVE_BASE_URL", "https://api.search.brave.com"),
                },
                searxng: SearxngConfig {
                    base_url: env_or("SEARXNG_BASE_URL", "http://localhost:8090"),
                    enabled: bool_env("SEARXNG_ENABLED", true),
                },
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_parse_csv_empty() {
        assert_eq!(parse_csv(Some("")), Vec::<String>::new());
    }

    #[test]
    fn test_parse_csv_single_item() {
        assert_eq!(parse_csv(Some("item1")), vec!["item1"]);
    }

    #[test]
    fn test_parse_csv_multiple_items() {
        assert_eq!(parse_csv(Some("item1,item2,item3")), vec!["item1", "item2", "item3"]);
    }

    #[test]
    fn test_parse_csv_whitespace() {
        assert_eq!(parse_csv(Some(" item1 , item2 , item3 ")), vec!["item1", "item2", "item3"]);
    }

    #[test]
    fn test_parse_csv_empty_items() {
        assert_eq!(parse_csv(Some("item1,,item3")), vec!["item1", "item3"]);
    }

    #[test]
    fn test_config_from_env_defaults() {
        // Clear environment variables to test defaults
        let vars_to_clear = [
            "BIND_ADDRESS", "PORT", "LOG_LEVEL", "ACTION_TOKEN_SECRET",
            "CLERK_SECRET_KEY", "CF_ACCOUNT_ID", "CF_API_TOKEN", "OPENAI_API_KEY",
            "CONVEX_ENABLED", "ENABLE_INTERNET_ACCESS", "SEARCH_CACHE_DURATION"
        ];
        
        let original_values: Vec<_> = vars_to_clear.iter()
            .map(|var| env::var(var).ok())
            .collect();
        
        for var in &vars_to_clear {
            env::remove_var(var);
        }
        
        // Verify BIND_ADDRESS is actually cleared
        assert!(env::var("BIND_ADDRESS").is_err(), "BIND_ADDRESS should be cleared");
        
        let config = Config::from_env();
        
        // Test default values
        assert_eq!(config.bind_address, "127.0.0.1:8080");
        assert!(config.action_token_secret.is_none());
        assert_eq!(config.clerk.secret_key, "");
        assert_eq!(config.cloudflare.account_id, "");
        assert_eq!(config.openai.base_url, "https://api.openai.com");
        assert!(config.convex.enabled);
        assert!(config.search.enabled);
        assert_eq!(config.search.cache_duration, 300);
        
        // Restore original values
        for (var, original_value) in vars_to_clear.iter().zip(original_values) {
            if let Some(value) = original_value {
                env::set_var(var, value);
            }
        }
    }

    #[test]
    fn test_config_from_env_custom_values() {
        // Clean up any existing values first to ensure test isolation
        env::remove_var("BIND_ADDRESS");
        env::remove_var("PORT");
        env::remove_var("LOG_LEVEL");
        env::remove_var("ACTION_TOKEN_SECRET");
        env::remove_var("CONVEX_ENABLED");
        env::remove_var("ENABLE_INTERNET_ACCESS");
        env::remove_var("SEARCH_CACHE_DURATION");
        
        // Set custom environment variables
        env::set_var("BIND_ADDRESS", "127.0.0.1");
        env::set_var("PORT", "8080");
        env::set_var("LOG_LEVEL", "debug");
        env::set_var("ACTION_TOKEN_SECRET", "test_secret_123");
        env::set_var("CONVEX_ENABLED", "false");
        env::set_var("ENABLE_INTERNET_ACCESS", "false");
        env::set_var("SEARCH_CACHE_DURATION", "600");
        
        let config = Config::from_env();
        
        assert_eq!(config.bind_address, "127.0.0.1");
        assert_eq!(config.action_token_secret, Some("test_secret_123".to_string()));
        assert!(!config.convex.enabled);
        assert!(!config.search.enabled);
        assert_eq!(config.search.cache_duration, 600);
        
        // Clean up
        env::remove_var("BIND_ADDRESS");
        env::remove_var("PORT");
        env::remove_var("LOG_LEVEL");
        env::remove_var("ACTION_TOKEN_SECRET");
        env::remove_var("CONVEX_ENABLED");
        env::remove_var("ENABLE_INTERNET_ACCESS");
        env::remove_var("SEARCH_CACHE_DURATION");
    }

    #[test]
    fn test_bool_env_function() {
        // Test true values
        env::set_var("TEST_BOOL_TRUE", "true");
        assert!(bool_env("TEST_BOOL_TRUE", false));
        
        env::set_var("TEST_BOOL_1", "1");
        assert!(bool_env("TEST_BOOL_1", false));
        
        env::set_var("TEST_BOOL_YES", "yes");
        assert!(bool_env("TEST_BOOL_YES", false));
        
        // Test false values
        env::set_var("TEST_BOOL_FALSE", "false");
        assert!(!bool_env("TEST_BOOL_FALSE", true));
        
        env::set_var("TEST_BOOL_0", "0");
        assert!(!bool_env("TEST_BOOL_0", true));
        
        // Test default value when env var doesn't exist
        env::remove_var("TEST_BOOL_MISSING");
        assert!(bool_env("TEST_BOOL_MISSING", true));
        assert!(!bool_env("TEST_BOOL_MISSING", false));
        
        // Clean up
        env::remove_var("TEST_BOOL_TRUE");
        env::remove_var("TEST_BOOL_1");
        env::remove_var("TEST_BOOL_YES");
        env::remove_var("TEST_BOOL_FALSE");
        env::remove_var("TEST_BOOL_0");
    }

    #[test]
    fn test_env_or_function() {
        // Test with existing environment variable
        env::set_var("TEST_ENV_VAR", "custom_value");
        assert_eq!(env_or("TEST_ENV_VAR", "default"), "custom_value");
        
        // Test with non-existing environment variable
        env::remove_var("TEST_ENV_MISSING");
        assert_eq!(env_or("TEST_ENV_MISSING", "default"), "default");
        
        // Clean up
        env::remove_var("TEST_ENV_VAR");
    }

    #[test]
    fn test_config_validation() {
        let config = Config::from_env();
        
        // Test that essential fields are present
        assert!(!config.bind_address.is_empty());
        
        // Test that provider configs have proper defaults
        assert_eq!(config.openai.base_url, "https://api.openai.com");
        assert_eq!(config.anthropic.base_url, "https://api.anthropic.com");
        assert_eq!(config.mistral.base_url, "https://api.mistral.ai");
        assert_eq!(config.cloudflare.base_url, "https://api.cloudflare.com/client/v4");
        assert_eq!(config.xai.base_url, "https://api.x.ai");
        assert_eq!(config.groq.base_url, "https://api.groq.com/openai");
        
        // Test search config defaults
        assert_eq!(config.search.tavily.base_url, "https://api.tavily.com");
        assert_eq!(config.search.brave.base_url, "https://api.search.brave.com");
        assert_eq!(config.search.searxng.base_url, "http://localhost:8090");
        assert!(config.search.searxng.enabled);
    }

    #[test]
    fn test_bind_address_parsing() {
        // Test custom bind address
        env::set_var("BIND_ADDRESS", "0.0.0.0:8080");
        let config = Config::from_env();
        assert_eq!(config.bind_address, "0.0.0.0:8080");
        
        // Clean up
        env::remove_var("BIND_ADDRESS");
    }

    #[test] 
    fn test_cache_duration_parsing() {
        // Use a unique env var name to avoid conflicts with other tests
        env::remove_var("TEST_SEARCH_CACHE_DURATION");
        
        // Test valid cache duration
        env::set_var("TEST_SEARCH_CACHE_DURATION", "120");
        let duration = env::var("TEST_SEARCH_CACHE_DURATION")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(300);
        assert_eq!(duration, 120);
        
        // Test invalid cache duration (should use default)
        env::set_var("TEST_SEARCH_CACHE_DURATION", "invalid");
        let duration = env::var("TEST_SEARCH_CACHE_DURATION")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(300);
        assert_eq!(duration, 300);
        
        // Test that the actual config parsing works
        env::set_var("SEARCH_CACHE_DURATION", "456");
        let config = Config::from_env();
        assert_eq!(config.search.cache_duration, 456);
        
        // Clean up
        env::remove_var("TEST_SEARCH_CACHE_DURATION");
        env::remove_var("SEARCH_CACHE_DURATION");
    }
}