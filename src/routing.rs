use std::collections::HashMap;

use crate::types::{Provider, RouteTarget};

#[allow(dead_code)]
pub type RoutingMap = HashMap<String, RouteTarget>; // key = `${op}.${tier}`

#[allow(dead_code)]
pub fn build_routing(routes_raw: &str) -> RoutingMap {
    let mut map = HashMap::new();
    
    for pair in routes_raw.split(',') {
        let trimmed = pair.trim();
        if trimmed.is_empty() || !trimmed.contains('=') {
            continue;
        }
        
        let parts: Vec<&str> = trimmed.split('=').map(|s| s.trim()).collect();
        if parts.len() != 2 {
            continue;
        }
        
        let lhs_parts: Vec<&str> = parts[0].split('.').map(|s| s.trim()).collect();
        if lhs_parts.len() != 2 {
            continue;
        }
        
        let op = lhs_parts[0];
        let tier = lhs_parts[1];
        
        let (provider, model) = if parts[1].contains(':') {
            let rhs_parts: Vec<&str> = parts[1].split(':').collect();
            if rhs_parts.len() == 2 {
                (normalize_provider(rhs_parts[0].trim()), rhs_parts[1].trim().to_string())
            } else {
                continue; // Skip invalid formats
            }
        } else {
            continue; // Skip entries without colon
        };
        
        let key = format!("{}.{}", op, tier);
        map.insert(key, RouteTarget { provider, model });
    }
    
    map
}

#[allow(dead_code)]
pub fn resolve_route<'a>(map: &'a RoutingMap, op: &str, tier: &str) -> Option<&'a RouteTarget> {
    let key = format!("{}.{}", op, tier);
    map.get(&key)
}

#[allow(dead_code)]
fn normalize_provider(provider_str: &str) -> Provider {
    match provider_str.to_lowercase().as_str() {
        "cf" | "cloudflare" => Provider::Cloudflare,
        "mistral" => Provider::Mistral,
        "openai" => Provider::OpenAI,
        "xai" => Provider::Xai,
        "groq" => Provider::Groq,
        "openrouter" => Provider::OpenRouter,
        "meta" => Provider::Meta,
        "anthropic" => Provider::Anthropic,
        _ => Provider::OpenAI,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_routing() {
        let routes_raw = "chat.fast=openai:gpt-4o-mini,chat.smart=anthropic:claude-3-5-sonnet-20241022";
        let routing = build_routing(routes_raw);
        
        assert_eq!(routing.len(), 2);
        
        let fast_route = routing.get("chat.fast").unwrap();
        assert!(matches!(fast_route.provider, Provider::OpenAI));
        assert_eq!(fast_route.model, "gpt-4o-mini");
        
        let smart_route = routing.get("chat.smart").unwrap();
        assert!(matches!(smart_route.provider, Provider::Anthropic));
        assert_eq!(smart_route.model, "claude-3-5-sonnet-20241022");
    }

    #[test]
    fn test_resolve_route() {
        let routes_raw = "chat.fast=openai:gpt-4o-mini";
        let routing = build_routing(routes_raw);
        
        let resolved = resolve_route(&routing, "chat", "fast");
        assert!(resolved.is_some());
        
        let not_found = resolve_route(&routing, "chat", "nonexistent");
        assert!(not_found.is_none());
    }
    
    #[test]
    fn test_build_routing_empty() {
        let routing = build_routing("");
        assert_eq!(routing.len(), 0);
    }
    
    #[test]
    fn test_build_routing_single_route() {
        let routes_raw = "fim.fast=mistral:codestral";
        let routing = build_routing(routes_raw);
        
        assert_eq!(routing.len(), 1);
        let route = routing.get("fim.fast").unwrap();
        assert!(matches!(route.provider, Provider::Mistral));
        assert_eq!(route.model, "codestral");
    }
    
    #[test]
    fn test_build_routing_multiple_operations() {
        let routes_raw = "chat.fast=openai:gpt-4o-mini,chat.smart=anthropic:claude-3-5-sonnet,fim.fast=mistral:codestral,fim.smart=openai:gpt-4o";
        let routing = build_routing(routes_raw);
        
        assert_eq!(routing.len(), 4);
        
        // Test all routes exist
        assert!(routing.contains_key("chat.fast"));
        assert!(routing.contains_key("chat.smart"));
        assert!(routing.contains_key("fim.fast"));
        assert!(routing.contains_key("fim.smart"));
    }
    
    #[test]
    fn test_build_routing_invalid_formats() {
        // Missing colon in provider:model
        let routes_raw = "chat.fast=openai-gpt-4o-mini";
        let routing = build_routing(routes_raw);
        assert_eq!(routing.len(), 0); // Should skip invalid entries
        
        // Missing equals sign
        let routes_raw = "chat.fast-openai:gpt-4o-mini";
        let routing = build_routing(routes_raw);
        assert_eq!(routing.len(), 0);
        
        // Empty parts
        let routes_raw = "=openai:gpt-4o-mini,chat.fast=,=";
        let routing = build_routing(routes_raw);
        assert_eq!(routing.len(), 0);
    }
    
    #[test]
    fn test_build_routing_whitespace_handling() {
        let routes_raw = " chat.fast = openai : gpt-4o-mini , chat.smart = anthropic : claude-3-5-sonnet ";
        let routing = build_routing(routes_raw);
        
        assert_eq!(routing.len(), 2);
        
        let fast_route = routing.get("chat.fast").unwrap();
        assert!(matches!(fast_route.provider, Provider::OpenAI));
        assert_eq!(fast_route.model, "gpt-4o-mini");
        
        let smart_route = routing.get("chat.smart").unwrap();
        assert!(matches!(smart_route.provider, Provider::Anthropic));
        assert_eq!(smart_route.model, "claude-3-5-sonnet");
    }
    
    #[test]
    fn test_build_routing_duplicate_keys() {
        let routes_raw = "chat.fast=openai:gpt-4o-mini,chat.fast=anthropic:claude-3-5-sonnet";
        let routing = build_routing(routes_raw);
        
        assert_eq!(routing.len(), 1);
        // Should use the last occurrence
        let route = routing.get("chat.fast").unwrap();
        assert!(matches!(route.provider, Provider::Anthropic));
        assert_eq!(route.model, "claude-3-5-sonnet");
    }
    
    #[test]
    fn test_normalize_provider() {
        assert!(matches!(normalize_provider("openai"), Provider::OpenAI));
        assert!(matches!(normalize_provider("OpenAI"), Provider::OpenAI));
        assert!(matches!(normalize_provider("OPENAI"), Provider::OpenAI));
        
        assert!(matches!(normalize_provider("anthropic"), Provider::Anthropic));
        assert!(matches!(normalize_provider("Anthropic"), Provider::Anthropic));
        
        assert!(matches!(normalize_provider("mistral"), Provider::Mistral));
        assert!(matches!(normalize_provider("Mistral"), Provider::Mistral));
        
        assert!(matches!(normalize_provider("cloudflare"), Provider::Cloudflare));
        assert!(matches!(normalize_provider("xai"), Provider::Xai));
        assert!(matches!(normalize_provider("groq"), Provider::Groq));
        
        // Unknown providers should default to OpenAI
        assert!(matches!(normalize_provider("unknown"), Provider::OpenAI));
        assert!(matches!(normalize_provider(""), Provider::OpenAI));
        assert!(matches!(normalize_provider("google"), Provider::OpenAI));
    }
    
    #[test]
    fn test_resolve_route_comprehensive() {
        let routes_raw = "chat.fast=openai:gpt-4o-mini,chat.smart=anthropic:claude-3-5-sonnet,fim.fast=mistral:codestral";
        let routing = build_routing(routes_raw);
        
        // Valid resolutions
        let chat_fast = resolve_route(&routing, "chat", "fast");
        assert!(chat_fast.is_some());
        assert!(matches!(chat_fast.unwrap().provider, Provider::OpenAI));
        
        let chat_smart = resolve_route(&routing, "chat", "smart");
        assert!(chat_smart.is_some());
        assert!(matches!(chat_smart.unwrap().provider, Provider::Anthropic));
        
        let fim_fast = resolve_route(&routing, "fim", "fast");
        assert!(fim_fast.is_some());
        assert!(matches!(fim_fast.unwrap().provider, Provider::Mistral));
        
        // Invalid resolutions
        assert!(resolve_route(&routing, "chat", "nonexistent").is_none());
        assert!(resolve_route(&routing, "nonexistent", "fast").is_none());
        assert!(resolve_route(&routing, "fim", "smart").is_none());
        assert!(resolve_route(&routing, "", "fast").is_none());
        assert!(resolve_route(&routing, "chat", "").is_none());
    }
    
    #[test]
    fn test_route_target_creation() {
        let routes_raw = "test.route=xai:grok-beta,another.route=groq:llama-3.1-70b";
        let routing = build_routing(routes_raw);
        
        let xai_route = routing.get("test.route").unwrap();
        assert!(matches!(xai_route.provider, Provider::Xai));
        assert_eq!(xai_route.model, "grok-beta");
        
        let groq_route = routing.get("another.route").unwrap();
        assert!(matches!(groq_route.provider, Provider::Groq));
        assert_eq!(groq_route.model, "llama-3.1-70b");
    }
    
    #[test]
    fn test_build_routing_with_special_model_names() {
        let routes_raw = "chat.fast=openai:gpt-4-0125-preview,chat.smart=anthropic:claude-3-5-sonnet-20241022";
        let routing = build_routing(routes_raw);
        
        assert_eq!(routing.len(), 2);
        
        let fast_route = routing.get("chat.fast").unwrap();
        assert_eq!(fast_route.model, "gpt-4-0125-preview");
        
        let smart_route = routing.get("chat.smart").unwrap();
        assert_eq!(smart_route.model, "claude-3-5-sonnet-20241022");
    }
    
    #[test]
    fn test_build_routing_edge_cases() {
        // Test with trailing comma
        let routes_raw = "chat.fast=openai:gpt-4o-mini,";
        let routing = build_routing(routes_raw);
        assert_eq!(routing.len(), 1);
        
        // Test with multiple commas
        let routes_raw = "chat.fast=openai:gpt-4o-mini,,chat.smart=anthropic:claude-3-5-sonnet";
        let routing = build_routing(routes_raw);
        assert_eq!(routing.len(), 2);
        
        // Test with only whitespace
        let routing = build_routing("   ");
        assert_eq!(routing.len(), 0);
        
        // Test with only commas
        let routing = build_routing(",,,");
        assert_eq!(routing.len(), 0);
    }
}