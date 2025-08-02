use crate::services::email_service_trait::{
    EmailConfig, EmailConfirmationEmail, EmailServiceError, EmailServiceTrait, PasswordResetConfirmationEmail,
    PasswordResetEmail,
};
use async_trait::async_trait;
use lettre::{
    transport::smtp::authentication::Credentials,
    Message, SmtpTransport, Transport,
};
use regex::Regex;

pub struct SmtpEmailService {
    config: EmailConfig,
    transport: SmtpTransport,
}

impl SmtpEmailService {
    pub fn new(config: EmailConfig) -> Result<Self, EmailServiceError> {
        // Validate configuration
        if config.smtp_username.is_empty() || config.smtp_password.is_empty() {
            return Err(EmailServiceError::NotConfigured(
                "SMTP credentials not configured".to_string(),
            ));
        }

        // Create SMTP transport
        let transport = SmtpTransport::relay(&config.smtp_server)
            .map_err(|e| EmailServiceError::SmtpError(format!("Failed to create SMTP transport: {}", e)))?
            .credentials(Credentials::new(config.smtp_username.clone(), config.smtp_password.clone()))
            .port(config.smtp_port)
            .build();

        Ok(Self { config, transport })
    }

    fn create_password_reset_html(&self, _email: &str, reset_token: &str, user_name: Option<&str>) -> String {
        let reset_link = format!("{}/reset-password?token={}", self.config.domain_url, reset_token);
        let greeting = user_name.map_or("Hello".to_string(), |name| format!("Hello {}", name));

        format!(
            r#"
            <html>
            <body style="font-family: Arial, sans-serif; max-width: 600px; margin: 0 auto; padding: 20px;">
                <div style="text-align: center; margin-bottom: 30px;">
                    <h1 style="color: #333;">Password Reset Request</h1>
                </div>
                
                <div style="background-color: #f9f9f9; padding: 20px; border-radius: 8px; margin-bottom: 20px;">
                    <p style="color: #666; font-size: 16px; line-height: 1.5;">
                        {}, we received a request to reset your password. If you didn't request this, please ignore this email.
                    </p>
                    
                    <p style="color: #666; font-size: 16px; line-height: 1.5;">
                        To reset your password, click the button below:
                    </p>
                    
                    <div style="text-align: center; margin: 30px 0;">
                        <a href="{}" 
                           style="background-color: #ca460b; color: white; text-decoration: none; 
                                  padding: 12px 24px; border-radius: 6px; display: inline-block; 
                                  font-weight: bold; font-size: 16px;">
                            Reset Password
                        </a>
                    </div>
                    
                    <p style="color: #666; font-size: 14px; line-height: 1.5;">
                        If the button doesn't work, you can also copy and paste this link into your browser:
                    </p>
                    <p style="color: #ca460b; font-size: 14px; word-break: break-all;">
                        {}
                    </p>
                </div>
                
                <div style="border-top: 1px solid #ddd; padding-top: 20px; color: #999; font-size: 12px;">
                    <p>This link will expire in 1 hour for security reasons.</p>
                    <p>If you have any questions, please contact our support team.</p>
                </div>
            </body>
            </html>
            "#,
            greeting, reset_link, reset_link
        )
    }

    fn create_password_reset_plain(&self, _email: &str, reset_token: &str, user_name: Option<&str>) -> String {
        let reset_link = format!("{}/reset-password?token={}", self.config.domain_url, reset_token);
        let greeting = user_name.map_or("Hello".to_string(), |name| format!("Hello {}", name));

        format!(
            r#"
            Password Reset Request
            
            {}, we received a request to reset your password. If you didn't request this, please ignore this email.
            
            To reset your password, visit this link:
            {}
            
            This link will expire in 1 hour for security reasons.
            
            If you have any questions, please contact our support team.
            "#,
            greeting, reset_link
        )
    }

    fn create_password_reset_confirmation_html(&self, user_name: Option<&str>) -> String {
        let greeting = user_name.map_or("Hello".to_string(), |name| format!("Hello {}", name));
        let login_link = format!("{}/login", self.config.domain_url);

        format!(
            r#"
            <html>
            <body style="font-family: Arial, sans-serif; max-width: 600px; margin: 0 auto; padding: 20px;">
                <div style="text-align: center; margin-bottom: 30px;">
                    <h1 style="color: #ca460b;">Password Reset Successful</h1>
                </div>
                
                <div style="background-color: #f9f9f9; padding: 20px; border-radius: 8px; margin-bottom: 20px;">
                    <p style="color: #666; font-size: 16px; line-height: 1.5;">
                        {}, your password has been successfully reset. You can now log in with your new password.
                    </p>
                    
                    <p style="color: #666; font-size: 16px; line-height: 1.5;">
                        If you didn't make this change, please contact our support team immediately.
                    </p>
                    
                    <div style="text-align: center; margin: 30px 0;">
                        <a href="{}" 
                           style="background-color: #ca460b; color: white; text-decoration: none; 
                                  padding: 12px 24px; border-radius: 6px; display: inline-block; 
                                  font-weight: bold; font-size: 16px;">
                            Log In
                        </a>
                    </div>
                </div>
                
                <div style="border-top: 1px solid #ddd; padding-top: 20px; color: #999; font-size: 12px;">
                    <p>For security reasons, please ensure you're using a strong, unique password.</p>
                </div>
            </body>
            </html>
            "#,
            greeting, login_link
        )
    }

    fn create_password_reset_confirmation_plain(&self, user_name: Option<&str>) -> String {
        let greeting = user_name.map_or("Hello".to_string(), |name| format!("Hello {}", name));
        let login_link = format!("{}/login", self.config.domain_url);

        format!(
            r#"
            Password Reset Successful
            
            {}, your password has been successfully reset. You can now log in with your new password.
            
            If you didn't make this change, please contact our support team immediately.
            
            Visit {} to log in.
            
            For security reasons, please ensure you're using a strong, unique password.
            "#,
            greeting, login_link
        )
    }

    fn create_email_confirmation_html(&self, _email: &str, confirmation_token: &str, user_name: Option<&str>) -> String {
        let confirmation_link = format!("{}/confirm-email?token={}", self.config.domain_url, confirmation_token);
        let greeting = user_name.map_or("Hello".to_string(), |name| format!("Hello {}", name));

        format!(
            r#"
            <html>
            <body style="font-family: Arial, sans-serif; max-width: 600px; margin: 0 auto; padding: 20px;">
                <div style="text-align: center; margin-bottom: 30px;">
                    <h1 style="color: #333;">Confirm Your Email Address</h1>
                </div>
                
                <div style="background-color: #f9f9f9; padding: 20px; border-radius: 8px; margin-bottom: 20px;">
                    <p style="color: #666; font-size: 16px; line-height: 1.5;">
                        {}, thank you for signing up! Please confirm your email address to complete your registration.
                    </p>
                    
                    <p style="color: #666; font-size: 16px; line-height: 1.5;">
                        Click the button below to confirm your email address:
                    </p>
                    
                    <div style="text-align: center; margin: 30px 0;">
                        <a href="{}" 
                           style="background-color: #ca460b; color: white; text-decoration: none; 
                                  padding: 12px 24px; border-radius: 6px; display: inline-block; 
                                  font-weight: bold; font-size: 16px;">
                            Confirm Email
                        </a>
                    </div>
                    
                    <p style="color: #666; font-size: 14px; line-height: 1.5;">
                        If the button doesn't work, you can also copy and paste this link into your browser:
                    </p>
                    <p style="color: #ca460b; font-size: 14px; word-break: break-all;">
                        {}
                    </p>
                </div>
                
                <div style="border-top: 1px solid #ddd; padding-top: 20px; color: #999; font-size: 12px;">
                    <p>This link will expire in 24 hours for security reasons.</p>
                    <p>If you have any questions, please contact our support team.</p>
                </div>
            </body>
            </html>
            "#,
            greeting, confirmation_link, confirmation_link
        )
    }

    fn create_email_confirmation_plain(&self, _email: &str, confirmation_token: &str, user_name: Option<&str>) -> String {
        let confirmation_link = format!("{}/confirm-email?token={}", self.config.domain_url, confirmation_token);
        let greeting = user_name.map_or("Hello".to_string(), |name| format!("Hello {}", name));

        format!(
            r#"
            Confirm Your Email Address
            
            {}, thank you for signing up! Please confirm your email address to complete your registration.
            
            To confirm your email address, visit this link:
            {}
            
            This link will expire in 24 hours for security reasons.
            
            If you have any questions, please contact our support team.
            "#,
            greeting, confirmation_link
        )
    }
}

#[async_trait]
impl EmailServiceTrait for SmtpEmailService {
    async fn send_password_reset_email(&self, request: PasswordResetEmail) -> Result<(), EmailServiceError> {
        // Validate email
        self.validate_email(&request.email)?;

        // Create email content
        let html_content = self.create_password_reset_html(&request.email, &request.reset_token, request.user_name.as_deref());
        let _plain_content = self.create_password_reset_plain(&request.email, &request.reset_token, request.user_name.as_deref());

        // Create email message
        let email = Message::builder()
            .from(self.config.from_email.parse().map_err(|e| {
                EmailServiceError::SmtpError(format!("Invalid from email: {}", e))
            })?)
            .to(request.email.parse().map_err(|e| {
                EmailServiceError::SmtpError(format!("Invalid to email: {}", e))
            })?)
            .subject("Password Reset Request")
            .header(lettre::message::header::ContentType::TEXT_HTML)
            .body(html_content)
            .map_err(|e| EmailServiceError::SmtpError(format!("Failed to create email message: {}", e)))?;

        // Send email
        self.transport
            .send(&email)
            .map_err(|e| EmailServiceError::SmtpError(format!("Failed to send email: {}", e)))?;

        Ok(())
    }

    async fn send_password_reset_confirmation_email(&self, request: PasswordResetConfirmationEmail) -> Result<(), EmailServiceError> {
        // Validate email
        self.validate_email(&request.email)?;

        // Create email content
        let html_content = self.create_password_reset_confirmation_html(request.user_name.as_deref());
        let _plain_content = self.create_password_reset_confirmation_plain(request.user_name.as_deref());

        // Create email message
        let email = Message::builder()
            .from(self.config.from_email.parse().map_err(|e| {
                EmailServiceError::SmtpError(format!("Invalid from email: {}", e))
            })?)
            .to(request.email.parse().map_err(|e| {
                EmailServiceError::SmtpError(format!("Invalid to email: {}", e))
            })?)
            .subject("Password Successfully Reset")
            .header(lettre::message::header::ContentType::TEXT_HTML)
            .body(html_content)
            .map_err(|e| EmailServiceError::SmtpError(format!("Failed to create email message: {}", e)))?;

        // Send email
        self.transport
            .send(&email)
            .map_err(|e| EmailServiceError::SmtpError(format!("Failed to send email: {}", e)))?;

        Ok(())
    }

    async fn send_email_confirmation(&self, request: EmailConfirmationEmail) -> Result<(), EmailServiceError> {
        // Validate email
        self.validate_email(&request.email)?;

        // Create email content
        let html_content = self.create_email_confirmation_html(&request.email, &request.confirmation_token, request.user_name.as_deref());
        let _plain_content = self.create_email_confirmation_plain(&request.email, &request.confirmation_token, request.user_name.as_deref());

        // Create email message
        let email = Message::builder()
            .from(self.config.from_email.parse().map_err(|e| {
                EmailServiceError::SmtpError(format!("Invalid from email: {}", e))
            })?)
            .to(request.email.parse().map_err(|e| {
                EmailServiceError::SmtpError(format!("Invalid to email: {}", e))
            })?)
            .subject("Confirm Your Email Address")
            .header(lettre::message::header::ContentType::TEXT_HTML)
            .body(html_content)
            .map_err(|e| EmailServiceError::SmtpError(format!("Failed to create email message: {}", e)))?;

        // Send email
        self.transport
            .send(&email)
            .map_err(|e| EmailServiceError::SmtpError(format!("Failed to send email: {}", e)))?;

        Ok(())
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
        !self.config.smtp_username.is_empty() && !self.config.smtp_password.is_empty()
    }
} 