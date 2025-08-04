use crate::middleware::AuthenticatedUser;
use crate::models::canvas::{CreateCanvasRequest, GetCanvasesRequest, UpdateCanvasRequest};
use crate::models::common::ListCanvasQuery;
use crate::services::canvas_service_trait::{CanvasServiceError, CanvasServiceTrait};
use actix_web::{delete, get, post, put, web, HttpResponse, Responder, Result};

use serde_json::json;
use std::sync::Arc;

/// GET /canvas - Get all canvases (list view) - REQUIRES AUTHENTICATION
#[get("/api/v1/canvas")]
pub async fn get_canvas_list(
    authenticated_user: AuthenticatedUser,
    service: web::Data<Arc<dyn CanvasServiceTrait>>,
    query: web::Query<ListCanvasQuery>,
) -> Result<impl Responder> {
    // If author_id is provided, filter by author, otherwise use authenticated user's ID
    let author_id = query.author_id.clone().unwrap_or(authenticated_user.user.id.clone());
    let request = GetCanvasesRequest {
        author_id,
        limit: query.limit,
        offset: query.offset,
    };

    match service.get_canvases(request).await {
        Ok(paginated_response) => Ok(HttpResponse::Ok().json(json!({
            "success": true,
            "data": paginated_response.data,
            "pagination": {
                "total": paginated_response.pagination.total,
                "limit": paginated_response.pagination.limit,
                "offset": paginated_response.pagination.offset,
                "current_page": paginated_response.pagination.current_page,
                "total_pages": paginated_response.pagination.total_pages,
                "has_next": paginated_response.pagination.has_next,
                "has_previous": paginated_response.pagination.has_previous
            },
            "message": null,
            "error": null
        }))),
        Err(CanvasServiceError::ValidationError(msg)) => {
            Ok(HttpResponse::BadRequest().json(json!({
                "success": false,
                "data": null,
                "pagination": null,
                "message": msg,
                "error": "ValidationError"
            })))
        }
        Err(CanvasServiceError::DatabaseError(msg)) => Ok(HttpResponse::InternalServerError()
            .json(json!({
                "success": false,
                "data": null,
                "pagination": null,
                "message": msg,
                "error": "DatabaseError"
            }))),
        Err(CanvasServiceError::NotFound) => Ok(HttpResponse::Ok().json(json!({
            "success": true,
            "data": [],
            "pagination": {
                "total": 0,
                "limit": query.limit.unwrap_or(50),
                "offset": query.offset.unwrap_or(0),
                "current_page": 1,
                "total_pages": 0,
                "has_next": false,
                "has_previous": false
            },
            "message": "No canvases found for this author",
            "error": null
        }))),
    }
}

/// POST /canvas - Create a new canvas - REQUIRES AUTHENTICATION
#[post("/api/v1/canvas")]
pub async fn create_canvas(
    authenticated_user: AuthenticatedUser,
    service: web::Data<Arc<dyn CanvasServiceTrait>>,
    mut req: web::Json<CreateCanvasRequest>,
) -> Result<impl Responder> {
    // Set the author_id to the authenticated user's ID
    req.author_id = authenticated_user.user.id.clone();
    match service.create_canvas(req.into_inner()).await {
        Ok(canvas) => Ok(HttpResponse::Created().json(json!({
            "success": true,
            "data": canvas,
            "pagination": null,
            "message": "Canvas created successfully",
            "error": null
        }))),
        Err(CanvasServiceError::ValidationError(msg)) => {
            Ok(HttpResponse::BadRequest().json(json!({
                "success": false,
                "data": null,
                "pagination": null,
                "message": msg,
                "error": "ValidationError"
            })))
        }
        Err(CanvasServiceError::DatabaseError(msg)) => Ok(HttpResponse::InternalServerError()
            .json(json!({
                "success": false,
                "data": null,
                "pagination": null,
                "message": msg,
                "error": "DatabaseError"
            }))),
        Err(CanvasServiceError::NotFound) => Ok(HttpResponse::NotFound().json(json!({
            "success": false,
            "data": null,
            "pagination": null,
            "message": "Canvas not found",
            "error": "NotFound"
        }))),
    }
}

/// GET /canvas/{id} - Get canvas by ID - REQUIRES AUTHENTICATION
#[get("/api/v1/canvas/{id}")]
pub async fn get_canvas(
    _authenticated_user: AuthenticatedUser,
    service: web::Data<Arc<dyn CanvasServiceTrait>>,
    path: web::Path<String>,
) -> Result<impl Responder> {
    let canvas_id = path.into_inner();

    match service.get_canvas_by_id(&canvas_id).await {
        Ok(canvas) => Ok(HttpResponse::Ok().json(json!({
            "success": true,
            "data": canvas,
            "pagination": null,
            "message": null,
            "error": null
        }))),
        Err(CanvasServiceError::NotFound) => Ok(HttpResponse::NotFound().json(json!({
            "success": false,
            "data": null,
            "pagination": null,
            "message": "Canvas not found",
            "error": "NotFound"
        }))),
        Err(CanvasServiceError::ValidationError(msg)) => {
            Ok(HttpResponse::BadRequest().json(json!({
                "success": false,
                "data": null,
                "pagination": null,
                "message": msg,
                "error": "ValidationError"
            })))
        }
        Err(CanvasServiceError::DatabaseError(msg)) => Ok(HttpResponse::InternalServerError()
            .json(json!({
                "success": false,
                "data": null,
                "pagination": null,
                "message": msg,
                "error": "DatabaseError"
            }))),
    }
}

/// PUT /canvas/{id} - Update canvas - REQUIRES AUTHENTICATION
#[put("/api/v1/canvas/{id}")]
pub async fn update_canvas(
    _authenticated_user: AuthenticatedUser,
    service: web::Data<Arc<dyn CanvasServiceTrait>>,
    path: web::Path<String>,
    req: web::Json<UpdateCanvasRequest>,
) -> Result<impl Responder> {
    let canvas_id = path.into_inner();

    match service.update_canvas(&canvas_id, req.into_inner()).await {
        Ok(canvas) => Ok(HttpResponse::Ok().json(json!({
            "success": true,
            "data": canvas,
            "pagination": null,
            "message": "Canvas updated successfully",
            "error": null
        }))),
        Err(CanvasServiceError::NotFound) => Ok(HttpResponse::NotFound().json(json!({
            "success": false,
            "data": null,
            "pagination": null,
            "message": "Canvas not found",
            "error": "NotFound"
        }))),
        Err(CanvasServiceError::ValidationError(msg)) => {
            Ok(HttpResponse::BadRequest().json(json!({
                "success": false,
                "data": null,
                "pagination": null,
                "message": msg,
                "error": "ValidationError"
            })))
        }
        Err(CanvasServiceError::DatabaseError(msg)) => Ok(HttpResponse::InternalServerError()
            .json(json!({
                "success": false,
                "data": null,
                "pagination": null,
                "message": msg,
                "error": "DatabaseError"
            }))),
    }
}

/// DELETE /canvas/{id} - Delete canvas - REQUIRES AUTHENTICATION
#[delete("/api/v1/canvas/{id}")]
pub async fn delete_canvas(
    _authenticated_user: AuthenticatedUser,
    service: web::Data<Arc<dyn CanvasServiceTrait>>,
    path: web::Path<String>,
) -> Result<impl Responder> {
    let canvas_id = path.into_inner();

    match service.delete_canvas(&canvas_id).await {
        Ok(()) => Ok(HttpResponse::Ok().json(json!({
            "success": true,
            "data": null,
            "pagination": null,
            "message": "Canvas deleted successfully",
            "error": null
        }))),
        Err(CanvasServiceError::NotFound) => Ok(HttpResponse::NotFound().json(json!({
            "success": false,
            "data": null,
            "pagination": null,
            "message": "Canvas not found",
            "error": "NotFound"
        }))),
        Err(CanvasServiceError::ValidationError(msg)) => {
            Ok(HttpResponse::BadRequest().json(json!({
                "success": false,
                "data": null,
                "pagination": null,
                "message": msg,
                "error": "ValidationError"
            })))
        }
        Err(CanvasServiceError::DatabaseError(msg)) => Ok(HttpResponse::InternalServerError()
            .json(json!({
                "success": false,
                "data": null,
                "pagination": null,
                "message": msg,
                "error": "DatabaseError"
            }))),
    }
}

/// GET /canvas/{canvasId}/graph-data - Get graph data for a canvas - REQUIRES AUTHENTICATION
#[get("/api/v1/canvas/{canvas_id}/graph-data")]
pub async fn get_canvas_graph_data(
    _authenticated_user: AuthenticatedUser,
    service: web::Data<Arc<dyn CanvasServiceTrait>>,
    path: web::Path<String>,
) -> Result<impl Responder> {
    let canvas_id = path.into_inner();

    match service.get_graph_data(&canvas_id).await {
        Ok(graph_data) => Ok(HttpResponse::Ok().json(json!({
            "success": true,
            "data": graph_data,
            "pagination": null,
            "message": null,
            "error": null
        }))),
        Err(CanvasServiceError::NotFound) => Ok(HttpResponse::NotFound().json(json!({
            "success": false,
            "data": null,
            "pagination": null,
            "message": "Canvas not found",
            "error": "NotFound"
        }))),
        Err(CanvasServiceError::ValidationError(msg)) => {
            Ok(HttpResponse::BadRequest().json(json!({
                "success": false,
                "data": null,
                "pagination": null,
                "message": msg,
                "error": "ValidationError"
            })))
        }
        Err(CanvasServiceError::DatabaseError(msg)) => Ok(HttpResponse::InternalServerError()
            .json(json!({
                "success": false,
                "data": null,
                "pagination": null,
                "message": msg,
                "error": "DatabaseError"
            }))),
    }
}
