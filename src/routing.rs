use std::collections::HashMap;

use crate::types::{Provider, RouteTarget};

pub type RoutingMap = HashMap<String, RouteTarget>; // key = `${op}.${tier}`

pub fn build_routing(routes_raw: &str) -> RoutingMap {
    let mut map = HashMap::new();
    
    for pair in routes_raw.split(',') {
        let trimmed = pair.trim();
        if trimmed.is_empty() || !trimmed.contains('=') {
            continue;
        }
        
        let parts: Vec<&str> = trimmed.split('=').collect();
        if parts.len() != 2 {
            continue;
        }
        
        let lhs_parts: Vec<&str> = parts[0].split('.').collect();
        if lhs_parts.len() != 2 {
            continue;
        }
        
        let op = lhs_parts[0];
        let tier = lhs_parts[1];
        
        let (provider, model) = if parts[1].contains(':') {
            let rhs_parts: Vec<&str> = parts[1].split(':').collect();
            if rhs_parts.len() == 2 {
                (normalize_provider(rhs_parts[0]), rhs_parts[1].to_string())
            } else {
                (Provider::OpenAI, parts[1].to_string())
            }
        } else {
            (Provider::OpenAI, parts[1].to_string())
        };
        
        let key = format!("{}.{}", op, tier);
        map.insert(key, RouteTarget { provider, model });
    }
    
    map
}

pub fn resolve_route<'a>(map: &'a RoutingMap, op: &str, tier: &str) -> Option<&'a RouteTarget> {
    let key = format!("{}.{}", op, tier);
    map.get(&key)
}

fn normalize_provider(provider_str: &str) -> Provider {
    match provider_str.to_lowercase().as_str() {
        "cf" => Provider::Cloudflare,
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
}