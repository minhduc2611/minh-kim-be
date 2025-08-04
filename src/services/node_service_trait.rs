use crate::models::node::{CreateNodeRequest, GetNodesRequest, UpdateNodeRequest};
use crate::models::canvas::GraphNode;
use crate::models::common::PaginatedResponse;
use async_trait::async_trait;

#[derive(Debug, thiserror::Error)]
pub enum NodeServiceError {
    #[error("Database access error: {0}")]
    DatabaseError(String),
    #[error("Validation error: {0}")]
    ValidationError(String),
    #[error("Node not found")]
    NotFound,
}

#[async_trait]
pub trait NodeServiceTrait: Send + Sync {
    async fn create_node(
        &self,
        request: CreateNodeRequest,
    ) -> Result<GraphNode, NodeServiceError>;

    async fn get_node_by_id(&self, id: &str) -> Result<GraphNode, NodeServiceError>;

    async fn get_nodes(
        &self,
        request: GetNodesRequest,
    ) -> Result<PaginatedResponse<GraphNode>, NodeServiceError>;

    async fn get_nodes_by_canvas(&self, canvas_id: &str) -> Result<Vec<GraphNode>, NodeServiceError>;

    async fn update_node(
        &self,
        id: &str,
        updates: UpdateNodeRequest,
    ) -> Result<GraphNode, NodeServiceError>;

    async fn delete_node(&self, id: &str) -> Result<(), NodeServiceError>;

    async fn delete_nodes_by_canvas(&self, canvas_id: &str) -> Result<(), NodeServiceError>;
} 