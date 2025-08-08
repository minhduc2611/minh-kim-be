use crate::middleware::AuthenticatedUser;
use crate::models::node::{CreateNodeRequest, GetNodesRequest, UpdateNodeRequest};
use crate::models::common::ListNodeQuery;
use crate::services::node_service_trait::{NodeServiceError, NodeServiceTrait};
use actix_web::{delete, get, post, put, web, HttpResponse, Responder, Result};

use serde_json::json;
use std::sync::Arc;

/// GET /nodes - Get all nodes for a canvas (list view) - REQUIRES AUTHENTICATION
#[get("/api/v1/nodes")]
pub async fn get_node_list(
    _authenticated_user: AuthenticatedUser,
    service: web::Data<Arc<dyn NodeServiceTrait>>,
    query: web::Query<ListNodeQuery>,
) -> Result<impl Responder> {
    let request = GetNodesRequest {
        canvas_id: query.canvas_id.clone(),
        limit: query.limit,
        offset: query.offset,
    };

    match service.get_nodes(request).await {
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
        Err(NodeServiceError::ValidationError(msg)) => {
            Ok(HttpResponse::BadRequest().json(json!({
                "success": false,
                "data": null,
                "pagination": null,
                "message": msg,
                "error": "ValidationError"
            })))
        }
        Err(NodeServiceError::DatabaseError(msg)) => Ok(HttpResponse::InternalServerError()
            .json(json!({
                "success": false,
                "data": null,
                "pagination": null,
                "message": msg,
                "error": "DatabaseError"
            }))),
        Err(NodeServiceError::NotFound) => Ok(HttpResponse::Ok().json(json!({
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
            "message": "No nodes found for this canvas",
            "error": null
        }))),
        Err(NodeServiceError::TopicAlreadyExists) => Ok(HttpResponse::Conflict().json(json!({
            "success": false,
            "data": null,
            "pagination": null,
            "message": "Topic already exists in this canvas",
            "error": "TopicAlreadyExists"
        }))),
        Err(NodeServiceError::CanvasNotFound) => Ok(HttpResponse::NotFound().json(json!({
            "success": false,
            "data": null,
            "pagination": null,
            "message": "Canvas not found",
            "error": "CanvasNotFound"
        }))),
    }
}

/// POST /nodes - Create a new node - REQUIRES AUTHENTICATION
#[post("/api/v1/nodes")]
pub async fn create_node(
    _authenticated_user: AuthenticatedUser,
    service: web::Data<Arc<dyn NodeServiceTrait>>,
    req: web::Json<CreateNodeRequest>,
) -> Result<impl Responder> {
    match service.create_node(req.into_inner()).await {
        Ok(node) => Ok(HttpResponse::Created().json(json!({
            "success": true,
            "data": node,
            "pagination": null,
            "message": "Node created successfully",
            "error": null
        }))),
        Err(NodeServiceError::ValidationError(msg)) => {
            Ok(HttpResponse::BadRequest().json(json!({
                "success": false,
                "data": null,
                "pagination": null,
                "message": msg,
                "error": "ValidationError"
            })))
        }
        Err(NodeServiceError::DatabaseError(msg)) => Ok(HttpResponse::InternalServerError()
            .json(json!({
                "success": false,
                "data": null,
                "pagination": null,
                "message": msg,
                "error": "DatabaseError"
            }))),
        Err(NodeServiceError::NotFound) => Ok(HttpResponse::NotFound().json(json!({
            "success": false,
            "data": null,
            "pagination": null,
            "message": "Node not found",
            "error": "NotFound"
        }))),
        Err(NodeServiceError::TopicAlreadyExists) => Ok(HttpResponse::Conflict().json(json!({
            "success": false,
            "data": null,
            "pagination": null,
            "message": "Topic already exists in this canvas",
            "error": "TopicAlreadyExists"
        }))),
        Err(NodeServiceError::CanvasNotFound) => Ok(HttpResponse::NotFound().json(json!({
            "success": false,
            "data": null,
            "pagination": null,
            "message": "Canvas not found",
            "error": "CanvasNotFound"
        }))),
        Err(NodeServiceError::TopicAlreadyExists) => Ok(HttpResponse::Conflict().json(json!({
            "success": false,
            "data": null,
            "pagination": null,
            "message": "Topic already exists in this canvas",
            "error": "TopicAlreadyExists"
        }))),
        Err(NodeServiceError::CanvasNotFound) => Ok(HttpResponse::NotFound().json(json!({
            "success": false,
            "data": null,
            "pagination": null,
            "message": "Canvas not found",
            "error": "CanvasNotFound"
        }))),
        Err(NodeServiceError::TopicAlreadyExists) => Ok(HttpResponse::Conflict().json(json!({
            "success": false,
            "data": null,
            "pagination": null,
            "message": "Topic already exists in this canvas",
            "error": "TopicAlreadyExists"
        }))),
        Err(NodeServiceError::CanvasNotFound) => Ok(HttpResponse::NotFound().json(json!({
            "success": false,
            "data": null,
            "pagination": null,
            "message": "Canvas not found",
            "error": "CanvasNotFound"
        }))),
        Err(NodeServiceError::TopicAlreadyExists) => Ok(HttpResponse::Conflict().json(json!({
            "success": false,
            "data": null,
            "pagination": null,
            "message": "Topic already exists in this canvas",
            "error": "TopicAlreadyExists"
        }))),
        Err(NodeServiceError::CanvasNotFound) => Ok(HttpResponse::NotFound().json(json!({
            "success": false,
            "data": null,
            "pagination": null,
            "message": "Canvas not found",
            "error": "CanvasNotFound"
        }))),
        Err(NodeServiceError::TopicAlreadyExists) => Ok(HttpResponse::Conflict().json(json!({
            "success": false,
            "data": null,
            "pagination": null,
            "message": "Topic already exists in this canvas",
            "error": "TopicAlreadyExists"
        }))),
        Err(NodeServiceError::CanvasNotFound) => Ok(HttpResponse::NotFound().json(json!({
            "success": false,
            "data": null,
            "pagination": null,
            "message": "Canvas not found",
            "error": "CanvasNotFound"
        }))),
    }
}

/// GET /nodes/{id} - Get node by ID - REQUIRES AUTHENTICATION
#[get("/api/v1/nodes/{id}")]
pub async fn get_node(
    _authenticated_user: AuthenticatedUser,
    service: web::Data<Arc<dyn NodeServiceTrait>>,
    path: web::Path<String>,
) -> Result<impl Responder> {
    let node_id = path.into_inner();

    match service.get_node_by_id(&node_id).await {
        Ok(node) => Ok(HttpResponse::Ok().json(json!({
            "success": true,
            "data": node,
            "pagination": null,
            "message": null,
            "error": null
        }))),
        Err(NodeServiceError::NotFound) => Ok(HttpResponse::NotFound().json(json!({
            "success": false,
            "data": null,
            "pagination": null,
            "message": "Node not found",
            "error": "NotFound"
        }))),
        Err(NodeServiceError::ValidationError(msg)) => {
            Ok(HttpResponse::BadRequest().json(json!({
                "success": false,
                "data": null,
                "pagination": null,
                "message": msg,
                "error": "ValidationError"
            })))
        }
        Err(NodeServiceError::DatabaseError(msg)) => Ok(HttpResponse::InternalServerError()
            .json(json!({
                "success": false,
                "data": null,
                "pagination": null,
                "message": msg,
                "error": "DatabaseError"
            }))),
        Err(NodeServiceError::TopicAlreadyExists) => Ok(HttpResponse::Conflict().json(json!({
            "success": false,
            "data": null,
            "pagination": null,
            "message": "Topic already exists in this canvas",
            "error": "TopicAlreadyExists"
        }))),
        Err(NodeServiceError::CanvasNotFound) => Ok(HttpResponse::NotFound().json(json!({
            "success": false,
            "data": null,
            "pagination": null,
            "message": "Canvas not found",
            "error": "CanvasNotFound"
        }))),
    }
}

/// PUT /nodes/{id} - Update node - REQUIRES AUTHENTICATION
#[put("/api/v1/nodes/{id}")]
pub async fn update_node(
    _authenticated_user: AuthenticatedUser,
    service: web::Data<Arc<dyn NodeServiceTrait>>,
    path: web::Path<String>,
    req: web::Json<UpdateNodeRequest>,
) -> Result<impl Responder> {
    let node_id = path.into_inner();

    match service.update_node(&node_id, req.into_inner()).await {
        Ok(node) => Ok(HttpResponse::Ok().json(json!({
            "success": true,
            "data": node,
            "pagination": null,
            "message": "Node updated successfully",
            "error": null
        }))),
        Err(NodeServiceError::NotFound) => Ok(HttpResponse::NotFound().json(json!({
            "success": false,
            "data": null,
            "pagination": null,
            "message": "Node not found",
            "error": "NotFound"
        }))),
        Err(NodeServiceError::ValidationError(msg)) => {
            Ok(HttpResponse::BadRequest().json(json!({
                "success": false,
                "data": null,
                "pagination": null,
                "message": msg,
                "error": "ValidationError"
            })))
        }
        Err(NodeServiceError::DatabaseError(msg)) => Ok(HttpResponse::InternalServerError()
            .json(json!({
                "success": false,
                "data": null,
                "pagination": null,
                "message": msg,
                "error": "DatabaseError"
            }))),
        Err(NodeServiceError::TopicAlreadyExists) => Ok(HttpResponse::Conflict().json(json!({
            "success": false,
            "data": null,
            "pagination": null,
            "message": "Topic already exists in this canvas",
            "error": "TopicAlreadyExists"
        }))),
        Err(NodeServiceError::CanvasNotFound) => Ok(HttpResponse::NotFound().json(json!({
            "success": false,
            "data": null,
            "pagination": null,
            "message": "Canvas not found",
            "error": "CanvasNotFound"
        }))),
    }
}

/// DELETE /nodes/{id} - Delete node - REQUIRES AUTHENTICATION
#[delete("/api/v1/nodes/{id}")]
pub async fn delete_node(
    _authenticated_user: AuthenticatedUser,
    service: web::Data<Arc<dyn NodeServiceTrait>>,
    path: web::Path<String>,
) -> Result<impl Responder> {
    let node_id = path.into_inner();

    match service.delete_node(&node_id).await {
        Ok(_) => Ok(HttpResponse::Ok().json(json!({
            "success": true,
            "data": null,
            "pagination": null,
            "message": "Node deleted successfully",
            "error": null
        }))),
        Err(NodeServiceError::NotFound) => Ok(HttpResponse::NotFound().json(json!({
            "success": false,
            "data": null,
            "pagination": null,
            "message": "Node not found",
            "error": "NotFound"
        }))),
        Err(NodeServiceError::ValidationError(msg)) => {
            Ok(HttpResponse::BadRequest().json(json!({
                "success": false,
                "data": null,
                "pagination": null,
                "message": msg,
                "error": "ValidationError"
            })))
        }
        Err(NodeServiceError::DatabaseError(msg)) => Ok(HttpResponse::InternalServerError()
            .json(json!({
                "success": false,
                "data": null,
                "pagination": null,
                "message": msg,
                "error": "DatabaseError"
            }))),
        Err(NodeServiceError::TopicAlreadyExists) => Ok(HttpResponse::Conflict().json(json!({
            "success": false,
            "data": null,
            "pagination": null,
            "message": "Topic already exists in this canvas",
            "error": "TopicAlreadyExists"
        }))),
        Err(NodeServiceError::CanvasNotFound) => Ok(HttpResponse::NotFound().json(json!({
            "success": false,
            "data": null,
            "pagination": null,
            "message": "Canvas not found",
            "error": "CanvasNotFound"
        }))),
    }
}

/// GET /canvas/{canvas_id}/nodes - Get all nodes for a specific canvas - REQUIRES AUTHENTICATION
#[get("/api/v1/canvas/{canvas_id}/nodes")]
pub async fn get_nodes_by_canvas(
    _authenticated_user: AuthenticatedUser,
    service: web::Data<Arc<dyn NodeServiceTrait>>,
    path: web::Path<String>,
) -> Result<impl Responder> {
    let canvas_id = path.into_inner();

    match service.get_nodes_by_canvas(&canvas_id).await {
        Ok(nodes) => Ok(HttpResponse::Ok().json(json!({
            "success": true,
            "data": nodes,
            "pagination": null,
            "message": null,
            "error": null
        }))),
        Err(NodeServiceError::NotFound) => Ok(HttpResponse::NotFound().json(json!({
            "success": false,
            "data": null,
            "pagination": null,
            "message": "Canvas not found",
            "error": "NotFound"
        }))),
        Err(NodeServiceError::ValidationError(msg)) => {
            Ok(HttpResponse::BadRequest().json(json!({
                "success": false,
                "data": null,
                "pagination": null,
                "message": msg,
                "error": "ValidationError"
            })))
        }
        Err(NodeServiceError::DatabaseError(msg)) => Ok(HttpResponse::InternalServerError()
            .json(json!({
                "success": false,
                "data": null,
                "pagination": null,
                "message": msg,
                "error": "DatabaseError"
            }))),
        Err(NodeServiceError::TopicAlreadyExists) => Ok(HttpResponse::Conflict().json(json!({
            "success": false,
            "data": null,
            "pagination": null,
            "message": "Topic already exists in this canvas",
            "error": "TopicAlreadyExists"
        }))),
        Err(NodeServiceError::CanvasNotFound) => Ok(HttpResponse::NotFound().json(json!({
            "success": false,
            "data": null,
            "pagination": null,
            "message": "Canvas not found",
            "error": "CanvasNotFound"
        }))),
    }
}

/// DELETE /canvas/{canvas_id}/nodes - Delete all nodes for a specific canvas - REQUIRES AUTHENTICATION
#[delete("/api/v1/canvas/{canvas_id}/nodes")]
pub async fn delete_nodes_by_canvas(
    _authenticated_user: AuthenticatedUser,
    service: web::Data<Arc<dyn NodeServiceTrait>>,
    path: web::Path<String>,
) -> Result<impl Responder> {
    let canvas_id = path.into_inner();

    match service.delete_nodes_by_canvas(&canvas_id).await {
        Ok(_) => Ok(HttpResponse::Ok().json(json!({
            "success": true,
            "data": null,
            "pagination": null,
            "message": "All nodes for canvas deleted successfully",
            "error": null
        }))),
        Err(NodeServiceError::NotFound) => Ok(HttpResponse::NotFound().json(json!({
            "success": false,
            "data": null,
            "pagination": null,
            "message": "Canvas not found",
            "error": "NotFound"
        }))),
        Err(NodeServiceError::ValidationError(msg)) => {
            Ok(HttpResponse::BadRequest().json(json!({
                "success": false,
                "data": null,
                "pagination": null,
                "message": msg,
                "error": "ValidationError"
            })))
        }
        Err(NodeServiceError::DatabaseError(msg)) => Ok(HttpResponse::InternalServerError()
            .json(json!({
                "success": false,
                "data": null,
                "pagination": null,
                "message": msg,
                "error": "DatabaseError"
            }))),
        Err(NodeServiceError::TopicAlreadyExists) => Ok(HttpResponse::Conflict().json(json!({
            "success": false,
            "data": null,
            "pagination": null,
            "message": "Topic already exists in this canvas",
            "error": "TopicAlreadyExists"
        }))),
        Err(NodeServiceError::CanvasNotFound) => Ok(HttpResponse::NotFound().json(json!({
            "success": false,
            "data": null,
            "pagination": null,
            "message": "Canvas not found",
            "error": "CanvasNotFound"
        }))),
    }
} 