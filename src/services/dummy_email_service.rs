use crate::services::email_service_trait::{
    EmailConfirmationEmail, EmailServiceError, EmailServiceTrait, PasswordResetConfirmationEmail, PasswordResetEmail,
};
use async_trait::async_trait;
use regex::Regex;

pub struct DummyEmailService;

#[async_trait]
impl EmailServiceTrait for DummyEmailService {
    async fn send_password_reset_email(&self, _request: PasswordResetEmail) -> Result<(), EmailServiceError> {
        Err(EmailServiceError::NotConfigured(
            "Email service not configured. Please set up SMTP credentials.".to_string(),
        ))
    }

    async fn send_password_reset_confirmation_email(&self, _request: PasswordResetConfirmationEmail) -> Result<(), EmailServiceError> {
        Err(EmailServiceError::NotConfigured(
            "Email service not configured. Please set up SMTP credentials.".to_string(),
        ))
    }

    async fn send_email_confirmation(&self, _request: EmailConfirmationEmail) -> Result<(), EmailServiceError> {
        Err(EmailServiceError::NotConfigured(
            "Email service not configured. Please set up SMTP credentials.".to_string(),
        ))
    }

    fn validate_email(&self, email: &str) -> Result<(), EmailServiceError> {
        let email_regex = Regex::new(r"^[^\s@]+@[^\s@]+\.[^\s@]+$")
            .map_err(|e| EmailServiceError::ValidationError(e.to_string()))?;

        if !email_regex.is_match(email) {
            return Err(EmailServiceError::ValidationError(
                "Invalid email format".to_string(),
            ));
        }

        Ok(())
    }

    fn is_configured(&self) -> bool {
        false
    }
} 