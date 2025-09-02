//! Rust-AI Main Application Entry Point
//! 
//! This module contains the HTTP server implementation and main application logic.
//! It provides:
//! - RESTful API endpoints for authentication, AI invocation, and analytics
//! - In-memory rate limiting for guest users
//! - Graceful shutdown handling
//! - CORS and tracing middleware
//! 
//! The server is built using Axum framework for high-performance async HTTP handling.

// Module declarations - each module handles a specific domain of functionality
mod auth;              // Authentication and user management
mod config;            // Configuration loading from environment variables
mod convex_service;    // Database abstraction layer for Convex backend
mod file_processor;    // File upload and processing utilities
mod routing;           // Provider routing and AI request handling
mod search_service;    // Web search integration for enhanced AI responses
mod types;             // Type definitions and serialization structs

// Standard library and external crate imports
use anyhow::Result;
use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    response::Json,
    routing::{get, post},
    Router,
};
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::signal;
use tower::ServiceBuilder;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing::info;
use uuid::Uuid;

// Internal module imports
use auth::{AuthService, CreateUserRequest, LoginRequest};
use config::Config;
use convex_service::ConvexService;
use search_service::SearchService;
use types::{ApiResponse, InvokeRequest, AuthUser};

// Rate limiting configuration for guest users
// This prevents abuse while allowing trial usage without registration
#[allow(dead_code)]
const MAX_GUEST_MESSAGES_PER_DAY: u32 = 5;

/// Guest usage tracking structure for rate limiting
/// 
/// Tracks usage count and reset timestamp for guest users to enforce
/// daily limits without requiring database persistence.
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct GuestUsage {
    /// Number of requests made by this guest today
    count: u32,
    /// Unix timestamp (milliseconds) when the counter resets
    reset_at: u64,
}

/// Thread-safe guest usage map using Arc<Mutex<>> for concurrent access
/// 
/// Key format: "fingerprint|ip_address" or "anon:anonymous_user_id"
/// This allows tracking by browser fingerprint + IP for better accuracy
type GuestUsageMap = Arc<Mutex<HashMap<String, GuestUsage>>>;

/// Application state container shared across all request handlers
/// 
/// Contains all services and configuration needed to process requests.
/// Cloned cheaply due to Arc wrappers in underlying services.
#[derive(Clone)]
#[allow(dead_code)]
struct AppState {
    /// Application configuration loaded from environment
    config: Config,
    /// Authentication service for user management and JWT handling
    auth_service: AuthService,
    /// Database service for persistent data operations
    convex_service: ConvexService,
    /// Search service for web search integration
    search_service: SearchService,
    /// In-memory rate limiting for guest users
    guest_usage: GuestUsageMap,
}

/// Request payload for user registration endpoint
/// 
/// Validated at the HTTP layer before being passed to auth service
#[derive(Debug, Deserialize)]
struct CreateUserParams {
    /// User's email address (used as unique identifier)
    email: String,
    /// Plain text password (will be hashed with bcrypt)
    password: String,
    /// Optional subscription tier (defaults to "free" if not provided)
    subscription_tier: Option<String>,
}

/// Request payload for user login endpoint
/// 
/// Simple email/password authentication
#[derive(Debug, Deserialize)]
struct LoginParams {
    /// User's registered email address
    email: String,
    /// User's password (compared against stored bcrypt hash)
    password: String,
}

/// Query parameters for analytics endpoint
/// 
/// Allows filtering analytics data by time range
#[derive(Debug, Deserialize)]
struct AnalyticsQuery {
    /// Number of hours back to fetch analytics data (optional)
    /// Defaults to all available data if not specified
    hours: Option<u32>,
}

/// Calculate the start of the next day in milliseconds since Unix epoch
/// 
/// Used for rate limiting reset times. Ensures all users get their
/// quota reset at the same time (start of day in UTC).
/// 
/// # Arguments
/// * `timestamp` - Current timestamp in milliseconds
/// 
/// # Returns
/// Timestamp in milliseconds representing the start of the next day
#[allow(dead_code)]
fn start_of_next_day(timestamp: u64) -> u64 {
    // Add 24 hours to current time, then round down to start of day
    let next_day = timestamp + (24 * 60 * 60 * 1000);
    let next_day_start = (next_day / (24 * 60 * 60 * 1000)) * (24 * 60 * 60 * 1000);
    next_day_start
}

/// Generate a unique key for guest user tracking
/// 
/// Creates a consistent key for rate limiting that can identify
/// unique guests across requests using available identifiers.
/// 
/// Priority: user_id (for anonymous users) > fingerprint + IP > fallback
/// 
/// # Arguments
/// * `fingerprint` - Browser fingerprint (optional)
/// * `ip_address` - Client IP address (optional) 
/// * `user_id` - Anonymous user ID if available (optional)
/// 
/// # Returns
/// String key for tracking this guest in the usage map
#[allow(dead_code)]
fn get_guest_key(fingerprint: Option<&str>, ip_address: Option<&str>, user_id: Option<&str>) -> String {
    // If we have an anonymous user ID, use that for consistency
    if let Some(uid) = user_id {
        if uid.starts_with("anon-") {
            return format!("anon:{}", uid);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum_test::TestServer;
    use serde_json::{json, Value};
    
    fn create_test_app_state() -> AppState {
        let config = Config::from_env();
        let convex_service = ConvexService::new(config.clone());
        let auth_service = AuthService::new(config.clone(), convex_service.clone());
        let search_service = SearchService::new(config.clone());
        
        AppState {
            config,
            auth_service,
            convex_service,
            search_service,
            guest_usage: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    #[tokio::test]
    async fn test_health_check() {
        let state = create_test_app_state();
        let app = create_router(state);
        let server = TestServer::new(app).unwrap();
        
        let response = server.get("/health").await;
        
        response.assert_status_ok();
        response.assert_json(&json!({
            "status": "ok",
            "service": "rust-ai-api",
            "version": env!("CARGO_PKG_VERSION")
        }));
    }
    
    #[tokio::test]
    async fn test_invoke_endpoint_structure() {
        let state = create_test_app_state();
        let app = create_router(state);
        let server = TestServer::new(app).unwrap();
        
        let request_body = json!({
            "operation": "chat",
            "tier": "fast",
            "messages": [
                {
                    "role": "user",
                    "content": "Hello, AI!"
                }
            ]
        });
        
        let response = server.post("/v1/invoke").json(&request_body).await;
        
        response.assert_status_ok();
        
        let body: Value = response.json();
        assert_eq!(body["status"], "success");
        assert!(body["data"]["request_id"].is_string());
        assert_eq!(body["data"]["status"], "processed");
    }
    
    #[tokio::test]
    async fn test_invoke_endpoint_invalid_request() {
        let state = create_test_app_state();
        let app = create_router(state);
        let server = TestServer::new(app).unwrap();
        
        let invalid_request = json!({
            "invalid_field": "value"
        });
        
        let response = server.post("/v1/invoke").json(&invalid_request).await;
        
        // Should return 400 Bad Request or similar for malformed JSON
        assert!(response.status_code().is_client_error());
    }
    
    #[tokio::test]
    async fn test_analytics_endpoint() {
        let state = create_test_app_state();
        let app = create_router(state);
        let server = TestServer::new(app).unwrap();
        
        let response = server.get("/v1/analytics").await;
        
        response.assert_status_ok();
        
        let body: Value = response.json();
        assert_eq!(body["status"], "success");
        assert!(body["data"].is_object());
        // Should contain analytics data structure
        assert!(body["data"]["total_requests"].is_number());
        assert!(body["data"]["active_users"].is_number());
    }
    
    #[tokio::test]
    async fn test_anonymous_session_creation() {
        let state = create_test_app_state();
        let app = create_router(state);
        let server = TestServer::new(app).unwrap();
        
        let response = server.post("/v1/auth/anonymous").await;
        
        response.assert_status_ok();
        
        let body: Value = response.json();
        assert_eq!(body["status"], "success");
        assert!(body["data"]["token"].is_string());
        assert!(body["data"]["user"]["is_anonymous"].as_bool().unwrap());
        assert!(body["data"]["user"]["id"].as_str().unwrap().starts_with("anon-"));
    }
    
    #[tokio::test]
    async fn test_user_registration_endpoint() {
        let state = create_test_app_state();
        let app = create_router(state);
        let server = TestServer::new(app).unwrap();
        
        let registration_request = json!({
            "email": "test@example.com",
            "password": "securepassword123"
        });
        
        let response = server.post("/v1/auth/register").json(&registration_request).await;
        
        // Note: This might fail in the actual implementation due to validation
        // or database constraints, but we're testing the endpoint structure
        response.assert_status_ok();
        
        let body: Value = response.json();
        // Should return success or appropriate validation error
        assert!(body["status"].is_string());
    }
    
    #[tokio::test]
    async fn test_login_endpoint() {
        let state = create_test_app_state();
        let app = create_router(state);
        let server = TestServer::new(app).unwrap();
        
        let login_request = json!({
            "email": "test@example.com",
            "password": "password123"
        });
        
        let response = server.post("/v1/auth/login").json(&login_request).await;
        
        response.assert_status_ok();
        
        let body: Value = response.json();
        assert_eq!(body["status"], "success");
        // Login should fail for non-existent user but return proper structure
        assert!(body["data"].is_object());
    }
    
    #[tokio::test]
    async fn test_cors_headers() {
        let state = create_test_app_state();
        let app = create_router(state);
        let server = TestServer::new(app).unwrap();
        
        let response = server.get("/health").await;
        
        // Check that CORS headers are present
        assert!(response.headers().contains_key("access-control-allow-origin"));
    }
    
    #[tokio::test]
    async fn test_nonexistent_route() {
        let state = create_test_app_state();
        let app = create_router(state);
        let server = TestServer::new(app).unwrap();
        
        let response = server.get("/nonexistent").await;
        
        response.assert_status_not_found();
    }
    
    #[tokio::test]
    async fn test_invoke_with_attachments() {
        let state = create_test_app_state();
        let app = create_router(state);
        let server = TestServer::new(app).unwrap();
        
        let request_body = json!({
            "operation": "chat",
            "tier": "smart",
            "messages": [
                {
                    "role": "user",
                    "content": "Analyze this file"
                }
            ],
            "attachments": [
                {
                    "filename": "test.txt",
                    "content_type": "text/plain",
                    "data": "SGVsbG8gV29ybGQ=" // Base64: "Hello World"
                }
            ],
            "temperature": 0.7,
            "max_tokens": 1000
        });
        
        let response = server.post("/v1/invoke").json(&request_body).await;
        
        response.assert_status_ok();
        
        let body: Value = response.json();
        assert_eq!(body["status"], "success");
        assert!(body["data"]["request_id"].is_string());
    }
    
    #[test]
    fn test_guest_usage_functions() {
        // Test get_guest_key function
        let key1 = get_guest_key(Some("fingerprint123"), Some("192.168.1.1"), None);
        assert_eq!(key1, "fingerprint123|192.168.1.1");
        
        let key2 = get_guest_key(None, Some("192.168.1.1"), Some("anon-123"));
        assert_eq!(key2, "anon:anon-123");
        
        let key3 = get_guest_key(None, Some("192.168.1.1"), None);
        assert_eq!(key3, "ip:192.168.1.1");
        
        // Test start_of_next_day function
        let timestamp = 1640995200000; // Jan 1, 2022 00:00:00 UTC
        let next_day = start_of_next_day(timestamp);
        let expected_next_day = 1641081600000; // Jan 2, 2022 00:00:00 UTC
        assert_eq!(next_day, expected_next_day);
    }
    
    #[test]
    fn test_check_guest_daily_limit() {
        let guest_usage = Arc::new(Mutex::new(HashMap::new()));
        
        // First request should be allowed
        let (allowed, remaining, reset_at, _message) = check_guest_daily_limit(
            &guest_usage,
            Some("fingerprint123"),
            Some("192.168.1.1"),
            None
        );
        
        assert!(allowed);
        assert_eq!(remaining, 4); // 5 - 1 = 4 remaining
        assert!(reset_at > 0);
    }
}
    // Otherwise combine fingerprint and IP for best guest tracking
    format!("{}|{}", 
        fingerprint.unwrap_or("unknown"), 
        ip_address.unwrap_or("unknown")
    )
}

/// Check and enforce daily rate limits for guest users
/// 
/// This is the primary rate limiting mechanism for unauthenticated users.
/// It prevents abuse while allowing genuine trial usage.
/// 
/// The function is thread-safe and handles concurrent access through mutex locking.
/// It automatically resets counters at the start of each new day.
/// 
/// # Arguments
/// * `guest_usage` - Shared map of guest usage tracking
/// * `fingerprint` - Browser fingerprint for identification
/// * `ip_address` - Client IP address for identification  
/// * `user_id` - Anonymous user ID if available
/// 
/// # Returns
/// Tuple containing:
/// * `bool` - Whether request is allowed (under rate limit)
/// * `u32` - Remaining requests for today
/// * `u64` - Timestamp when limit resets (milliseconds since epoch)
/// * `String` - Status message for logging/debugging
#[allow(dead_code)]
fn check_guest_daily_limit(
    guest_usage: &GuestUsageMap,
    fingerprint: Option<&str>,
    ip_address: Option<&str>,
    user_id: Option<&str>,
) -> (bool, u32, u64, String) {
    let key = get_guest_key(fingerprint, ip_address, user_id);
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;

    // Lock the usage map for thread-safe access
    let mut usage_map = guest_usage.lock().unwrap();
    
    if let Some(entry) = usage_map.get_mut(&key) {
        // Check if we need to reset for a new day
        if now >= entry.reset_at {
            let reset_at = start_of_next_day(now);
            let remaining = MAX_GUEST_MESSAGES_PER_DAY.saturating_sub(1);
            *entry = GuestUsage { count: 1, reset_at };
            return (true, remaining, reset_at, "fallback_ok".to_string());
        }
        
        // Check if user has exceeded daily limit
        if entry.count >= MAX_GUEST_MESSAGES_PER_DAY {
            return (false, 0, entry.reset_at, "fallback_limit".to_string());
        }
        
        // Increment usage counter
        entry.count += 1;
        let remaining = MAX_GUEST_MESSAGES_PER_DAY.saturating_sub(entry.count);
        (true, remaining, entry.reset_at, "fallback_ok".to_string())
    } else {
        // First request from this guest - create new tracking entry
        let reset_at = start_of_next_day(now);
        let remaining = MAX_GUEST_MESSAGES_PER_DAY.saturating_sub(1);
        usage_map.insert(key, GuestUsage { count: 1, reset_at });
        (true, remaining, reset_at, "fallback_ok".to_string())
    }
}

/// Health check endpoint for monitoring and load balancer probes
/// 
/// Returns server status and current timestamp. Used by:
/// - Load balancers for health checks
/// - Monitoring systems for uptime tracking
/// - Developers for quick service verification
/// 
/// Always returns 200 OK with JSON response.
async fn health_check() -> Json<Value> {
    Json(json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

/// User registration endpoint
/// 
/// Creates a new user account with email/password authentication.
/// Passwords are automatically hashed with bcrypt before storage.
/// 
/// # Request Body
/// ```json
/// {
///   "email": "user@example.com",
///   "password": "secure_password",
///   "subscription_tier": "free" // optional, defaults to "free"
/// }
/// ```
/// 
/// # Response
/// Returns created user data with API key for immediate use.
/// 
/// # Errors
/// - 400 BAD_REQUEST: Email already exists or validation failed
/// - 500 INTERNAL_SERVER_ERROR: Database or service error
async fn create_user(
    State(state): State<AppState>,
    Json(params): Json<CreateUserParams>,
) -> Result<Json<ApiResponse<AuthUser>>, StatusCode> {
    let request = CreateUserRequest {
        email: params.email,
        password: params.password,
        subscription_tier: params.subscription_tier,
    };

    match state.auth_service.create_user(request).await {
        Ok(result) => {
            if result.success {
                if let Some(user) = result.user {
                    Ok(Json(ApiResponse::success(user)))
                } else {
                    Err(StatusCode::INTERNAL_SERVER_ERROR)
                }
            } else {
                Err(StatusCode::BAD_REQUEST)
            }
        }
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// User login endpoint
/// 
/// Authenticates user with email/password and returns JWT token.
/// Token can be used for subsequent authenticated requests.
/// 
/// # Request Body
/// ```json
/// {
///   "email": "user@example.com", 
///   "password": "user_password"
/// }
/// ```
/// 
/// # Response
/// Returns JWT token and user information on successful login.
/// 
/// # Errors
/// - 401 UNAUTHORIZED: Invalid credentials
/// - 500 INTERNAL_SERVER_ERROR: Database or service error
async fn login(
    State(state): State<AppState>,
    Json(params): Json<LoginParams>,
) -> Result<Json<ApiResponse<Value>>, StatusCode> {
    let request = LoginRequest {
        email: params.email,
        password: params.password,
    };

    match state.auth_service.login(request).await {
        Ok(result) => {
            if result.success {
                let response_data = json!({
                    "token": result.token,
                    "user": result.user
                });
                Ok(Json(ApiResponse::success(response_data)))
            } else {
                Err(StatusCode::UNAUTHORIZED)
            }
        }
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Anonymous session creation endpoint
/// 
/// Creates a temporary guest user with limited capabilities.
/// Useful for trials and demos without requiring registration.
/// 
/// Guest users have:
/// - Limited daily request quota (5 requests/day)
/// - Temporary session (no persistent data)
/// - Basic AI access without advanced features
/// 
/// # Response  
/// Returns JWT token for the guest session.
/// 
/// # Errors
/// - 500 INTERNAL_SERVER_ERROR: Service error
async fn create_anonymous_session(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Value>>, StatusCode> {
    match state.auth_service.create_guest_user().await {
        Ok(result) => {
            if result.success {
                let response_data = json!({
                    "token": result.token,
                    "user": result.user
                });
                Ok(Json(ApiResponse::success(response_data)))
            } else {
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Analytics data retrieval endpoint
/// 
/// Provides usage statistics and system metrics.
/// Useful for monitoring, billing, and system optimization.
/// 
/// # Query Parameters
/// - `hours`: Optional number of hours back to fetch data
/// 
/// # Example
/// ```
/// GET /v1/analytics?hours=24
/// ```
/// 
/// # Response
/// Returns aggregated analytics data including:
/// - Request counts by provider
/// - Response times
/// - Error rates
/// - Usage by time period
/// 
/// # Errors
/// - 500 INTERNAL_SERVER_ERROR: Database error
async fn get_analytics(
    State(state): State<AppState>,
    Query(query): Query<AnalyticsQuery>,
) -> Result<Json<ApiResponse<Value>>, StatusCode> {
    let hours = query.hours;
    
    match state.convex_service.get_analytics(None, hours).await {
        Ok(analytics_data) => Ok(Json(ApiResponse::success(analytics_data))),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Main AI invocation endpoint (PLACEHOLDER IMPLEMENTATION)
/// 
/// This is the core endpoint for AI requests. Currently a placeholder
/// that demonstrates the expected structure and response format.
/// 
/// TODO: Full implementation needed with:
/// - JWT token validation and user authentication
/// - Rate limiting enforcement (guest vs. registered users)
/// - Provider routing based on request and availability
/// - Request/response logging and analytics
/// - Error handling and fallback logic
/// - Context injection from file uploads and search
/// 
/// # Request Body
/// ```json
/// {
///   "op": "chat", 
///   "input": {
///     "messages": [{"role": "user", "content": "Hello"}],
///     "provider": "openai", // optional
///     "model": "gpt-3.5-turbo" // optional
///   }
/// }
/// ```
/// 
/// # Headers
/// - Authorization: Bearer <JWT_TOKEN> (required)
/// 
/// # Response
/// Returns AI provider response with metadata.
/// 
/// # Errors
/// - 401 UNAUTHORIZED: Missing/invalid token or rate limit exceeded
/// - 400 BAD_REQUEST: Invalid request format
/// - 503 SERVICE_UNAVAILABLE: All providers unavailable
/// - 500 INTERNAL_SERVER_ERROR: Service error
async fn invoke(
    State(_state): State<AppState>,
    _headers: HeaderMap,
    Json(request): Json<InvokeRequest>,
) -> Result<Json<ApiResponse<Value>>, StatusCode> {
    let request_id = Uuid::new_v4().to_string();
    
    // TODO: Implement full invoke logic with provider routing, authentication, etc.
    // This is a placeholder that demonstrates the structure
    
    tracing::info!("Processing invoke request: {:?}", request);
    
    // For now, return a simple response
    let response_data = json!({
        "request_id": request_id,
        "status": "processed",
        "message": "This is a placeholder response - full implementation needed"
    });
    
    Ok(Json(ApiResponse::success(response_data)))
}

/// Create and configure the Axum router with all routes and middleware
/// 
/// Sets up the complete HTTP service with:
/// - All API endpoints with proper HTTP methods
/// - Middleware stack (tracing, CORS)
/// - Shared application state
/// 
/// The middleware stack is applied in reverse order:
/// 1. CORS (outermost - handles preflight requests)
/// 2. Tracing (logs all requests and responses)
/// 3. Route handlers (innermost - actual business logic)
/// 
/// # Arguments
/// * `state` - Application state shared across all handlers
/// 
/// # Returns
/// Configured Axum Router ready for serving
fn create_router(state: AppState) -> Router {
    Router::new()
        // Health and monitoring endpoints
        .route("/health", get(health_check))
        
        // Authentication endpoints
        .route("/v1/auth/register", post(create_user))
        .route("/v1/auth/login", post(login))
        .route("/v1/auth/anonymous", post(create_anonymous_session))
        
        // Analytics and monitoring
        .route("/v1/analytics", get(get_analytics))
        
        // Core AI functionality 
        .route("/v1/invoke", post(invoke))
        
        // Middleware stack (applied in reverse order)
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(
                    CorsLayer::new()
                        .allow_origin(Any) // TODO: Configure proper CORS based on config
                        .allow_methods(Any)
                        .allow_headers(Any)
                )
        )
        .with_state(state)
}

/// Application entry point
/// 
/// Initializes all services, configures the HTTP server, and starts
/// listening for requests with graceful shutdown support.
/// 
/// Startup sequence:
/// 1. Initialize structured logging with tracing
/// 2. Load configuration from environment variables
/// 3. Create all service instances with dependency injection
/// 4. Build the HTTP router with middleware stack
/// 5. Start the server with graceful shutdown handling
/// 
/// The server will continue running until receiving:
/// - SIGTERM (graceful shutdown signal)
/// - SIGINT/Ctrl+C (user interruption)
/// 
/// # Returns
/// Result indicating successful startup or initialization error
#[tokio::main]
async fn main() -> Result<()> {
    // Initialize structured logging for observability
    // Uses environment variable RUST_LOG for level control
    tracing_subscriber::fmt::init();
    
    // Load all configuration from environment variables
    // Validates required settings and provides sensible defaults
    let config = Config::from_env();
    
    info!("Starting Rust-AI server...");
    info!("Bind address: {}", config.bind_address);
    
    // Initialize all services with dependency injection
    // Order matters: ConvexService first (used by others)
    let convex_service = ConvexService::new(config.clone());
    let auth_service = AuthService::new(config.clone(), convex_service.clone());
    let search_service = SearchService::new(config.clone());
    
    // Initialize in-memory rate limiting for guest users
    let guest_usage = Arc::new(Mutex::new(HashMap::new()));
    
    // Create shared application state for all request handlers
    let state = AppState {
        config: config.clone(),
        auth_service,
        convex_service,
        search_service,
        guest_usage,
    };
    
    // Build the complete HTTP router with middleware
    let app = create_router(state);
    
    // Parse the bind address from configuration
    let addr: SocketAddr = config.bind_address.parse()
        .expect("Invalid bind address format");
    
    info!("Server listening on {}", addr);
    
    // Start the HTTP server with graceful shutdown support
    let listener = tokio::net::TcpListener::bind(addr).await?;
    
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    
    Ok(())
}

/// Graceful shutdown signal handler
/// 
/// Listens for system signals that indicate the server should shut down:
/// - SIGTERM: Sent by process managers (Docker, systemd, etc.)  
/// - SIGINT: Sent by Ctrl+C from terminal
/// 
/// When a signal is received, the server will:
/// 1. Stop accepting new connections
/// 2. Wait for existing requests to complete  
/// 3. Clean up resources and exit
/// 
/// This ensures data integrity and proper cleanup on shutdown.
async fn shutdown_signal() {
    // Handle Ctrl+C signal (SIGINT)
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    // Handle SIGTERM signal (Unix systems only)
    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    // On non-Unix systems, only handle Ctrl+C
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    // Wait for either signal to be received
    tokio::select! {
        _ = ctrl_c => {
            info!("Received Ctrl+C, shutting down...");
        },
        _ = terminate => {
            info!("Received SIGTERM, shutting down...");
        },
    }
}