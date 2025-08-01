use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, thiserror::Error)]
pub enum AuthServiceError {
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),
    #[error("Invalid token: {0}")]
    InvalidToken(String),
    #[error("Token expired")]
    TokenExpired,
    #[error("Unauthorized")]
    #[allow(dead_code)]
    Unauthorized,
    #[error("User not found")]
    UserNotFound,
    #[error("External service error: {0}")]
    ExternalServiceError(String),
    #[error("Validation error: {0}")]
    ValidationError(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthUser {
    pub id: String,
    pub email: String,
    pub name: Option<String>,
    pub roles: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignUpRequest {
    pub email: String,
    pub password: String,
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginResponse {
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
    pub user: AuthUser,
    pub expires_in: u64,
    pub email_confirmation_pending: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}

#[async_trait]
pub trait AuthServiceTrait: Send + Sync {
    /// Sign up new user with email and password
    async fn sign_up(&self, request: SignUpRequest) -> Result<LoginResponse, AuthServiceError>;

    /// Authenticate user with email and password
    async fn login(&self, request: LoginRequest) -> Result<LoginResponse, AuthServiceError>;

    /// Verify and decode access token
    async fn verify_token(&self, token: &str) -> Result<AuthUser, AuthServiceError>;

    /// Refresh access token using refresh token
    async fn refresh_token(
        &self,
        request: RefreshTokenRequest,
    ) -> Result<LoginResponse, AuthServiceError>;

    /// Get user information by user ID
    async fn get_user_by_id(&self, user_id: &str) -> Result<AuthUser, AuthServiceError>;

    /// Logout user (invalidate tokens)
    async fn logout(&self, token: &str) -> Result<(), AuthServiceError>;

    /// Validate email format
    fn validate_email(&self, email: &str) -> Result<(), AuthServiceError>;

    /// Validate password strength
    fn validate_password(&self, password: &str) -> Result<(), AuthServiceError>;
}
