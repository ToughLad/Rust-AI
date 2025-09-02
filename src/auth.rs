use anyhow::{anyhow, Result};
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::{DateTime, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::config::Config;
use crate::convex_service::{ConvexService, UserAccount};
use crate::types::AuthUser;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUserRequest {
    pub email: String,
    pub password: String,
    pub subscription_tier: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResult {
    pub success: bool,
    pub token: Option<String>,
    pub user: Option<AuthUser>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub user_id: String,
    pub email: String,
    pub r#type: String,
    pub iat: i64,
    pub exp: i64,
}

#[derive(Clone)]
pub struct AuthService {
    config: Config,
    convex_service: ConvexService,
}

impl AuthService {
    pub fn new(config: Config, convex_service: ConvexService) -> Self {
        Self {
            config,
            convex_service,
        }
    }

    pub async fn hash_password(&self, password: &str) -> Result<String> {
        let password = password.to_string();
        tokio::task::spawn_blocking(move || {
            hash(password, DEFAULT_COST).map_err(|e| anyhow!("Failed to hash password: {}", e))
        })
        .await?
    }

    pub async fn verify_password(&self, password: &str, hash: &str) -> Result<bool> {
        let password = password.to_string();
        let hash = hash.to_string();
        tokio::task::spawn_blocking(move || {
            verify(password, &hash).map_err(|e| anyhow!("Failed to verify password: {}", e))
        })
        .await?
    }

    pub fn generate_api_key(&self) -> String {
        format!("ak_{}", Uuid::new_v4().simple())
    }

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
            exp: now + (7 * 24 * 60 * 60), // 7 days
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(secret.as_bytes()),
        )
        .map_err(|e| anyhow!("Failed to generate JWT: {}", e))
    }

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

    pub async fn create_user(&self, request: CreateUserRequest) -> Result<AuthResult> {
        // Check if user already exists
        if let Ok(Some(_)) = self.convex_service.get_user(&request.email).await {
            return Ok(AuthResult {
                success: false,
                token: None,
                user: None,
                error: Some("User with this email already exists".to_string()),
            });
        }

        // Hash password
        let password_hash = self.hash_password(&request.password).await?;
        let user_id = Uuid::new_v4().to_string();

        // Create user account
        let user_account = UserAccount {
            email: request.email.clone(),
            password_hash,
            subscription_tier: request.subscription_tier.unwrap_or_else(|| "free".to_string()),
            api_key: self.generate_api_key(),
            is_active: true,
        };

        match self.convex_service.create_user(user_account.clone()).await {
            Ok(convex_user_id) => {
                // Log system event
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

                // Generate JWT token
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

    pub async fn login(&self, request: LoginRequest) -> Result<AuthResult> {
        // Get user from database
        let user = match self.convex_service.get_user(&request.email).await? {
            Some(user) => user,
            None => {
                return Ok(AuthResult {
                    success: false,
                    token: None,
                    user: None,
                    error: Some("Invalid email or password".to_string()),
                });
            }
        };

        // Verify password
        let is_valid_password = self.verify_password(&request.password, &user.password_hash).await?;
        if !is_valid_password {
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

            return Ok(AuthResult {
                success: false,
                token: None,
                user: None,
                error: Some("Invalid email or password".to_string()),
            });
        }

        // Check if user is active
        if !user.is_active {
            return Ok(AuthResult {
                success: false,
                token: None,
                user: None,
                error: Some("Account is disabled".to_string()),
            });
        }

        // Log successful login
        self.convex_service.log_system_event(
            "user_login",
            "info",
            &format!("User logged in: {}", request.email),
            Some(&user.id),
            Some(serde_json::json!({"email": request.email})),
        ).await.ok();

        // Generate JWT token
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

    pub async fn get_user_from_token(&self, token: &str) -> Option<(String, String)> {
        if let Some((user_id, email)) = self.verify_jwt(token) {
            // Verify user still exists and is active
            if let Ok(Some(user)) = self.convex_service.get_user(&email).await {
                if user.is_active {
                    return Some((user_id, email));
                }
            }
        }

        // Try Clerk token verification
        if !self.config.clerk.secret_key.is_empty() {
            // TODO: Implement Clerk token verification
        }

        None
    }

    pub async fn create_guest_user(&self) -> Result<AuthResult> {
        let guest_id = format!("anon-{}-{}", 
            Utc::now().timestamp_millis(),
            Uuid::new_v4().simple().to_string().chars().take(8).collect::<String>()
        );
        let guest_email = format!("{}@anon.local", guest_id);

        // Generate JWT token for guest
        let token = self.generate_jwt(&guest_id, &guest_email)?;

        // Log anonymous session creation
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