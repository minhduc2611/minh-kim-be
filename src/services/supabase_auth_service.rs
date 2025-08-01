use crate::services::auth_service_trait::{
    AuthServiceError, AuthServiceTrait, AuthUser, LoginRequest, LoginResponse, RefreshTokenRequest,
    SignUpRequest,
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

        let user = AuthUser {
            id: user_data["id"].as_str().unwrap_or_default().to_string(),
            email: user_data["email"].as_str().unwrap_or_default().to_string(),
            name: user_data["user_metadata"]["name"]
                .as_str()
                .map(|s| s.to_string()),
            roles: vec!["user".to_string()], // Default role
        };

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
        let user = AuthUser {
            id: user_data["id"].as_str().unwrap_or_default().to_string(),
            email: user_data["email"].as_str().unwrap_or_default().to_string(),
            name: user_data["user_metadata"]["name"]
                .as_str()
                .map(|s| s.to_string()),
            roles: vec!["user".to_string()], // Default role
        };

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
        let user = AuthUser {
            id: user_data["id"].as_str().unwrap_or_default().to_string(),
            email: user_data["email"].as_str().unwrap_or_default().to_string(),
            name: user_data["user_metadata"]["name"]
                .as_str()
                .map(|s| s.to_string()),
            roles: vec!["user".to_string()],
        };

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
