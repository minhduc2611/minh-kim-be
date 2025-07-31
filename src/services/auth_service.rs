use crate::services::auth_service_trait::{
    AuthServiceError, AuthServiceTrait, AuthUser, LoginRequest, LoginResponse, RefreshTokenRequest,
};
use crate::services::jwt_weviate_auth_service::BasicJWTWeviateAuthService;
use crate::services::supabase_auth_service::SupabaseAuthService;
use async_trait::async_trait;
use std::sync::Arc;

// Re-export config structs for convenience
pub use crate::services::jwt_weviate_auth_service::BasicJWTWeviateConfig;
pub use crate::services::supabase_auth_service::SupabaseConfig;

/// Example usage:
///
/// ```rust
/// // Using Supabase
/// let supabase_config = SupabaseConfig {
///     url: "https://your-project.supabase.co".to_string(),
///     anon_key: "your-anon-key".to_string(),
///     service_role_key: "your-service-role-key".to_string(),
/// };
/// let auth_service = AuthService::with_supabase(supabase_config);
///
/// // Using BasicJWT + Weviate
/// let jwt_config = BasicJWTWeviateConfig {
///     jwt_secret: "your-secret-key".to_string(),
///     weviate_url: "http://localhost:8080".to_string(),
///     weviate_api_key: "your-weviate-api-key".to_string(),
///     token_expiry_hours: 24,
/// };
/// let auth_service = AuthService::with_basic_jwt_weviate(jwt_config);
///
/// // Login user
/// let login_request = LoginRequest {
///     email: "user@example.com".to_string(),
///     password: "password123".to_string(),
/// };
/// let login_response = auth_service.login(login_request).await?;
///
/// // Verify token
/// let user = auth_service.verify_token(&login_response.access_token).await?;
/// ```

/// Base AuthService struct that can be configured with different implementations
pub struct AuthService {
    implementation: Arc<dyn AuthServiceTrait>,
}

impl AuthService {
    pub fn new(implementation: Arc<dyn AuthServiceTrait>) -> Self {
        Self { implementation }
    }

    /// Create AuthService with Supabase implementation
    pub fn with_supabase(config: SupabaseConfig) -> Self {
        Self {
            implementation: Arc::new(SupabaseAuthService::new(config)),
        }
    }

    /// Create AuthService with BasicJWT and Weviate implementation
    pub fn with_basic_jwt_weviate(config: BasicJWTWeviateConfig) -> Self {
        Self {
            implementation: Arc::new(BasicJWTWeviateAuthService::new(config)),
        }
    }
}

#[async_trait]
impl AuthServiceTrait for AuthService {
    async fn login(&self, request: LoginRequest) -> Result<LoginResponse, AuthServiceError> {
        self.implementation.login(request).await
    }

    async fn verify_token(&self, token: &str) -> Result<AuthUser, AuthServiceError> {
        self.implementation.verify_token(token).await
    }

    async fn refresh_token(
        &self,
        request: RefreshTokenRequest,
    ) -> Result<LoginResponse, AuthServiceError> {
        self.implementation.refresh_token(request).await
    }

    async fn get_user_by_id(&self, user_id: &str) -> Result<AuthUser, AuthServiceError> {
        self.implementation.get_user_by_id(user_id).await
    }

    async fn logout(&self, token: &str) -> Result<(), AuthServiceError> {
        self.implementation.logout(token).await
    }

    fn validate_email(&self, email: &str) -> Result<(), AuthServiceError> {
        self.implementation.validate_email(email)
    }

    fn validate_password(&self, password: &str) -> Result<(), AuthServiceError> {
        self.implementation.validate_password(password)
    }
}
