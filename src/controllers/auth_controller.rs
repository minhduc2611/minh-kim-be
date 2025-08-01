use crate::services::auth_service_trait::{
    AuthServiceError, AuthServiceTrait, LoginRequest, RefreshTokenRequest, SignUpRequest,
};
use actix_web::{get, post, web, HttpRequest, HttpResponse, Responder, Result};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub message: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserIdParam {
    pub user_id: String,
}

/// POST /auth/signup - Sign up new user with email and password
/// (AuthFlow-email-signup 2) Frontend ->> Supabase: supabase.auth.signUp(email, password)
#[post("/auth/signup")]
pub async fn signup(
    service: web::Data<Arc<dyn AuthServiceTrait>>,
    request: web::Json<SignUpRequest>,
) -> Result<impl Responder> {
    match service.sign_up(request.into_inner()).await {
        Ok(login_response) => {
            // Check if email confirmation is pending
            if login_response.email_confirmation_pending.unwrap_or(false) {
                // (AuthFlow-email-signup 3) Email confirmation required
                Ok(HttpResponse::Created().json(AuthResponse {
                    success: true,
                    data: Some(json!({
                        "user": login_response.user,
                        "email_confirmation_pending": true,
                        "message": "Please check your email and click the confirmation link to complete registration"
                    })),
                    message: Some("User registered successfully. Email confirmation required.".to_string()),
                    error: None,
                }))
            } else {
                // (AuthFlow-email-signup 5) Supabase -->> Frontend: JWT tokens (access & refresh)
                Ok(HttpResponse::Created().json(AuthResponse {
                    success: true,
                    data: Some(json!({
                        "access_token": login_response.access_token,
                        "refresh_token": login_response.refresh_token,
                        "user": login_response.user,
                        "expires_in": login_response.expires_in
                    })),
                    message: Some("User registered successfully".to_string()),
                    error: None,
                }))
            }
        }
        Err(AuthServiceError::ValidationError(msg)) => {
            Ok(HttpResponse::BadRequest().json(AuthResponse::<()> {
                success: false,
                data: None,
                message: Some(msg),
                error: Some("ValidationError".to_string()),
            }))
        }
        Err(AuthServiceError::AuthenticationFailed(msg)) => {
            Ok(HttpResponse::BadRequest().json(AuthResponse::<()> {
                success: false,
                data: None,
                message: Some(msg),
                error: Some("AuthenticationFailed".to_string()),
            }))
        }
        Err(AuthServiceError::ExternalServiceError(msg)) => Ok(HttpResponse::InternalServerError()
            .json(AuthResponse::<()> {
                success: false,
                data: None,
                message: Some("Internal server error".to_string()),
                error: Some(format!("ExternalServiceError: {}", msg)),
            })),
        Err(err) => Ok(
            HttpResponse::InternalServerError().json(AuthResponse::<()> {
                success: false,
                data: None,
                message: Some("Internal server error".to_string()),
                error: Some(format!("{:?}", err)),
            }),
        ),
    }
}

/// POST /auth/login - Authenticate user with email and password
/// (AuthFlow-email-login 2) Frontend ->> Supabase: supabase.auth.signIn(email, password)
#[post("/auth/login")]
pub async fn login(
    service: web::Data<Arc<dyn AuthServiceTrait>>,
    request: web::Json<LoginRequest>,
) -> Result<impl Responder> {
    match service.login(request.into_inner()).await {
        Ok(login_response) => {
            // (AuthFlow-email-login 5) Supabase -->> Frontend: JWT tokens (access & refresh)
            Ok(HttpResponse::Ok().json(AuthResponse {
                success: true,
                data: Some(json!({
                    "access_token": login_response.access_token,
                    "refresh_token": login_response.refresh_token,
                    "user": login_response.user,
                    "expires_in": login_response.expires_in
                })),
                message: Some("Login successful".to_string()),
                error: None,
            }))
        }
        Err(AuthServiceError::ValidationError(msg)) => {
            Ok(HttpResponse::BadRequest().json(AuthResponse::<()> {
                success: false,
                data: None,
                message: Some(msg),
                error: Some("ValidationError".to_string()),
            }))
        }
        Err(AuthServiceError::AuthenticationFailed(msg)) => {
            Ok(HttpResponse::Unauthorized().json(AuthResponse::<()> {
                success: false,
                data: None,
                message: Some(msg),
                error: Some("AuthenticationFailed".to_string()),
            }))
        }
        Err(AuthServiceError::ExternalServiceError(msg)) => Ok(HttpResponse::InternalServerError()
            .json(AuthResponse::<()> {
                success: false,
                data: None,
                message: Some("Internal server error".to_string()),
                error: Some(format!("ExternalServiceError: {}", msg)),
            })),
        Err(err) => Ok(
            HttpResponse::InternalServerError().json(AuthResponse::<()> {
                success: false,
                data: None,
                message: Some("Internal server error".to_string()),
                error: Some(format!("{:?}", err)),
            }),
        ),
    }
}

/// GET /auth/verify - Verify access token and get user info
/// (AuthFlow-email-signup 7) Backend ->> Supabase: auth.getUser(token)
#[get("/auth/verify")]
pub async fn verify_token(
    service: web::Data<Arc<dyn AuthServiceTrait>>,
    req: HttpRequest,
) -> Result<impl Responder> {
    // Extract Bearer token from Authorization header
    let auth_header = req
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "));

    let token = match auth_header {
        Some(token) => token,
        None => {
            return Ok(HttpResponse::Unauthorized().json(AuthResponse::<()> {
                success: false,
                data: None,
                message: Some("Missing or invalid Authorization header".to_string()),
                error: Some("MissingToken".to_string()),
            }));
        }
    };

    match service.verify_token(token).await {
        Ok(user) => {
            // (AuthFlow-email-signup 8) Supabase -->> Backend: Valid user data
            Ok(HttpResponse::Ok().json(AuthResponse {
                success: true,
                data: Some(user),
                message: Some("Token verified successfully".to_string()),
                error: None,
            }))
        }
        Err(AuthServiceError::InvalidToken(msg)) => {
            Ok(HttpResponse::Unauthorized().json(AuthResponse::<()> {
                success: false,
                data: None,
                message: Some(msg),
                error: Some("InvalidToken".to_string()),
            }))
        }
        Err(AuthServiceError::TokenExpired) => {
            Ok(HttpResponse::Unauthorized().json(AuthResponse::<()> {
                success: false,
                data: None,
                message: Some("Token has expired".to_string()),
                error: Some("TokenExpired".to_string()),
            }))
        }
        Err(AuthServiceError::ExternalServiceError(msg)) => Ok(HttpResponse::InternalServerError()
            .json(AuthResponse::<()> {
                success: false,
                data: None,
                message: Some("Internal server error".to_string()),
                error: Some(format!("ExternalServiceError: {}", msg)),
            })),
        Err(err) => Ok(
            HttpResponse::InternalServerError().json(AuthResponse::<()> {
                success: false,
                data: None,
                message: Some("Internal server error".to_string()),
                error: Some(format!("{:?}", err)),
            }),
        ),
    }
}

/// POST /auth/refresh - Refresh access token using refresh token
#[post("/auth/refresh")]
pub async fn refresh_token(
    service: web::Data<Arc<dyn AuthServiceTrait>>,
    request: web::Json<RefreshTokenRequest>,
) -> Result<impl Responder> {
    match service.refresh_token(request.into_inner()).await {
        Ok(login_response) => Ok(HttpResponse::Ok().json(AuthResponse {
            success: true,
            data: Some(json!({
                "access_token": login_response.access_token,
                "refresh_token": login_response.refresh_token,
                "user": login_response.user,
                "expires_in": login_response.expires_in
            })),
            message: Some("Token refreshed successfully".to_string()),
            error: None,
        })),
        Err(AuthServiceError::InvalidToken(msg)) => {
            Ok(HttpResponse::Unauthorized().json(AuthResponse::<()> {
                success: false,
                data: None,
                message: Some(msg),
                error: Some("InvalidToken".to_string()),
            }))
        }
        Err(AuthServiceError::ExternalServiceError(msg)) => Ok(HttpResponse::InternalServerError()
            .json(AuthResponse::<()> {
                success: false,
                data: None,
                message: Some("Internal server error".to_string()),
                error: Some(format!("ExternalServiceError: {}", msg)),
            })),
        Err(err) => Ok(
            HttpResponse::InternalServerError().json(AuthResponse::<()> {
                success: false,
                data: None,
                message: Some("Internal server error".to_string()),
                error: Some(format!("{:?}", err)),
            }),
        ),
    }
}

/// POST /auth/logout - Logout user (invalidate tokens)
#[post("/auth/logout")]
pub async fn logout(
    service: web::Data<Arc<dyn AuthServiceTrait>>,
    req: HttpRequest,
) -> Result<impl Responder> {
    // Extract Bearer token from Authorization header
    let auth_header = req
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "));

    let token = match auth_header {
        Some(token) => token,
        None => {
            return Ok(HttpResponse::Unauthorized().json(AuthResponse::<()> {
                success: false,
                data: None,
                message: Some("Missing or invalid Authorization header".to_string()),
                error: Some("MissingToken".to_string()),
            }));
        }
    };

    match service.logout(token).await {
        Ok(()) => Ok(HttpResponse::Ok().json(AuthResponse::<()> {
            success: true,
            data: None,
            message: Some("Logout successful".to_string()),
            error: None,
        })),
        Err(AuthServiceError::ExternalServiceError(msg)) => Ok(HttpResponse::InternalServerError()
            .json(AuthResponse::<()> {
                success: false,
                data: None,
                message: Some("Internal server error".to_string()),
                error: Some(format!("ExternalServiceError: {}", msg)),
            })),
        Err(err) => Ok(
            HttpResponse::InternalServerError().json(AuthResponse::<()> {
                success: false,
                data: None,
                message: Some("Internal server error".to_string()),
                error: Some(format!("{:?}", err)),
            }),
        ),
    }
}

