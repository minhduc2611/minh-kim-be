use crate::services::email_service_trait::{
    EmailConfig, EmailConfirmationEmail, EmailServiceError, EmailServiceTrait, PasswordResetConfirmationEmail,
    PasswordResetEmail,
};
use crate::services::smtp_email_service::SmtpEmailService;
use async_trait::async_trait;
use std::sync::Arc;

pub struct EmailService {
    implementation: Arc<dyn EmailServiceTrait>,
}

impl EmailService {
    pub fn new(implementation: Arc<dyn EmailServiceTrait>) -> Self {
        Self { implementation }
    }

    /// Create EmailService with SMTP implementation
    pub fn with_smtp(config: EmailConfig) -> Result<Self, EmailServiceError> {
        let smtp_service = SmtpEmailService::new(config)?;
        Ok(Self {
            implementation: Arc::new(smtp_service),
        })
    }
}

#[async_trait]
impl EmailServiceTrait for EmailService {
    async fn send_password_reset_email(&self, request: PasswordResetEmail) -> Result<(), EmailServiceError> {
        self.implementation.send_password_reset_email(request).await
    }

    async fn send_password_reset_confirmation_email(&self, request: PasswordResetConfirmationEmail) -> Result<(), EmailServiceError> {
        self.implementation.send_password_reset_confirmation_email(request).await
    }

    async fn send_email_confirmation(&self, request: EmailConfirmationEmail) -> Result<(), EmailServiceError> {
        self.implementation.send_email_confirmation(request).await
    }

    fn validate_email(&self, email: &str) -> Result<(), EmailServiceError> {
        self.implementation.validate_email(email)
    }

    fn is_configured(&self) -> bool {
        self.implementation.is_configured()
    }
} 