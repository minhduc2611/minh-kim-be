use crate::services::auth_service_trait::{
    AuthServiceError, AuthServiceTrait, AuthUser, ForgotPasswordRequest, LoginRequest, LoginResponse, OAuthTokenRequest,
    RefreshTokenRequest, ResetPasswordRequest, SignUpRequest,
};
use async_trait::async_trait;
use regex::Regex;

#[derive(Debug, Clone)]
pub struct SupabaseConfig {
    pub url: String,
    pub anon_key: String,
    pub service_role_key: String,
}

pub struct SupabaseAuthService {
    config: SupabaseConfig,
    client: reqwest::Client,
}

impl SupabaseAuthService {
    pub fn new(config: SupabaseConfig) -> Self {
        Self {
            config,
            client: reqwest::Client::new(),
        }
    }

    /// Helper function to extract comprehensive user data from Supabase user object
    fn extract_user_data(&self, user_data: &serde_json::Value) -> AuthUser {
        // Extract providers from app_metadata
        let providers = user_data["app_metadata"]["providers"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();
        
        // Extract user metadata
        let user_metadata = &user_data["user_metadata"];
        
        AuthUser {
            id: user_data["id"].as_str().unwrap_or_default().to_string(),
            email: user_data["email"].as_str().unwrap_or_default().to_string(),
            name: user_metadata["name"].as_str().map(|s| s.to_string()),
            full_name: user_metadata["full_name"].as_str().map(|s| s.to_string()),
            avatar_url: user_metadata["avatar_url"].as_str().map(|s| s.to_string()),
            email_verified: user_metadata["email_verified"].as_bool(),
            phone: user_data["phone"].as_str().map(|s| s.to_string()),
            phone_verified: user_metadata["phone_verified"].as_bool(),
            role: user_data["role"].as_str().map(|s| s.to_string()),
            providers,
            last_sign_in_at: user_data["last_sign_in_at"].as_str().map(|s| s.to_string()),
            created_at: user_data["created_at"].as_str().map(|s| s.to_string()),
            updated_at: user_data["updated_at"].as_str().map(|s| s.to_string()),
            confirmed_at: user_data["confirmed_at"].as_str().map(|s| s.to_string()),
            email_confirmed_at: user_data["email_confirmed_at"].as_str().map(|s| s.to_string()),
            is_anonymous: user_data["is_anonymous"].as_bool(),
            roles: vec!["user".to_string()], // Default role
        }
    }
}

#[async_trait]
impl AuthServiceTrait for SupabaseAuthService {
    // (AuthFlow-email-signup 2) Frontend ->> Supabase: supabase.auth.signUp(email, password)
    async fn sign_up(&self, request: SignUpRequest) -> Result<LoginResponse, AuthServiceError> {
        // Validate input
        self.validate_email(&request.email)?;
        self.validate_password(&request.password)?;

        // (AuthFlow-email-signup 2) Make request to Supabase Auth API for signup
        let url = format!("{}/auth/v1/signup", self.config.url);

        let mut signup_data = serde_json::json!({
            "email": request.email,
            "password": request.password
        });

        // Add name to user metadata if provided
        if let Some(name) = request.name {
            signup_data["data"] = serde_json::json!({
                "name": name
            });
        }

        let response = self
            .client
            .post(&url)
            .header("apikey", &self.config.anon_key)
            .header("Content-Type", "application/json")
            .json(&signup_data)
            .send()
            .await
            .map_err(|e| AuthServiceError::ExternalServiceError(e.to_string()))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(AuthServiceError::AuthenticationFailed(error_text));
        }

        // Parse response
        let auth_response: serde_json::Value = response
            .json()
            .await
            .map_err(|e| AuthServiceError::ExternalServiceError(e.to_string()))?;

        println!("auth_response: {:?}", auth_response);
        println!("================================================");
        // (AuthFlow-email-signup 3) Supabase -->> User: Email confirmation link (optional)
        // Note: If email confirmation is enabled, user will receive confirmation email
        // The tokens are not included until email is confirmed

        // Check if email confirmation is pending by looking for confirmation_sent_at field
        let email_confirmation_pending = auth_response["confirmation_sent_at"].is_string();

        // Extract token and user info from Supabase response
        let access_token = auth_response["access_token"]
            .as_str()
            .map(|s| s.to_string());

        // User data is directly in the response when email confirmation is pending
        // but nested under "user" when tokens are included
        let user_data = if email_confirmation_pending {
            &auth_response
        } else {
            &auth_response["user"]
        };

        let user = self.extract_user_data(user_data);

        // (AuthFlow-email-signup 5) Supabase -->> Frontend: JWT tokens (access & refresh) or confirmation pending
        Ok(LoginResponse {
            access_token,
            refresh_token: auth_response["refresh_token"]
                .as_str()
                .map(|s| s.to_string()),
            user,
            expires_in: auth_response["expires_in"].as_u64().unwrap_or(3600),
            email_confirmation_pending: Some(email_confirmation_pending),
        })
    }

    async fn login(&self, request: LoginRequest) -> Result<LoginResponse, AuthServiceError> {
        // Validate input
        self.validate_email(&request.email)?;
        self.validate_password(&request.password)?;

        // (AuthFlow-email-login 2) Frontend ->> Supabase: supabase.auth.signIn(email, password)
        // Make request to Supabase Auth API for login
        let url = format!("{}/auth/v1/token?grant_type=password", self.config.url);

        let response = self
            .client
            .post(&url)
            .header("apikey", &self.config.anon_key)
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({
                "email": request.email,
                "password": request.password
            }))
            .send()
            .await
            .map_err(|e| AuthServiceError::ExternalServiceError(e.to_string()))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(AuthServiceError::AuthenticationFailed(error_text));
        }

        // Parse response (simplified - you'd parse actual Supabase response structure)
        let auth_response: serde_json::Value = response
            .json()
            .await
            .map_err(|e| AuthServiceError::ExternalServiceError(e.to_string()))?;

        // Extract token and user info from Supabase response
        let access_token = auth_response["access_token"]
            .as_str()
            .ok_or_else(|| {
                AuthServiceError::ExternalServiceError("Missing access token".to_string())
            })?
            .to_string();

        let user_data = &auth_response["user"];
        
        let user = self.extract_user_data(user_data);

        // (AuthFlow-email-login 5) Supabase -->> Frontend: JWT tokens (access & refresh)
        Ok(LoginResponse {
            access_token: Some(access_token),
            refresh_token: auth_response["refresh_token"]
                .as_str()
                .map(|s| s.to_string()),
            user,
            expires_in: auth_response["expires_in"].as_u64().unwrap_or(3600),
            email_confirmation_pending: Some(false),
        })
    }

    async fn verify_token(&self, token: &str) -> Result<AuthUser, AuthServiceError> {
        // (AuthFlow-email-signup 7) Backend ->> Supabase: auth.getUser(token)
        // Get user info from Supabase using the access token
        let url = format!("{}/auth/v1/user", self.config.url);

        let response = self
            .client
            .get(&url)
            .header("apikey", &self.config.anon_key)
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
            .map_err(|e| AuthServiceError::ExternalServiceError(e.to_string()))?;

        if response.status() == 401 {
            return Err(AuthServiceError::InvalidToken(
                "Token is invalid or expired".to_string(),
            ));
        }

        if !response.status().is_success() {
            return Err(AuthServiceError::ExternalServiceError(
                "Failed to verify token".to_string(),
            ));
        }

        let user_data: serde_json::Value = response
            .json()
            .await
            .map_err(|e| AuthServiceError::ExternalServiceError(e.to_string()))?;

        // (AuthFlow-email-signup 8) Supabase -->> Backend: Valid user data
        
        Ok(self.extract_user_data(&user_data))
    }

    async fn refresh_token(
        &self,
        request: RefreshTokenRequest,
    ) -> Result<LoginResponse, AuthServiceError> {
        // Refresh expired access token using refresh token
        let url = format!("{}/auth/v1/token?grant_type=refresh_token", self.config.url);

        let response = self
            .client
            .post(&url)
            .header("apikey", &self.config.anon_key)
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({
                "refresh_token": request.refresh_token
            }))
            .send()
            .await
            .map_err(|e| AuthServiceError::ExternalServiceError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(AuthServiceError::InvalidToken(
                "Invalid refresh token".to_string(),
            ));
        }

        // Parse and return new tokens (similar to login response parsing)
        let auth_response: serde_json::Value = response
            .json()
            .await
            .map_err(|e| AuthServiceError::ExternalServiceError(e.to_string()))?;

        let access_token = auth_response["access_token"]
            .as_str()
            .ok_or_else(|| {
                AuthServiceError::ExternalServiceError("Missing access token".to_string())
            })?
            .to_string();

        let user_data = &auth_response["user"];
        
        let user = self.extract_user_data(user_data);

        // Return new JWT tokens (access & refresh)
        Ok(LoginResponse {
            access_token: Some(access_token),
            refresh_token: auth_response["refresh_token"]
                .as_str()
                .map(|s| s.to_string()),
            user,
            expires_in: auth_response["expires_in"].as_u64().unwrap_or(3600),
            email_confirmation_pending: Some(false),
        })
    }

    async fn get_user_by_id(&self, user_id: &str) -> Result<AuthUser, AuthServiceError> {
        let url = format!("{}/rest/v1/auth.users", self.config.url);

        let response = self
            .client
            .get(&url)
            .header("apikey", &self.config.service_role_key)
            .header(
                "Authorization",
                format!("Bearer {}", self.config.service_role_key),
            )
            .query(&[("id", format!("eq.{}", user_id))])
            .send()
            .await
            .map_err(|e| AuthServiceError::ExternalServiceError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(AuthServiceError::UserNotFound);
        }

        let users: Vec<serde_json::Value> = response
            .json()
            .await
            .map_err(|e| AuthServiceError::ExternalServiceError(e.to_string()))?;

        let user_data = users.first().ok_or(AuthServiceError::UserNotFound)?;

        Ok(self.extract_user_data(&user_data))
    }

    async fn logout(&self, _token: &str) -> Result<(), AuthServiceError> {
        // Supabase handles logout client-side by removing tokens
        // For server-side logout, you might revoke the token if needed
        Ok(())
    }

    async fn verify_oauth_token(&self, request: OAuthTokenRequest) -> Result<AuthUser, AuthServiceError> {
        // (AuthFlow-google-signup-login 8) Frontend ->> Backend: Sends access_token
        // For Supabase OAuth, the token verification is the same as regular token verification
        // The OAuth flow happens client-side, and we just verify the resulting Supabase JWT
        
        // Validate that this is a supported provider
        let supported_providers = vec!["google", "github", "facebook", "twitter"];
        if !supported_providers.contains(&request.provider.as_str()) {
            return Err(AuthServiceError::ValidationError(
                format!("Unsupported OAuth provider: {}", request.provider)
            ));
        }

        // Verify the token using the same method as regular tokens
        // Note: Supabase OAuth tokens are standard Supabase JWT tokens after OAuth flow
        self.verify_token(&request.access_token).await
    }

    fn validate_email(&self, email: &str) -> Result<(), AuthServiceError> {
        let email_regex = Regex::new(r"^[^\s@]+@[^\s@]+\.[^\s@]+$")
            .map_err(|e| AuthServiceError::ValidationError(e.to_string()))?;

        if !email_regex.is_match(email) {
            return Err(AuthServiceError::ValidationError(
                "Invalid email format".to_string(),
            ));
        }

        Ok(())
    }

    fn validate_password(&self, password: &str) -> Result<(), AuthServiceError> {
        if password.len() < 8 {
            return Err(AuthServiceError::ValidationError(
                "Password must be at least 8 characters".to_string(),
            ));
        }
        Ok(())
    }

    async fn forgot_password(&self, request: ForgotPasswordRequest) -> Result<(), AuthServiceError> {
        // Validate email format
        self.validate_email(&request.email)?;

        // Make request to Supabase Auth API for password reset
        let url = format!("{}/auth/v1/recover", self.config.url);

        let response = self
            .client
            .post(&url)
            .header("apikey", &self.config.anon_key)
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({
                "email": request.email,
            }))
            .send()
            .await
            .map_err(|e| AuthServiceError::ExternalServiceError(e.to_string()))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(AuthServiceError::ExternalServiceError(error_text));
        }

        Ok(())
    }

    async fn reset_password(&self, request: ResetPasswordRequest, token: &str) -> Result<(), AuthServiceError> {
        // Validate password
        self.validate_password(&request.password)?;

        // Make request to Supabase Auth API for password update
        let url = format!("{}/auth/v1/user", self.config.url);

        let response = self
            .client
            .put(&url)
            .header("apikey", &self.config.anon_key)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({
                "password": request.password
            }))
            .send()
            .await
            .map_err(|e| AuthServiceError::ExternalServiceError(e.to_string()))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(AuthServiceError::ExternalServiceError(error_text));
        }

        Ok(())
    }
}
