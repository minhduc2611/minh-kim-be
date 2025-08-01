use crate::services::auth_service_trait::{
    AuthServiceError, AuthServiceTrait, AuthUser, LoginRequest, LoginResponse, RefreshTokenRequest,
    SignUpRequest,
};
use crate::services::jwt_weviate_auth_service::BasicJWTWeviateAuthService;
use crate::services::supabase_auth_service::SupabaseAuthService;
use async_trait::async_trait;
use std::sync::Arc;

// Re-export config structs for convenience
pub use crate::services::jwt_weviate_auth_service::BasicJWTWeviateConfig;
pub use crate::services::supabase_auth_service::SupabaseConfig;

/// Example usage following the Supabase auth flow:
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
/// // (AuthFlow-email-signup 1) User ->> Frontend: Sign up with email & password
/// let signup_request = SignUpRequest {
///     email: "user@example.com".to_string(),
///     password: "password123".to_string(),
///     name: Some("John Doe".to_string()),
/// };
/// // (AuthFlow-email-signup 2) Frontend ->> Supabase: supabase.auth.signUp(email, password)
/// // (AuthFlow-email-signup 3) Supabase -->> User: Email confirmation link (optional)
/// // (AuthFlow-email-signup 5) Supabase -->> Frontend: JWT tokens (access & refresh)
/// let signup_response = auth_service.sign_up(signup_request).await?;
///
/// // (AuthFlow-email-signup 6) Frontend ->> Backend: Bearer access_token in header
/// // (AuthFlow-email-signup 7) Backend ->> Supabase: auth.getUser(token)
/// // (AuthFlow-email-signup 8) Supabase -->> Backend: Valid user data
/// let user = auth_service.verify_token(&signup_response.access_token).await?;
/// // (AuthFlow-email-signup 9) Backend -->> Frontend: Protected resource
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
    async fn sign_up(&self, request: SignUpRequest) -> Result<LoginResponse, AuthServiceError> {
        self.implementation.sign_up(request).await
    }

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
