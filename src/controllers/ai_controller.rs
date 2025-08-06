use crate::middleware::AuthenticatedUser;
use crate::services::vertex_ai_service::ChatRequest;
use crate::services::vertex_ai_service_trait::{VertexAIServiceTrait, VertexAIServiceError};
use actix_web::{post, web, HttpResponse, Responder, Result};
use serde_json::json;
use std::sync::Arc;

/// POST /api/v1/ai - Generate AI content using Vertex AI - REQUIRES AUTHENTICATION
#[post("/api/v1/ai")]
pub async fn generate_ai_content(
    _authenticated_user: AuthenticatedUser,
    service: web::Data<Arc<dyn VertexAIServiceTrait>>,
    req: web::Json<ChatRequest>,
) -> Result<impl Responder> {
    // Generate content using Vertex AI service
    match service.chat(&req).await {
        Ok(chat_response) => {
            Ok(HttpResponse::Ok().json(json!({
                "success": true,
                "data": chat_response,
                "pagination": null,
                "message": "AI content generated successfully",
                "error": null
            })))
        },
        Err(VertexAIServiceError::GenerationFailed(msg)) => {
            Ok(HttpResponse::InternalServerError().json(json!({
                "success": false,
                "data": null,
                "pagination": null,
                "message": msg,
                "error": "GenerationError"
            })))
        }
        Err(VertexAIServiceError::ConfigurationError(msg)) => {
            Ok(HttpResponse::BadRequest().json(json!({
                "success": false,
                "data": null,
                "pagination": null,
                "message": msg,
                "error": "ConfigurationError"
            })))
        }
        Err(VertexAIServiceError::ApiError(msg)) => {
            Ok(HttpResponse::InternalServerError().json(json!({
                "success": false,
                "data": null,
                "pagination": null,
                "message": msg,
                "error": "ApiError"
            })))
        }
        Err(VertexAIServiceError::AgentNotFound(msg)) => {
            Ok(HttpResponse::BadRequest().json(json!({
                "success": false,
                "data": null,
                "pagination": null,
                "message": msg,
                "error": "AgentNotFound"
            })))
        }
    }
} 