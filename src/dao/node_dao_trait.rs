use crate::models::node::{GetNodesRequest, InsertNode, UpdateNodeRequest};
use crate::models::canvas::GraphNode;
use crate::models::common::PaginatedResponse;
use async_trait::async_trait;

#[derive(Debug, thiserror::Error)]
pub enum NodeRepositoryError {
    #[error("Database error: {0}")]
    DatabaseError(String),
    #[error("Node not found")]
    #[allow(dead_code)]
    NotFound,
    #[error("Invalid data format: {0}")]
    InvalidData(String),
}

#[async_trait]
pub trait NodeRepository: Send + Sync {
    async fn create_node(
        &self,
        insert_node: InsertNode,
    ) -> Result<GraphNode, NodeRepositoryError>;

    async fn get_node_by_id(&self, id: &str) -> Result<Option<GraphNode>, NodeRepositoryError>;

    async fn get_nodes(
        &self,
        request: GetNodesRequest,
    ) -> Result<PaginatedResponse<GraphNode>, NodeRepositoryError>;

    async fn get_nodes_by_canvas(&self, canvas_id: &str) -> Result<Vec<GraphNode>, NodeRepositoryError>;

    async fn update_node(
        &self,
        id: &str,
        updates: UpdateNodeRequest,
    ) -> Result<Option<GraphNode>, NodeRepositoryError>;

    async fn delete_node(&self, id: &str) -> Result<(), NodeRepositoryError>;

    async fn delete_nodes_by_canvas(&self, canvas_id: &str) -> Result<(), NodeRepositoryError>;
} 