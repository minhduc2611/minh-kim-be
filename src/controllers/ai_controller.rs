use crate::middleware::AuthenticatedUser;
use crate::services::vertex_ai_service_trait::{VertexAIServiceTrait, VertexAIServiceError, ChatRequest};
use crate::services::ai_service::GenerateKeywordsRequest;
use crate::services::ai_service_trait::{AIServiceTrait, AIServiceError};
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

/// POST /api/v1/ai/generate-keywords - Generate keywords for a topic - REQUIRES AUTHENTICATION
#[post("/api/v1/ai/generate-keywords")]
pub async fn generate_keywords(
    _authenticated_user: AuthenticatedUser,
    service: web::Data<Arc<dyn AIServiceTrait>>,
    req: web::Json<GenerateKeywordsRequest>,
) -> Result<impl Responder> {
    // Generate keywords using AI service
    match service.generate_keywords(req.into_inner()).await {
        Ok(response) => {
            Ok(HttpResponse::Ok().json(json!({
                "success": true,
                "data": response,
                "pagination": null,
                "message": "Keywords generated successfully",
                "error": null
            })))
        },
        Err(AIServiceError::CanvasNotFound(msg)) => {
            Ok(HttpResponse::NotFound().json(json!({
                "success": false,
                "data": null,
                "pagination": null,
                "message": msg,
                "error": "CanvasNotFound"
            })))
        },
        Err(AIServiceError::TopicNotFound(msg)) => {
            Ok(HttpResponse::NotFound().json(json!({
                "success": false,
                "data": null,
                "pagination": null,
                "message": msg,
                "error": "TopicNotFound"
            })))
        },
        Err(e) => {
            Ok(HttpResponse::InternalServerError().json(json!({
                "success": false,
                "data": null,
                "pagination": null,
                "message": e.to_string(),
                "error": "GenerationError"
            })))
        }
    }
} 