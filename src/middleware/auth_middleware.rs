use crate::services::auth_service_trait::{AuthServiceError, AuthServiceTrait, AuthUser};
use actix_web::{
    dev::Payload, error::ErrorUnauthorized, web, Error, FromRequest, HttpRequest,
};
use serde_json::json;
use std::{future::Future, pin::Pin, sync::Arc};

/// Authenticated user extractor
/// 
/// Usage in handlers:
/// ```rust
/// pub async fn protected_endpoint(
///     authenticated_user: AuthenticatedUser,
///     // ... other parameters
/// ) -> Result<impl Responder> {
///     let user = authenticated_user.user;
///     // Use user.id, user.email, etc.
/// }
/// ```
#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub user: AuthUser,
}

impl FromRequest for AuthenticatedUser {
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        let req = req.clone();
        
        Box::pin(async move {
            // Extract Authorization header
            let auth_header = req
                .headers()
                .get("Authorization")
                .and_then(|h| h.to_str().ok())
                .and_then(|h| h.strip_prefix("Bearer "));

            let token = match auth_header {
                Some(token) => token,
                None => {
                    return Err(ErrorUnauthorized(
                        json!({
                            "success": false,
                            "data": null,
                            "message": "Missing or invalid Authorization header. Please provide: Authorization: Bearer <your_token>",
                            "error": "MissingToken"
                        })
                        .to_string(),
                    ));
                }
            };

            // Get auth service from app data
            let auth_service = req
                .app_data::<web::Data<Arc<dyn AuthServiceTrait>>>()
                .ok_or_else(|| {
                    ErrorUnauthorized(
                        json!({
                            "success": false,
                            "data": null,
                            "message": "Authentication service not available",
                            "error": "ServiceUnavailable"
                        })
                        .to_string(),
                    )
                })?;

            // Verify token and get user
            match auth_service.verify_token(token).await {
                Ok(user) => Ok(AuthenticatedUser { user }),
                Err(AuthServiceError::InvalidToken(msg)) => Err(ErrorUnauthorized(
                    json!({
                        "success": false,
                        "data": null,
                        "message": msg,
                        "error": "InvalidToken"
                    })
                    .to_string(),
                )),
                Err(AuthServiceError::TokenExpired) => Err(ErrorUnauthorized(
                    json!({
                        "success": false,
                        "data": null,
                        "message": "Token has expired. Please refresh your token or login again.",
                        "error": "TokenExpired"
                    })
                    .to_string(),
                )),
                Err(AuthServiceError::ExternalServiceError(msg)) => Err(ErrorUnauthorized(
                    json!({
                        "success": false,
                        "data": null,
                        "message": "Authentication service error",
                        "error": format!("ExternalServiceError: {}", msg)
                    })
                    .to_string(),
                )),
                Err(err) => Err(ErrorUnauthorized(
                    json!({
                        "success": false,
                        "data": null,
                        "message": "Authentication failed",
                        "error": format!("{:?}", err)
                    })
                    .to_string(),
                )),
            }
        })
    }
}
