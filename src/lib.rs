pub mod auth;
pub mod config;
pub mod convex_service;
pub mod file_processor;
pub mod routing;
pub mod search_service;
pub mod types;

use reqwest::Client;

use crate::{
    auth::{AuthService, CreateUserRequest, LoginRequest},
    config::Config,
    convex_service::ConvexService,
    file_processor::{process_file_attachments, supports_multimodal},
    routing::{build_routing, resolve_route},
    search_service::SearchService,
    types::{Attachment, InvokeRequest, MessageRole, Operation, Provider, ChatMessage},
};

#[tokio::test]
async fn test_basic_functionality() {
    // Test config loading
    let config = Config::from_env();
    assert!(!config.bind_address.is_empty());

    // Test routing
    let routing = build_routing("chat.fast=openai:gpt-4o-mini,chat.smart=anthropic:claude-3-5-sonnet");
    assert_eq!(routing.len(), 2);
    
    let route = resolve_route(&routing, "chat", "fast");
    assert!(route.is_some());
    
    // Test multimodal support
    assert!(supports_multimodal("openai", "gpt-4o"));
    assert!(!supports_multimodal("openai", "gpt-3.5-turbo"));

    // Test services initialization
    let convex_service = ConvexService::new(config.clone());
    let auth_service = AuthService::new(config.clone(), convex_service.clone());
    let search_service = SearchService::new(config.clone());

    // Basic search test
    assert!(!search_service.needs_internet_search("hello world"));
    assert!(search_service.needs_internet_search("what is the weather today"));

    println!("All basic tests passed!");
}

#[test]
fn test_types_serialization() {
    use serde_json;

    // Test ChatMessage serialization
    let message = ChatMessage {
        role: MessageRole::User,
        content: "Hello world".to_string(),
    };
    
    let json = serde_json::to_string(&message).unwrap();
    assert!(json.contains("user"));
    assert!(json.contains("Hello world"));

    // Test Provider serialization
    let provider = Provider::OpenAI;
    let json = serde_json::to_string(&provider).unwrap();
    assert!(json.contains("openai"));

    println!("Serialization tests passed!");
}