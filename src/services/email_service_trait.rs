use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, thiserror::Error)]
pub enum EmailServiceError {
    #[error("Email service not configured: {0}")]
    NotConfigured(String),
    #[error("SMTP error: {0}")]
    SmtpError(String),
    #[error("Email validation error: {0}")]
    ValidationError(String),
    #[error("External service error: {0}")]
    ExternalServiceError(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailConfig {
    pub smtp_server: String,
    pub smtp_port: u16,
    pub smtp_username: String,
    pub smtp_password: String,
    pub from_email: String,
    pub domain_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordResetEmail {
    pub email: String,
    pub reset_token: String,
    pub user_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordResetConfirmationEmail {
    pub email: String,
    pub user_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailConfirmationEmail {
    pub email: String,
    pub confirmation_token: String,
    pub user_name: Option<String>,
}

#[async_trait]
pub trait EmailServiceTrait: Send + Sync {
    /// Send password reset email with magic link
    async fn send_password_reset_email(&self, request: PasswordResetEmail) -> Result<(), EmailServiceError>;

    /// Send confirmation email after password reset
    async fn send_password_reset_confirmation_email(&self, request: PasswordResetConfirmationEmail) -> Result<(), EmailServiceError>;

    /// Send email confirmation for new user registration
    async fn send_email_confirmation(&self, request: EmailConfirmationEmail) -> Result<(), EmailServiceError>;

    /// Validate email format
    fn validate_email(&self, email: &str) -> Result<(), EmailServiceError>;

    /// Check if email service is properly configured
    fn is_configured(&self) -> bool;
} 