use crate::services::auth_service_trait::{
    AuthServiceError, AuthServiceTrait, AuthUser, LoginRequest, LoginResponse, RefreshTokenRequest,
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
}

#[async_trait]
impl AuthServiceTrait for SupabaseAuthService {
    async fn login(&self, request: LoginRequest) -> Result<LoginResponse, AuthServiceError> {
        // Validate input
        self.validate_email(&request.email)?;
        self.validate_password(&request.password)?;

        // Make request to Supabase Auth API
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
        let user = AuthUser {
            id: user_data["id"].as_str().unwrap_or_default().to_string(),
            email: user_data["email"].as_str().unwrap_or_default().to_string(),
            name: user_data["user_metadata"]["name"]
                .as_str()
                .map(|s| s.to_string()),
            roles: vec!["user".to_string()], // Default role
        };

        Ok(LoginResponse {
            access_token,
            refresh_token: auth_response["refresh_token"]
                .as_str()
                .map(|s| s.to_string()),
            user,
            expires_in: auth_response["expires_in"].as_u64().unwrap_or(3600),
        })
    }

    async fn verify_token(&self, token: &str) -> Result<AuthUser, AuthServiceError> {
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

        Ok(AuthUser {
            id: user_data["id"].as_str().unwrap_or_default().to_string(),
            email: user_data["email"].as_str().unwrap_or_default().to_string(),
            name: user_data["user_metadata"]["name"]
                .as_str()
                .map(|s| s.to_string()),
            roles: vec!["user".to_string()],
        })
    }

    async fn refresh_token(
        &self,
        request: RefreshTokenRequest,
    ) -> Result<LoginResponse, AuthServiceError> {
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
        let user = AuthUser {
            id: user_data["id"].as_str().unwrap_or_default().to_string(),
            email: user_data["email"].as_str().unwrap_or_default().to_string(),
            name: user_data["user_metadata"]["name"]
                .as_str()
                .map(|s| s.to_string()),
            roles: vec!["user".to_string()],
        };

        Ok(LoginResponse {
            access_token,
            refresh_token: auth_response["refresh_token"]
                .as_str()
                .map(|s| s.to_string()),
            user,
            expires_in: auth_response["expires_in"].as_u64().unwrap_or(3600),
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

        Ok(AuthUser {
            id: user_data["id"].as_str().unwrap_or_default().to_string(),
            email: user_data["email"].as_str().unwrap_or_default().to_string(),
            name: user_data["user_metadata"]["name"]
                .as_str()
                .map(|s| s.to_string()),
            roles: vec!["user".to_string()],
        })
    }

    async fn logout(&self, _token: &str) -> Result<(), AuthServiceError> {
        // Supabase handles logout client-side by removing tokens
        // For server-side logout, you might revoke the token if needed
        Ok(())
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
}
