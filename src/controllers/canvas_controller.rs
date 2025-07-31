use actix_web::{web, HttpResponse, Responder, Result, get, post, put, delete};
use serde_json::json;
use crate::services::canvas_service_trait::{CanvasServiceTrait, CanvasServiceError};
use crate::models::canvas::{CreateCanvasRequest, UpdateCanvasRequest};
use std::sync::Arc;
use serde::Deserialize;

/// POST /canvas - Create a new canvas
#[post("/canvas")]
pub async fn create_canvas(
    service: web::Data<Arc<dyn CanvasServiceTrait>>,
    req: web::Json<CreateCanvasRequest>,
) -> Result<impl Responder> {
    match service.create_canvas(req.into_inner()).await {
        Ok(canvas) => Ok(HttpResponse::Created().json(json!({
            "success": true,
            "data": canvas,
            "message": "Canvas created successfully"
        }))),
        Err(CanvasServiceError::ValidationError(msg)) => {
            Ok(HttpResponse::BadRequest().json(json!({
                "success": false,
                "error": "Validation Error",
                "message": msg
            })))
        }
        Err(CanvasServiceError::DatabaseError(msg)) => {
            Ok(HttpResponse::InternalServerError().json(json!({
                "success": false,
                "error": "Database Error",
                "message": msg
            })))
        }
        Err(CanvasServiceError::NotFound) => {
            Ok(HttpResponse::NotFound().json(json!({
                "success": false,
                "error": "Not Found",
                "message": "Canvas not found"
            })))
        }
    }
}

/// GET /canvas/{id} - Get canvas by ID
#[get("/canvas/{id}")]
pub async fn get_canvas(
    service: web::Data<Arc<dyn CanvasServiceTrait>>,
    path: web::Path<String>,
) -> Result<impl Responder> {
    let canvas_id = path.into_inner();
    
    match service.get_canvas_by_id(&canvas_id).await {
        Ok(canvas) => Ok(HttpResponse::Ok().json(json!({
            "success": true,
            "data": canvas
        }))),
        Err(CanvasServiceError::NotFound) => {
            Ok(HttpResponse::NotFound().json(json!({
                "success": false,
                "error": "Not Found",
                "message": "Canvas not found"
            })))
        }
        Err(CanvasServiceError::ValidationError(msg)) => {
            Ok(HttpResponse::BadRequest().json(json!({
                "success": false,
                "error": "Validation Error",
                "message": msg
            })))
        }
        Err(CanvasServiceError::DatabaseError(msg)) => {
            Ok(HttpResponse::InternalServerError().json(json!({
                "success": false,
                "error": "Database Error",
                "message": msg
            })))
        }
    }
}

/// GET /canvas/author/{author_id} - Get all canvases by author
#[get("/canvas/author/{author_id}")]
pub async fn get_canvases_by_author(
    service: web::Data<Arc<dyn CanvasServiceTrait>>,
    path: web::Path<String>,
) -> Result<impl Responder> {
    let author_id = path.into_inner();
    
    match service.get_canvases_by_author(&author_id).await {
        Ok(canvases) => Ok(HttpResponse::Ok().json(json!({
            "success": true,
            "data": canvases,
            "count": canvases.len()
        }))),
        Err(CanvasServiceError::ValidationError(msg)) => {
            Ok(HttpResponse::BadRequest().json(json!({
                "success": false,
                "error": "Validation Error",
                "message": msg
            })))
        }
        Err(CanvasServiceError::DatabaseError(msg)) => {
            Ok(HttpResponse::InternalServerError().json(json!({
                "success": false,
                "error": "Database Error",
                "message": msg
            })))
        }
        Err(CanvasServiceError::NotFound) => {
            Ok(HttpResponse::Ok().json(json!({
                "success": true,
                "data": [],
                "count": 0,
                "message": "No canvases found for this author"
            })))
        }
    }
}

/// PUT /canvas/{id} - Update canvas
#[put("/canvas/{id}")]
pub async fn update_canvas(
    service: web::Data<Arc<dyn CanvasServiceTrait>>,
    path: web::Path<String>,
    req: web::Json<UpdateCanvasRequest>,
) -> Result<impl Responder> {
    let canvas_id = path.into_inner();
    
    match service.update_canvas(&canvas_id, req.into_inner()).await {
        Ok(canvas) => Ok(HttpResponse::Ok().json(json!({
            "success": true,
            "data": canvas,
            "message": "Canvas updated successfully"
        }))),
        Err(CanvasServiceError::NotFound) => {
            Ok(HttpResponse::NotFound().json(json!({
                "success": false,
                "error": "Not Found",
                "message": "Canvas not found"
            })))
        }
        Err(CanvasServiceError::ValidationError(msg)) => {
            Ok(HttpResponse::BadRequest().json(json!({
                "success": false,
                "error": "Validation Error",
                "message": msg
            })))
        }
        Err(CanvasServiceError::DatabaseError(msg)) => {
            Ok(HttpResponse::InternalServerError().json(json!({
                "success": false,
                "error": "Database Error",
                "message": msg
            })))
        }
    }
}

/// DELETE /canvas/{id} - Delete canvas
#[delete("/canvas/{id}")]
pub async fn delete_canvas(
    service: web::Data<Arc<dyn CanvasServiceTrait>>,
    path: web::Path<String>,
) -> Result<impl Responder> {
    let canvas_id = path.into_inner();
    
    match service.delete_canvas(&canvas_id).await {
        Ok(()) => Ok(HttpResponse::Ok().json(json!({
            "success": true,
            "message": "Canvas deleted successfully"
        }))),
        Err(CanvasServiceError::NotFound) => {
            Ok(HttpResponse::NotFound().json(json!({
                "success": false,
                "error": "Not Found",
                "message": "Canvas not found"
            })))
        }
        Err(CanvasServiceError::ValidationError(msg)) => {
            Ok(HttpResponse::BadRequest().json(json!({
                "success": false,
                "error": "Validation Error",
                "message": msg
            })))
        }
        Err(CanvasServiceError::DatabaseError(msg)) => {
            Ok(HttpResponse::InternalServerError().json(json!({
                "success": false,
                "error": "Database Error",
                "message": msg
            })))
        }
    }
}

/// GET /canvas - Get all canvases (list view)
#[get("/canvas")]
pub async fn get_canvas_list(
    service: web::Data<Arc<dyn CanvasServiceTrait>>,
    query: web::Query<ListCanvasQuery>,
) -> Result<impl Responder> {
    // If author_id is provided, filter by author, otherwise get all canvases
    if let Some(author_id) = &query.author_id {
        match service.get_canvases_by_author(author_id).await {
            Ok(canvases) => Ok(HttpResponse::Ok().json(json!({
                "success": true,
                "data": canvases,
                "count": canvases.len(),
                "filter": {
                    "author_id": author_id
                }
            }))),
            Err(CanvasServiceError::ValidationError(msg)) => {
                Ok(HttpResponse::BadRequest().json(json!({
                    "success": false,
                    "error": "Validation Error",
                    "message": msg
                })))
            }
            Err(CanvasServiceError::DatabaseError(msg)) => {
                Ok(HttpResponse::InternalServerError().json(json!({
                    "success": false,
                    "error": "Database Error",
                    "message": msg
                })))
            }
            Err(CanvasServiceError::NotFound) => {
                Ok(HttpResponse::Ok().json(json!({
                    "success": true,
                    "data": [],
                    "count": 0,
                    "message": "No canvases found for this author"
                })))
            }
        }
    } else {
        // TODO: Implement get_all_canvases in service when needed
        // For now, return a helpful message
        Ok(HttpResponse::Ok().json(json!({
            "success": true,
            "message": "To get canvases, please provide an author_id query parameter: GET /canvas?author_id=your_id",
            "example": "/canvas?author_id=user123",
            "alternative_endpoints": {
                "get_specific_canvas": "GET /canvas/{id}",
                "get_by_author": "GET /canvas/author/{author_id}"
            }
        })))
    }
}

#[derive(Deserialize)]
pub struct ListCanvasQuery {
    pub author_id: Option<String>,
}