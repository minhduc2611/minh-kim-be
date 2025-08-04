use crate::models::canvas::{Canvas, CreateCanvasRequest, GetCanvasesRequest, UpdateCanvasRequest, GraphData};
use crate::models::common::PaginatedResponse;
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
    async fn create_canvas(
        &self,
        request: CreateCanvasRequest,
    ) -> Result<Canvas, CanvasServiceError>;

    async fn get_canvas_by_id(&self, id: &str) -> Result<Canvas, CanvasServiceError>;

    async fn get_canvases(
        &self,
        request: GetCanvasesRequest,
    ) -> Result<PaginatedResponse<Canvas>, CanvasServiceError>;

    async fn update_canvas(
        &self,
        id: &str,
        updates: UpdateCanvasRequest,
    ) -> Result<Canvas, CanvasServiceError>;

    async fn delete_canvas(&self, id: &str) -> Result<(), CanvasServiceError>;

    // New method for graph data
    async fn get_graph_data(&self, canvas_id: &str) -> Result<GraphData, CanvasServiceError>;
}
