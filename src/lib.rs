//! Rust-AI Library Module
//! 
//! This is the main library interface for the Rust-AI application.
//! It re-exports all public modules and provides integration tests
//! to ensure all components work together correctly.
//! 
//! The library provides:
//! - Configuration management from environment variables
//! - Authentication services with JWT and bcrypt
//! - Multiple AI provider integration
//! - Web search capabilities for enhanced responses
//! - File processing for multimodal AI interactions
//! - Database services for persistence and analytics
//! 
//! # Usage
//! 
//! This library is primarily used by the main application binary,
//! but can also be used as a standalone library for AI integration.

// Public module exports for external usage
pub mod auth;              // Authentication and user management
pub mod config;            // Configuration from environment variables  
pub mod convex_service;    // Database abstraction layer
pub mod file_processor;    // File upload and processing utilities
pub mod routing;           // AI provider routing logic
pub mod search_service;    // Web search integration
pub mod types;             // Shared type definitions

use reqwest::Client;

// Internal imports for testing
use crate::{
    auth::{AuthService, CreateUserRequest, LoginRequest},
    config::Config,
    convex_service::ConvexService,
    file_processor::{process_file_attachments, supports_multimodal},
    routing::{build_routing, resolve_route},
    search_service::SearchService,
    types::{Attachment, InvokeRequest, MessageRole, Operation, Provider, ChatMessage},
};

/// Integration test for basic system functionality
/// 
/// Tests the core components working together:
/// - Configuration loading from environment
/// - Routing system setup and resolution
/// - Service initialization and basic operations
/// - Multimodal support detection
/// 
/// This test ensures all major components can be initialized
/// and perform basic operations without errors.
#[tokio::test]
async fn test_basic_functionality() {
    // Test configuration loading from environment variables
    let config = Config::from_env();
    assert!(!config.bind_address.is_empty());

    // Test routing system with provider/model mapping
    let routing = build_routing("chat.fast=openai:gpt-4o-mini,chat.smart=anthropic:claude-3-5-sonnet");
    assert_eq!(routing.len(), 2);
    
    // Test route resolution for specific operation and tier
    let route = resolve_route(&routing, "chat", "fast");
    assert!(route.is_some());
    
    // Test multimodal support detection for different models
    assert!(supports_multimodal("openai", "gpt-4o"));
    assert!(!supports_multimodal("openai", "gpt-3.5-turbo"));

    // Test service initialization with dependency injection
    let convex_service = ConvexService::new(config.clone());
    let auth_service = AuthService::new(config.clone(), convex_service.clone());
    let search_service = SearchService::new(config.clone());

    // Test search service basic functionality
    assert!(!search_service.needs_internet_search("hello world"));
    assert!(search_service.needs_internet_search("what is the weather today"));

    println!("All basic tests passed!");
}

/// Test serialization and deserialization of core types
/// 
/// Ensures that all API types can be properly converted to/from JSON
/// for HTTP communication. Tests the serde implementations and
/// custom serialization attributes.
#[test]
fn test_types_serialization() {
    use serde_json;

    // Test ChatMessage serialization with role-based content
    let message = ChatMessage {
        role: MessageRole::User,
        content: "Hello world".to_string(),
    };
    
    let json = serde_json::to_string(&message).unwrap();
    assert!(json.contains("user"));
    assert!(json.contains("Hello world"));

    // Test Provider enum serialization with custom rename attributes
    let provider = Provider::OpenAI;
    let json = serde_json::to_string(&provider).unwrap();
    assert!(json.contains("openai"));

    println!("Serialization tests passed!");
}