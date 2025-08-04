use crate::models::canvas::{Canvas, GetCanvasesRequest, InsertCanvas, UpdateCanvasRequest, GraphNode, GraphEdge};
use crate::models::common::PaginatedResponse;
use async_trait::async_trait;

#[derive(Debug, thiserror::Error)]
pub enum CanvasRepositoryError {
    #[error("Database error: {0}")]
    DatabaseError(String),
    #[error("Canvas not found")]
    #[allow(dead_code)]
    NotFound,
    #[error("Invalid data format: {0}")]
    InvalidData(String),
}

#[async_trait]
pub trait CanvasRepository: Send + Sync {
    async fn create_canvas(
        &self,
        insert_canvas: InsertCanvas,
    ) -> Result<Canvas, CanvasRepositoryError>;

    async fn get_canvas_by_id(&self, id: &str) -> Result<Option<Canvas>, CanvasRepositoryError>;

    async fn get_canvases(
        &self,
        request: GetCanvasesRequest,
    ) -> Result<PaginatedResponse<Canvas>, CanvasRepositoryError>;

    async fn update_canvas(
        &self,
        id: &str,
        updates: UpdateCanvasRequest,
    ) -> Result<Option<Canvas>, CanvasRepositoryError>;

    async fn delete_canvas(&self, id: &str) -> Result<(), CanvasRepositoryError>;

    // New methods for graph data
    async fn get_topics_by_canvas(&self, canvas_id: &str) -> Result<Vec<GraphNode>, CanvasRepositoryError>;
    
    async fn get_relationships_by_canvas(&self, canvas_id: &str) -> Result<Vec<GraphEdge>, CanvasRepositoryError>;
}
