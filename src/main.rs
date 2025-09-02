mod auth;
mod config;
mod convex_service;
mod file_processor;
mod routing;
mod search_service;
mod types;

use anyhow::Result;
use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    middleware,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::signal;
use tower::ServiceBuilder;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing::{info, warn};
use uuid::Uuid;

use auth::{AuthService, CreateUserRequest, LoginRequest};
use config::Config;
use convex_service::ConvexService;
use search_service::SearchService;
use types::{ApiResponse, InvokeRequest, AuthUser};

// Fallback in-memory rate limiter for guest users
const MAX_GUEST_MESSAGES_PER_DAY: u32 = 5;

#[derive(Debug, Clone)]
struct GuestUsage {
    count: u32,
    reset_at: u64,
}

type GuestUsageMap = Arc<Mutex<HashMap<String, GuestUsage>>>;

#[derive(Clone)]
struct AppState {
    config: Config,
    auth_service: AuthService,
    convex_service: ConvexService,
    search_service: SearchService,
    guest_usage: GuestUsageMap,
}

#[derive(Debug, Deserialize)]
struct CreateUserParams {
    email: String,
    password: String,
    subscription_tier: Option<String>,
}

#[derive(Debug, Deserialize)]
struct LoginParams {
    email: String,
    password: String,
}

#[derive(Debug, Deserialize)]
struct AnalyticsQuery {
    hours: Option<u32>,
}

fn start_of_next_day(timestamp: u64) -> u64 {
    // Convert to milliseconds and add 24 hours, then round down to start of day
    let next_day = timestamp + (24 * 60 * 60 * 1000);
    let next_day_start = (next_day / (24 * 60 * 60 * 1000)) * (24 * 60 * 60 * 1000);
    next_day_start
}

fn get_guest_key(fingerprint: Option<&str>, ip_address: Option<&str>, user_id: Option<&str>) -> String {
    if let Some(uid) = user_id {
        if uid.starts_with("anon-") {
            return format!("anon:{}", uid);
        }
    }
    format!("{}|{}", 
        fingerprint.unwrap_or("unknown"), 
        ip_address.unwrap_or("unknown")
    )
}

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

    let mut usage_map = guest_usage.lock().unwrap();
    
    if let Some(entry) = usage_map.get_mut(&key) {
        if now >= entry.reset_at {
            // Reset for new day
            let reset_at = start_of_next_day(now);
            let remaining = MAX_GUEST_MESSAGES_PER_DAY.saturating_sub(1);
            *entry = GuestUsage { count: 1, reset_at };
            return (true, remaining, reset_at, "fallback_ok".to_string());
        }
        
        if entry.count >= MAX_GUEST_MESSAGES_PER_DAY {
            return (false, 0, entry.reset_at, "fallback_limit".to_string());
        }
        
        entry.count += 1;
        let remaining = MAX_GUEST_MESSAGES_PER_DAY.saturating_sub(entry.count);
        (true, remaining, entry.reset_at, "fallback_ok".to_string())
    } else {
        // New entry
        let reset_at = start_of_next_day(now);
        let remaining = MAX_GUEST_MESSAGES_PER_DAY.saturating_sub(1);
        usage_map.insert(key, GuestUsage { count: 1, reset_at });
        (true, remaining, reset_at, "fallback_ok".to_string())
    }
}

// Health check endpoint
async fn health_check() -> Json<Value> {
    Json(json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

// Create user endpoint
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

// Login endpoint
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

// Anonymous session creation endpoint
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

// Analytics endpoint
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

// Main invoke endpoint (placeholder)
async fn invoke(
    State(state): State<AppState>,
    headers: HeaderMap,
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

fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/v1/auth/register", post(create_user))
        .route("/v1/auth/login", post(login))
        .route("/v1/auth/anonymous", post(create_anonymous_session))
        .route("/v1/analytics", get(get_analytics))
        .route("/v1/invoke", post(invoke))
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

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    // Load configuration from environment
    let config = Config::from_env();
    
    info!("Starting y-rust server...");
    info!("Bind address: {}", config.bind_address);
    
    // Initialize services
    let convex_service = ConvexService::new(config.clone());
    let auth_service = AuthService::new(config.clone(), convex_service.clone());
    let search_service = SearchService::new(config.clone());
    let guest_usage = Arc::new(Mutex::new(HashMap::new()));
    
    // Create application state
    let state = AppState {
        config: config.clone(),
        auth_service,
        convex_service,
        search_service,
        guest_usage,
    };
    
    // Create router
    let app = create_router(state);
    
    // Parse bind address
    let addr: SocketAddr = config.bind_address.parse()
        .expect("Invalid bind address format");
    
    info!("Server listening on {}", addr);
    
    // Start the server
    let listener = tokio::net::TcpListener::bind(addr).await?;
    
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            info!("Received Ctrl+C, shutting down...");
        },
        _ = terminate => {
            info!("Received SIGTERM, shutting down...");
        },
    }
}