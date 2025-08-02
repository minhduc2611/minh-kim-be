use crate::services::email_service_trait::{
    EmailConfirmationEmail, EmailServiceError, EmailServiceTrait, PasswordResetConfirmationEmail, PasswordResetEmail,
};
use actix_web::{post, web, HttpResponse, Responder, Result};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize)]
pub struct EmailResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub message: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PasswordResetRequest {
    pub email: String,
    pub user_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PasswordResetConfirmationRequest {
    pub email: String,
    pub user_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EmailConfirmationRequest {
    pub email: String,
    pub user_name: Option<String>,
}

/// POST /email/password-reset - Send password reset email
#[post("/email/password-reset")]
pub async fn send_password_reset_email(
    service: web::Data<Arc<dyn EmailServiceTrait>>,
    request: web::Json<PasswordResetRequest>,
) -> Result<impl Responder> {
    // Check if email service is configured
    if !service.is_configured() {
        return Ok(HttpResponse::ServiceUnavailable().json(EmailResponse::<()> {
            success: false,
            data: None,
            message: Some("Email service not configured".to_string()),
            error: Some("ServiceUnavailable".to_string()),
        }));
    }

    // Generate a reset token (in production, you'd use a proper JWT or UUID)
    let reset_token = uuid::Uuid::new_v4().to_string();

    let email_request = PasswordResetEmail {
        email: request.email.clone(),
        reset_token,
        user_name: request.user_name.clone(),
    };

    match service.send_password_reset_email(email_request).await {
        Ok(()) => Ok(HttpResponse::Ok().json(EmailResponse {
            success: true,
            data: Some(json!({
                "message": "Password reset email sent successfully"
            })),
            message: Some("Password reset email sent".to_string()),
            error: None,
        })),
        Err(EmailServiceError::ValidationError(msg)) => {
            Ok(HttpResponse::BadRequest().json(EmailResponse::<()> {
                success: false,
                data: None,
                message: Some(msg),
                error: Some("ValidationError".to_string()),
            }))
        }
        Err(EmailServiceError::NotConfigured(msg)) => {
            Ok(HttpResponse::ServiceUnavailable().json(EmailResponse::<()> {
                success: false,
                data: None,
                message: Some(msg),
                error: Some("NotConfigured".to_string()),
            }))
        }
        Err(EmailServiceError::SmtpError(msg)) => {
            Ok(HttpResponse::InternalServerError().json(EmailResponse::<()> {
                success: false,
                data: None,
                message: Some("Failed to send email".to_string()),
                error: Some(format!("SmtpError: {}", msg)),
            }))
        }
        Err(EmailServiceError::ExternalServiceError(msg)) => {
            Ok(HttpResponse::InternalServerError().json(EmailResponse::<()> {
                success: false,
                data: None,
                message: Some("Email service error".to_string()),
                error: Some(format!("ExternalServiceError: {}", msg)),
            }))
        }
    }
}

/// POST /email/password-reset-confirmation - Send password reset confirmation email
#[post("/email/password-reset-confirmation")]
pub async fn send_password_reset_confirmation_email(
    service: web::Data<Arc<dyn EmailServiceTrait>>,
    request: web::Json<PasswordResetConfirmationRequest>,
) -> Result<impl Responder> {
    // Check if email service is configured
    if !service.is_configured() {
        return Ok(HttpResponse::ServiceUnavailable().json(EmailResponse::<()> {
            success: false,
            data: None,
            message: Some("Email service not configured".to_string()),
            error: Some("ServiceUnavailable".to_string()),
        }));
    }

    let email_request = PasswordResetConfirmationEmail {
        email: request.email.clone(),
        user_name: request.user_name.clone(),
    };

    match service.send_password_reset_confirmation_email(email_request).await {
        Ok(()) => Ok(HttpResponse::Ok().json(EmailResponse {
            success: true,
            data: Some(json!({
                "message": "Password reset confirmation email sent successfully"
            })),
            message: Some("Password reset confirmation email sent".to_string()),
            error: None,
        })),
        Err(EmailServiceError::ValidationError(msg)) => {
            Ok(HttpResponse::BadRequest().json(EmailResponse::<()> {
                success: false,
                data: None,
                message: Some(msg),
                error: Some("ValidationError".to_string()),
            }))
        }
        Err(EmailServiceError::NotConfigured(msg)) => {
            Ok(HttpResponse::ServiceUnavailable().json(EmailResponse::<()> {
                success: false,
                data: None,
                message: Some(msg),
                error: Some("NotConfigured".to_string()),
            }))
        }
        Err(EmailServiceError::SmtpError(msg)) => {
            Ok(HttpResponse::InternalServerError().json(EmailResponse::<()> {
                success: false,
                data: None,
                message: Some("Failed to send email".to_string()),
                error: Some(format!("SmtpError: {}", msg)),
            }))
        }
        Err(EmailServiceError::ExternalServiceError(msg)) => {
            Ok(HttpResponse::InternalServerError().json(EmailResponse::<()> {
                success: false,
                data: None,
                message: Some("Email service error".to_string()),
                error: Some(format!("ExternalServiceError: {}", msg)),
            }))
        }
    }
}

/// POST /email/confirmation - Send email confirmation for new user registration
#[post("/email/confirmation")]
pub async fn send_email_confirmation(
    service: web::Data<Arc<dyn EmailServiceTrait>>,
    request: web::Json<EmailConfirmationRequest>,
) -> Result<impl Responder> {
    // Check if email service is configured
    if !service.is_configured() {
        return Ok(HttpResponse::ServiceUnavailable().json(EmailResponse::<()> {
            success: false,
            data: None,
            message: Some("Email service not configured".to_string()),
            error: Some("ServiceUnavailable".to_string()),
        }));
    }

    // Generate a confirmation token (in production, you'd use a proper JWT or UUID)
    let confirmation_token = uuid::Uuid::new_v4().to_string();

    let email_request = EmailConfirmationEmail {
        email: request.email.clone(),
        confirmation_token,
        user_name: request.user_name.clone(),
    };

    match service.send_email_confirmation(email_request).await {
        Ok(()) => Ok(HttpResponse::Ok().json(EmailResponse {
            success: true,
            data: Some(json!({
                "message": "Email confirmation sent successfully"
            })),
            message: Some("Email confirmation sent".to_string()),
            error: None,
        })),
        Err(EmailServiceError::ValidationError(msg)) => {
            Ok(HttpResponse::BadRequest().json(EmailResponse::<()> {
                success: false,
                data: None,
                message: Some(msg),
                error: Some("ValidationError".to_string()),
            }))
        }
        Err(EmailServiceError::NotConfigured(msg)) => {
            Ok(HttpResponse::ServiceUnavailable().json(EmailResponse::<()> {
                success: false,
                data: None,
                message: Some(msg),
                error: Some("NotConfigured".to_string()),
            }))
        }
        Err(EmailServiceError::SmtpError(msg)) => {
            Ok(HttpResponse::InternalServerError().json(EmailResponse::<()> {
                success: false,
                data: None,
                message: Some("Failed to send email".to_string()),
                error: Some(format!("SmtpError: {}", msg)),
            }))
        }
        Err(EmailServiceError::ExternalServiceError(msg)) => {
            Ok(HttpResponse::InternalServerError().json(EmailResponse::<()> {
                success: false,
                data: None,
                message: Some("Email service error".to_string()),
                error: Some(format!("ExternalServiceError: {}", msg)),
            }))
        }
    }
} 