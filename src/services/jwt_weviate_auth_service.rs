use crate::services::auth_service_trait::{
    AuthServiceError, AuthServiceTrait, AuthUser, LoginRequest, LoginResponse, RefreshTokenRequest,
    SignUpRequest,
};
use async_trait::async_trait;
use regex::Regex;

#[derive(Debug, Clone)]
pub struct BasicJWTWeviateConfig {
    pub jwt_secret: String,
    pub weviate_url: String,
    pub weviate_api_key: String,
    pub token_expiry_hours: u64,
}

pub struct BasicJWTWeviateAuthService {
    config: BasicJWTWeviateConfig,
    client: reqwest::Client,
}

impl BasicJWTWeviateAuthService {
    pub fn new(config: BasicJWTWeviateConfig) -> Self {
        Self {
            config,
            client: reqwest::Client::new(),
        }
    }

    // Helper method to create JWT token
    fn create_jwt_token(&self, user: &AuthUser) -> Result<String, AuthServiceError> {
        // This is a simplified JWT creation - in production you'd use a proper JWT library
        // like `jsonwebtoken` crate
        use base64::{engine::general_purpose, Engine as _};

        let header = general_purpose::STANDARD.encode(r#"{"alg":"HS256","typ":"JWT"}"#);
        let payload = general_purpose::STANDARD.encode(&serde_json::to_string(&serde_json::json!({
            "sub": user.id,
            "email": user.email,
            "name": user.name,
            "roles": user.roles,
            "exp": chrono::Utc::now().timestamp() + (self.config.token_expiry_hours as i64 * 3600)
        })).map_err(|e| AuthServiceError::ExternalServiceError(e.to_string()))?);

        let message = format!("{}.{}", header, payload);
        let signature = self.sign_message(&message)?;

        Ok(format!("{}.{}", message, signature))
    }

    fn sign_message(&self, message: &str) -> Result<String, AuthServiceError> {
        // Simplified HMAC-SHA256 signing - use proper crypto library in production
        use base64::{engine::general_purpose, Engine as _};
        use hmac::{Hmac, Mac};
        use sha2::Sha256;

        type HmacSha256 = Hmac<Sha256>;

        let mut mac = HmacSha256::new_from_slice(self.config.jwt_secret.as_bytes())
            .map_err(|e| AuthServiceError::ValidationError(e.to_string()))?;
        mac.update(message.as_bytes());
        let result = mac.finalize();
        Ok(general_purpose::STANDARD.encode(result.into_bytes()))
    }

    async fn authenticate_with_weviate(
        &self,
        email: &str,
        password: &str,
    ) -> Result<AuthUser, AuthServiceError> {
        // Query Weviate for user with matching email and password hash
        let query = serde_json::json!({
            "query": format!(r#"
                {{
                    Get {{
                        User(where: {{
                            path: ["email"],
                            operator: Equal,
                            valueString: "{}"
                        }}) {{
                            id
                            email
                            name
                            passwordHash
                            roles
                        }}
                    }}
                }}
            "#, email)
        });

        let response = self
            .client
            .post(&format!("{}/v1/graphql", self.config.weviate_url))
            .header(
                "Authorization",
                format!("Bearer {}", self.config.weviate_api_key),
            )
            .header("Content-Type", "application/json")
            .json(&query)
            .send()
            .await
            .map_err(|e| AuthServiceError::ExternalServiceError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(AuthServiceError::ExternalServiceError(
                "Weviate query failed".to_string(),
            ));
        }

        let result: serde_json::Value = response
            .json()
            .await
            .map_err(|e| AuthServiceError::ExternalServiceError(e.to_string()))?;

        let users = result["data"]["Get"]["User"]
            .as_array()
            .ok_or(AuthServiceError::UserNotFound)?;

        let user_data = users.first().ok_or(AuthServiceError::UserNotFound)?;

        // Verify password hash (simplified - use proper password hashing in production)
        let stored_hash = user_data["passwordHash"].as_str().unwrap_or_default();
        if !self.verify_password_hash(password, stored_hash)? {
            return Err(AuthServiceError::AuthenticationFailed(
                "Invalid credentials".to_string(),
            ));
        }

        Ok(AuthUser {
            id: user_data["id"].as_str().unwrap_or_default().to_string(),
            email: user_data["email"].as_str().unwrap_or_default().to_string(),
            name: user_data["name"].as_str().map(|s| s.to_string()),
            roles: user_data["roles"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect()
                })
                .unwrap_or_else(|| vec!["user".to_string()]),
        })
    }

    fn verify_password_hash(&self, password: &str, hash: &str) -> Result<bool, AuthServiceError> {
        // Simplified password verification - use proper hashing library like bcrypt in production
        let computed_hash = format!("hash:{}", password); // This is NOT secure, just for demo
        Ok(computed_hash == hash)
    }
}

#[async_trait]
impl AuthServiceTrait for BasicJWTWeviateAuthService {
    async fn sign_up(&self, request: SignUpRequest) -> Result<LoginResponse, AuthServiceError> {
        // Validate input
        self.validate_email(&request.email)?;
        self.validate_password(&request.password)?;

        // Create user in Weviate
        let user_id = uuid::Uuid::new_v4().to_string();
        let password_hash = format!("hash:{}", request.password); // Simplified hashing - use bcrypt in production

        let name = request.name.clone().unwrap_or_default();
        let mutation = serde_json::json!({
            "query": format!(r#"
                mutation {{
                    createUser(input: {{
                        id: "{}"
                        email: "{}"
                        name: "{}"
                        passwordHash: "{}"
                        roles: ["user"]
                    }}) {{
                        id
                        email
                        name
                        roles
                    }}
                }}
            "#, 
            user_id,
            request.email,
            name,
            password_hash
            )
        });

        let response = self
            .client
            .post(&format!("{}/v1/graphql", self.config.weviate_url))
            .header(
                "Authorization",
                format!("Bearer {}", self.config.weviate_api_key),
            )
            .header("Content-Type", "application/json")
            .json(&mutation)
            .send()
            .await
            .map_err(|e| AuthServiceError::ExternalServiceError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(AuthServiceError::ExternalServiceError(
                "Failed to create user in Weviate".to_string(),
            ));
        }

        // Create user object for token generation
        let user = AuthUser {
            id: user_id,
            email: request.email,
            name: request.name,
            roles: vec!["user".to_string()],
        };

        // Create JWT token
        let access_token = self.create_jwt_token(&user)?;

        Ok(LoginResponse {
            access_token: Some(access_token),
            refresh_token: None, // Basic JWT doesn't typically use refresh tokens
            user,
            expires_in: self.config.token_expiry_hours * 3600,
            email_confirmation_pending: Some(false), // JWT auth doesn't require email confirmation
        })
    }

    async fn login(&self, request: LoginRequest) -> Result<LoginResponse, AuthServiceError> {
        // Validate input
        self.validate_email(&request.email)?;
        self.validate_password(&request.password)?;

        // Authenticate with Weviate
        let user = self
            .authenticate_with_weviate(&request.email, &request.password)
            .await?;

        // Create JWT token
        let access_token = self.create_jwt_token(&user)?;

        Ok(LoginResponse {
            access_token: Some(access_token),
            refresh_token: None, // Basic JWT doesn't typically use refresh tokens
            user,
            expires_in: self.config.token_expiry_hours * 3600,
            email_confirmation_pending: Some(false), // JWT auth doesn't require email confirmation
        })
    }

    async fn verify_token(&self, token: &str) -> Result<AuthUser, AuthServiceError> {
        // Parse JWT token (simplified - use proper JWT library in production)
        let parts: Vec<&str> = token.split('.').collect();
        if parts.len() != 3 {
            return Err(AuthServiceError::InvalidToken(
                "Invalid token format".to_string(),
            ));
        }

        // Verify signature
        let message = format!("{}.{}", parts[0], parts[1]);
        let expected_signature = self.sign_message(&message)?;
        if expected_signature != parts[2] {
            return Err(AuthServiceError::InvalidToken(
                "Invalid token signature".to_string(),
            ));
        }

        // Decode payload
        use base64::{engine::general_purpose, Engine as _};
        let payload_bytes = general_purpose::STANDARD
            .decode(parts[1])
            .map_err(|e| AuthServiceError::InvalidToken(e.to_string()))?;
        let payload: serde_json::Value = serde_json::from_slice(&payload_bytes)
            .map_err(|e| AuthServiceError::InvalidToken(e.to_string()))?;

        // Check expiration
        let exp = payload["exp"].as_i64().ok_or_else(|| {
            AuthServiceError::InvalidToken("Missing expiration claim".to_string())
        })?;

        if chrono::Utc::now().timestamp() > exp {
            return Err(AuthServiceError::TokenExpired);
        }

        // Extract user info
        Ok(AuthUser {
            id: payload["sub"].as_str().unwrap_or_default().to_string(),
            email: payload["email"].as_str().unwrap_or_default().to_string(),
            name: payload["name"].as_str().map(|s| s.to_string()),
            roles: payload["roles"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect()
                })
                .unwrap_or_else(|| vec!["user".to_string()]),
        })
    }

    async fn refresh_token(
        &self,
        _request: RefreshTokenRequest,
    ) -> Result<LoginResponse, AuthServiceError> {
        // Basic JWT implementation doesn't support refresh tokens
        Err(AuthServiceError::ExternalServiceError(
            "Refresh tokens not supported in BasicJWT implementation".to_string(),
        ))
    }

    async fn get_user_by_id(&self, user_id: &str) -> Result<AuthUser, AuthServiceError> {
        // Query Weviate for user by ID
        let query = serde_json::json!({
            "query": format!(r#"
                {{
                    Get {{
                        User(where: {{
                            path: ["id"],
                            operator: Equal,
                            valueString: "{}"
                        }}) {{
                            id
                            email
                            name
                            roles
                        }}
                    }}
                }}
            "#, user_id)
        });

        let response = self
            .client
            .post(&format!("{}/v1/graphql", self.config.weviate_url))
            .header(
                "Authorization",
                format!("Bearer {}", self.config.weviate_api_key),
            )
            .header("Content-Type", "application/json")
            .json(&query)
            .send()
            .await
            .map_err(|e| AuthServiceError::ExternalServiceError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(AuthServiceError::ExternalServiceError(
                "Weviate query failed".to_string(),
            ));
        }

        let result: serde_json::Value = response
            .json()
            .await
            .map_err(|e| AuthServiceError::ExternalServiceError(e.to_string()))?;

        let users = result["data"]["Get"]["User"]
            .as_array()
            .ok_or(AuthServiceError::UserNotFound)?;

        let user_data = users.first().ok_or(AuthServiceError::UserNotFound)?;

        Ok(AuthUser {
            id: user_data["id"].as_str().unwrap_or_default().to_string(),
            email: user_data["email"].as_str().unwrap_or_default().to_string(),
            name: user_data["name"].as_str().map(|s| s.to_string()),
            roles: user_data["roles"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect()
                })
                .unwrap_or_else(|| vec!["user".to_string()]),
        })
    }

    async fn logout(&self, _token: &str) -> Result<(), AuthServiceError> {
        // JWT tokens are stateless, so logout is typically handled client-side
        // You could implement a token blacklist here if needed
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
