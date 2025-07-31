use crate::models::canvas::{Canvas, CreateCanvasRequest, UpdateCanvasRequest};
use async_trait::async_trait;

#[derive(Debug, thiserror::Error)]
pub enum CanvasServiceError {
    #[error("Database access error: {0}")]
    DatabaseError(String),
    #[error("Validation error: {0}")]
    ValidationError(String),
    #[error("Canvas not found")]
    NotFound,
}

#[async_trait]
pub trait CanvasServiceTrait: Send + Sync {
    async fn create_canvas(&self, request: CreateCanvasRequest) -> Result<Canvas, CanvasServiceError>;
    
    async fn get_canvas_by_id(&self, id: &str) -> Result<Canvas, CanvasServiceError>;
    
    async fn get_canvases_by_author(&self, author_id: &str) -> Result<Vec<Canvas>, CanvasServiceError>;
    
    async fn update_canvas(&self, id: &str, updates: UpdateCanvasRequest) -> Result<Canvas, CanvasServiceError>;
    
    async fn delete_canvas(&self, id: &str) -> Result<(), CanvasServiceError>;
} 