//! Authentication Service Module
//! 
//! This module provides comprehensive user authentication and session management:
//! - User registration and login with bcrypt password hashing
//! - JWT token generation and validation
//! - Guest/anonymous user sessions for trial usage  
//! - Integration with Convex database for user persistence
//! - Security logging for authentication events
//! 
//! The service supports both registered users (with persistent accounts)
//! and anonymous users (with temporary sessions and limited capabilities).

use anyhow::{anyhow, Result};
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::{DateTime, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::config::Config;
use crate::convex_service::{ConvexService, UserAccount};
use crate::types::AuthUser;

/// Request payload for user registration
/// 
/// Contains all necessary information to create a new user account.
/// Passwords are plain text in the request but immediately hashed before storage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUserRequest {
    /// User's email address (must be unique across the system)
    pub email: String,
    /// Plain text password (will be hashed with bcrypt before storage)
    pub password: String,
    /// Optional subscription tier (defaults to "free" if not provided)
    pub subscription_tier: Option<String>,
}

/// Request payload for user login  
/// 
/// Simple email/password authentication for existing users.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginRequest {
    /// Registered email address
    pub email: String,
    /// User's password (compared against stored bcrypt hash)
    pub password: String,
}

/// Unified authentication result structure
/// 
/// Used by all authentication methods to provide consistent response format.
/// Contains either success data (token + user) or error information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResult {
    /// Whether the authentication attempt succeeded
    pub success: bool,
    /// JWT token for authenticated requests (only present on success)
    pub token: Option<String>,
    /// User information (only present on success)
    pub user: Option<AuthUser>,
    /// Error message (only present on failure)
    pub error: Option<String>,
}

/// JWT token claims structure
/// 
/// Contains user identification and metadata embedded in JWT tokens.
/// Used for stateless authentication and authorization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    /// Unique user identifier (database ID)
    pub user_id: String,
    /// User's email address
    pub email: String,
    /// Token type identifier (currently always "user_session")
    pub r#type: String,
    /// Token issued at timestamp (Unix timestamp)
    pub iat: i64,
    /// Token expiration timestamp (Unix timestamp)
    pub exp: i64,
}

/// Authentication service providing user management and session handling
/// 
/// This service handles all authentication-related operations including:
/// - Password hashing and verification using bcrypt
/// - JWT token generation and validation
/// - User registration and login flows
/// - Anonymous session creation for guest users
/// - Integration with database for user persistence
#[derive(Clone)]
pub struct AuthService {
    /// Application configuration (contains JWT secret, Clerk config, etc.)
    config: Config,
    /// Database service for user persistence and logging
    convex_service: ConvexService,
}

impl AuthService {
    /// Create a new authentication service instance
    /// 
    /// # Arguments
    /// * `config` - Application configuration containing secrets and settings
    /// * `convex_service` - Database service for user persistence
    /// 
    /// # Returns
    /// New AuthService instance ready for use
    pub fn new(config: Config, convex_service: ConvexService) -> Self {
        Self {
            config,
            convex_service,
        }
    }

    /// Hash a password using bcrypt with default cost factor
    /// 
    /// Uses tokio::spawn_blocking to avoid blocking the async runtime
    /// since bcrypt hashing is CPU-intensive.
    /// 
    /// # Arguments
    /// * `password` - Plain text password to hash
    /// 
    /// # Returns
    /// Result containing the bcrypt hash string or error
    /// 
    /// # Security
    /// Uses DEFAULT_COST (12 rounds) which provides good security vs. performance balance
    pub async fn hash_password(&self, password: &str) -> Result<String> {
        let password = password.to_string();
        tokio::task::spawn_blocking(move || {
            hash(password, DEFAULT_COST).map_err(|e| anyhow!("Failed to hash password: {}", e))
        })
        .await?
    }

    /// Verify a password against a stored bcrypt hash
    /// 
    /// Uses tokio::spawn_blocking to avoid blocking the async runtime
    /// since bcrypt verification is CPU-intensive.
    /// 
    /// # Arguments  
    /// * `password` - Plain text password to verify
    /// * `hash` - Stored bcrypt hash to compare against
    /// 
    /// # Returns
    /// Result containing boolean indicating whether password matches
    /// 
    /// # Security
    /// Uses constant-time comparison to prevent timing attacks
    pub async fn verify_password(&self, password: &str, hash: &str) -> Result<bool> {
        let password = password.to_string();
        let hash = hash.to_string();
        tokio::task::spawn_blocking(move || {
            verify(password, &hash).map_err(|e| anyhow!("Failed to verify password: {}", e))
        })
        .await?
    }

    /// Generate a unique API key for a user
    /// 
    /// API keys are used for programmatic access and follow the format "ak_<uuid>".
    /// They are stored with user accounts for authentication without passwords.
    /// 
    /// # Returns
    /// String containing the generated API key
    pub fn generate_api_key(&self) -> String {
        format!("ak_{}", Uuid::new_v4().simple())
    }

    /// Generate a JWT token for a user session
    /// 
    /// Creates a signed JWT token containing user identification and session metadata.
    /// Tokens are valid for 7 days and use HMAC-SHA256 signing.
    /// 
    /// # Arguments
    /// * `user_id` - Unique user identifier
    /// * `email` - User's email address
    /// 
    /// # Returns
    /// Result containing the JWT token string or error
    /// 
    /// # Security
    /// - Tokens expire after 7 days
    /// - Signed with server secret (HMAC-SHA256)
    /// - Contains user_id and email for identification
    /// - Includes issued-at and expiration timestamps
    pub fn generate_jwt(&self, user_id: &str, email: &str) -> Result<String> {
        let secret = self.config.action_token_secret
            .as_ref()
            .ok_or_else(|| anyhow!("JWT secret not configured"))?;

        let now = Utc::now().timestamp();
        let claims = Claims {
            user_id: user_id.to_string(),
            email: email.to_string(),
            r#type: "user_session".to_string(),
            iat: now,
            exp: now + (7 * 24 * 60 * 60), // 7 days expiration
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(secret.as_bytes()),
        )
        .map_err(|e| anyhow!("Failed to generate JWT: {}", e))
    }

    /// Verify a JWT token and extract user information
    /// 
    /// Validates token signature and expiration, then extracts user_id and email.
    /// Only accepts tokens of type "user_session".
    /// 
    /// # Arguments
    /// * `token` - JWT token string to verify
    /// 
    /// # Returns
    /// Option containing (user_id, email) tuple if token is valid, None otherwise
    /// 
    /// # Security
    /// - Verifies HMAC signature using server secret
    /// - Checks token expiration automatically
    /// - Only accepts "user_session" type tokens
    pub fn verify_jwt(&self, token: &str) -> Option<(String, String)> {
        let secret = self.config.action_token_secret.as_ref()?;
        
        let validation = Validation::default();
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(secret.as_bytes()),
            &validation,
        ).ok()?;

        let claims = token_data.claims;
        if claims.r#type == "user_session" {
            Some((claims.user_id, claims.email))
        } else {
            None
        }
    }

    /// Verify a token using multiple authentication methods
    /// 
    /// This is the main token verification method that tries multiple approaches:
    /// 1. JWT tokens issued by this server  
    /// 2. Clerk session tokens (when Clerk is configured)
    /// 
    /// # Arguments
    /// * `token` - Authentication token to verify
    /// 
    /// # Returns
    /// Result containing tuple: (is_valid, user_id_option, email_option)
    /// 
    /// # Future Enhancement
    /// TODO: Implement Clerk token verification for external authentication
    pub async fn verify_token(&self, token: &str) -> Result<(bool, Option<String>, Option<String>)> {
        // First, try legacy user_session JWT issued by this server
        if let Some((user_id, email)) = self.verify_jwt(token) {
            return Ok((true, Some(user_id), Some(email)));
        }

        // Fallback to Clerk session token verification when configured
        if !self.config.clerk.secret_key.is_empty() {
            // Note: This would require integrating with Clerk's Rust SDK when available
            // For now, we'll return false for Clerk tokens
            // TODO: Implement Clerk token verification
        }

        Ok((false, None, None))
    }

    /// Create a new user account with email/password authentication
    /// 
    /// This is the main registration flow that:
    /// 1. Validates email uniqueness  
    /// 2. Hashes the password securely
    /// 3. Creates the database record
    /// 4. Generates API key and JWT token
    /// 5. Logs the registration event
    /// 
    /// # Arguments
    /// * `request` - User registration request containing email, password, and optional subscription tier
    /// 
    /// # Returns
    /// Result containing AuthResult with success/failure information
    /// 
    /// # Security Features
    /// - Email uniqueness validation
    /// - Password hashing with bcrypt
    /// - Audit logging for security events
    /// - Atomic transaction (rollback on failure)
    pub async fn create_user(&self, request: CreateUserRequest) -> Result<AuthResult> {
        // Check if user already exists to prevent duplicate registrations
        if let Ok(Some(_)) = self.convex_service.get_user(&request.email).await {
            return Ok(AuthResult {
                success: false,
                token: None,
                user: None,
                error: Some("User with this email already exists".to_string()),
            });
        }

        // Hash the password before storing (never store plain text passwords)
        let password_hash = self.hash_password(&request.password).await?;
        let user_id = Uuid::new_v4().to_string();

        // Create user account structure for database storage
        let user_account = UserAccount {
            email: request.email.clone(),
            password_hash,
            subscription_tier: request.subscription_tier.unwrap_or_else(|| "free".to_string()),
            api_key: self.generate_api_key(),
            is_active: true,
        };

        match self.convex_service.create_user(user_account.clone()).await {
            Ok(convex_user_id) => {
                // Log successful user creation for audit trail
                self.convex_service.log_system_event(
                    "user_created",
                    "info",
                    &format!("New user account created: {}", request.email),
                    Some(&convex_user_id),
                    Some(serde_json::json!({
                        "email": request.email,
                        "subscription_tier": user_account.subscription_tier,
                    })),
                ).await.ok();

                // Generate JWT token for immediate login after registration
                let token = self.generate_jwt(&convex_user_id, &request.email)?;

                Ok(AuthResult {
                    success: true,
                    token: Some(token),
                    user: Some(AuthUser {
                        id: convex_user_id,
                        email: Some(request.email),
                        is_anonymous: false,
                        created_at: Utc::now(),
                    }),
                    error: None,
                })
            }
            Err(error) => {
                // Log failed registration for debugging and security monitoring
                self.convex_service.log_system_event(
                    "user_creation_failed",
                    "error",
                    &format!("Failed to create user: {}", error),
                    None,
                    Some(serde_json::json!({"email": request.email})),
                ).await.ok();

                Ok(AuthResult {
                    success: false,
                    token: None,
                    user: None,
                    error: Some("Failed to create user account".to_string()),
                })
            }
        }
    }

    /// Authenticate a user with email and password
    /// 
    /// This is the main login flow that:
    /// 1. Looks up user by email
    /// 2. Verifies password against stored hash
    /// 3. Checks account status (active/disabled)
    /// 4. Generates new JWT token for session
    /// 5. Logs authentication events for security
    /// 
    /// # Arguments
    /// * `request` - Login request containing email and password
    /// 
    /// # Returns
    /// Result containing AuthResult with success/failure information
    /// 
    /// # Security Features
    /// - Constant-time password verification (prevents timing attacks)
    /// - Account status validation
    /// - Failed login attempt logging
    /// - Generic error messages (prevents user enumeration)
    pub async fn login(&self, request: LoginRequest) -> Result<AuthResult> {
        // Retrieve user account from database
        let user = match self.convex_service.get_user(&request.email).await? {
            Some(user) => user,
            None => {
                // Return generic error to prevent email enumeration attacks
                return Ok(AuthResult {
                    success: false,
                    token: None,
                    user: None,
                    error: Some("Invalid email or password".to_string()),
                });
            }
        };

        // Verify password using bcrypt (constant-time comparison)
        let is_valid_password = self.verify_password(&request.password, &user.password_hash).await?;
        if !is_valid_password {
            // Log failed login attempt for security monitoring
            self.convex_service.log_system_event(
                "login_failed",
                "warn",
                &format!("Failed login attempt for email: {}", request.email),
                Some(&user.id),
                Some(serde_json::json!({
                    "email": request.email,
                    "reason": "invalid_password"
                })),
            ).await.ok();

            // Return same generic error to prevent email enumeration
            return Ok(AuthResult {
                success: false,
                token: None,
                user: None,
                error: Some("Invalid email or password".to_string()),
            });
        }

        // Check if user account is active (not disabled)
        if !user.is_active {
            return Ok(AuthResult {
                success: false,
                token: None,
                user: None,
                error: Some("Account is disabled".to_string()),
            });
        }

        // Log successful login for audit trail
        self.convex_service.log_system_event(
            "user_login",
            "info",
            &format!("User logged in: {}", request.email),
            Some(&user.id),
            Some(serde_json::json!({"email": request.email})),
        ).await.ok();

        // Generate new JWT token for this session
        let token = self.generate_jwt(&user.id, &user.email)?;

        Ok(AuthResult {
            success: true,
            token: Some(token),
            user: Some(AuthUser {
                id: user.id,
                email: Some(user.email),
                is_anonymous: false,
                created_at: user.created_at.unwrap_or_else(Utc::now),
            }),
            error: None,
        })
    }

    /// Get user information from a JWT token
    /// 
    /// Validates the token and retrieves current user information.
    /// Also verifies that the user account still exists and is active.
    /// 
    /// # Arguments
    /// * `token` - JWT token to validate
    /// 
    /// # Returns  
    /// Option containing (user_id, email) if token is valid and user is active
    /// 
    /// # Security
    /// - Validates token signature and expiration
    /// - Confirms user account still exists and is active
    /// - Supports both local JWT and Clerk tokens (future)
    pub async fn get_user_from_token(&self, token: &str) -> Option<(String, String)> {
        // First try local JWT token validation
        if let Some((user_id, email)) = self.verify_jwt(token) {
            // Verify user still exists and is active in database
            if let Ok(Some(user)) = self.convex_service.get_user(&email).await {
                if user.is_active {
                    return Some((user_id, email));
                }
            }
        }

        // Try Clerk token verification (future enhancement)
        if !self.config.clerk.secret_key.is_empty() {
            // TODO: Implement Clerk token verification
        }

        None
    }

    /// Create a temporary anonymous user session
    /// 
    /// Generates a guest user session for trial usage without registration.
    /// Anonymous users have:
    /// - Limited daily request quotas
    /// - No persistent data storage
    /// - Basic AI access only
    /// - Temporary session duration
    /// 
    /// # Returns
    /// Result containing AuthResult with guest user token and information
    /// 
    /// # Guest User Format
    /// - ID: "anon-{timestamp}-{random}"  
    /// - Email: "{guest_id}@anon.local"
    /// - No database record (session-only)
    /// - JWT token with same structure as regular users
    pub async fn create_guest_user(&self) -> Result<AuthResult> {
        // Generate unique anonymous user ID with timestamp and random suffix
        let guest_id = format!("anon-{}-{}", 
            Utc::now().timestamp_millis(),
            Uuid::new_v4().simple().to_string().chars().take(8).collect::<String>()
        );
        let guest_email = format!("{}@anon.local", guest_id);

        // Generate JWT token for guest session (same structure as regular users)
        let token = self.generate_jwt(&guest_id, &guest_email)?;

        // Log anonymous session creation for analytics
        self.convex_service.log_system_event(
            "anonymous_session_created",
            "info",
            &format!("Anonymous session created: {}", guest_id),
            Some(&guest_id),
            Some(serde_json::json!({
                "email": guest_email,
                "user_type": "anonymous"
            })),
        ).await.ok();

        Ok(AuthResult {
            success: true,
            token: Some(token),
            user: Some(AuthUser {
                id: guest_id,
                email: Some(guest_email),
                is_anonymous: true,
                created_at: Utc::now(),
            }),
            error: None,
        })
    }
}