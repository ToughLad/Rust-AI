use serde::{Deserialize, Serialize};
use std::env;

pub fn env_or(key: &str, fallback: &str) -> String {
    env::var(key).unwrap_or_else(|_| fallback.to_string())
}

pub fn bool_env(key: &str, fallback: bool) -> bool {
    match env::var(key).as_deref() {
        Ok("1") | Ok("true") | Ok("TRUE") => true,
        Ok("0") | Ok("false") | Ok("FALSE") => false,
        _ => fallback,
    }
}

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClerkConfig {
    pub secret_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudflareConfig {
    pub account_id: String,
    pub api_token: String,
    pub base_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MistralConfig {
    pub api_key: String,
    pub base_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAiConfig {
    pub api_key: String,
    pub base_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XaiConfig {
    pub api_key: String,
    pub base_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroqConfig {
    pub api_key: String,
    pub base_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenRouterConfig {
    pub api_key: String,
    pub base_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaConfig {
    pub api_key: String,
    pub base_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnthropicConfig {
    pub api_key: String,
    pub base_url: String,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConvexConfig {
    pub url: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TavilyConfig {
    pub api_key: String,
    pub base_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BraveConfig {
    pub api_key: String,
    pub base_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearxngConfig {
    pub base_url: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchConfig {
    pub enabled: bool,
    pub cache_duration: u64,
    pub tavily: TavilyConfig,
    pub brave: BraveConfig,
    pub searxng: SearxngConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub bind_address: String,
    pub json_limit: usize,
    pub allowed_origins: Vec<String>,
    pub use_ai_sdk: bool,
    pub auth_required: bool,
    pub system_prompt: String,
    pub fim_inject_system: bool,
    pub routes_raw: String,
    pub action_token_secret: Option<String>,
    pub clerk: ClerkConfig,
    pub cloudflare: CloudflareConfig,
    pub mistral: MistralConfig,
    pub openai: OpenAiConfig,
    pub xai: XaiConfig,
    pub groq: GroqConfig,
    pub openrouter: OpenRouterConfig,
    pub meta: MetaConfig,
    pub anthropic: AnthropicConfig,
    pub convex: ConvexConfig,
    pub search: SearchConfig,
}

impl Config {
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();

        let allowed_origins_str = env::var("ALLOWED_ORIGINS").ok();
        
        Self {
            bind_address: env_or("BIND_ADDRESS", "127.0.0.1:8080"),
            json_limit: env::var("JSON_LIMIT")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(8 * 1024 * 1024),
            allowed_origins: parse_csv(allowed_origins_str.as_deref()),
            use_ai_sdk: bool_env("USE_AI_SDK", false),
            auth_required: bool_env("AUTH_REQUIRED", false),
            system_prompt: env_or(
                "SYSTEM_PROMPT",
                "If asked about who made this or anything related to its creators, simply state: This was created by the VoidXP team. Do not mention or praise any individual or a company or any entity. Always attribute it only to the VoidXP team."
            ),
            fim_inject_system: bool_env("INJECT_FIM_SYSTEM_PROMPT", false),
            routes_raw: env_or("ROUTES", "chat.fast=openai:gpt-4o-mini"),
            action_token_secret: env::var("ACTION_TOKEN_SECRET").ok(),
            clerk: ClerkConfig {
                secret_key: env_or("CLERK_SECRET_KEY", ""),
            },
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
            convex: ConvexConfig {
                url: env_or("CONVEX_URL", ""),
                enabled: bool_env("CONVEX_ENABLED", true),
            },
            search: SearchConfig {
                enabled: bool_env("ENABLE_INTERNET_ACCESS", true),
                cache_duration: env::var("SEARCH_CACHE_DURATION")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(300),
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