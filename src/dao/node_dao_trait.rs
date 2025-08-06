use crate::models::node::{GetNodesRequest, InsertNode, UpdateNodeRequest, InsertRelationship, Relationship};
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
    async fn create_topic(
        &self,
        insert_node: InsertNode,
    ) -> Result<GraphNode, NodeRepositoryError>;

    async fn get_topic_by_id(&self, id: &str) -> Result<Option<GraphNode>, NodeRepositoryError>;

    async fn get_topics(
        &self,
        request: GetNodesRequest,
    ) -> Result<PaginatedResponse<GraphNode>, NodeRepositoryError>;

    async fn get_topics_by_canvas(&self, canvas_id: &str) -> Result<Vec<GraphNode>, NodeRepositoryError>;

    async fn update_topic(
        &self,
        id: &str,
        updates: UpdateNodeRequest,
    ) -> Result<Option<GraphNode>, NodeRepositoryError>;

    async fn delete_topic(&self, id: &str) -> Result<(), NodeRepositoryError>;

    async fn delete_topics_by_canvas(&self, canvas_id: &str) -> Result<(), NodeRepositoryError>;

    async fn get_topic_by_name_and_canvas(
        &self,
        name: &str,
        canvas_id: &str,
    ) -> Result<Option<GraphNode>, NodeRepositoryError>;

    async fn get_topic_path(
        &self,
        topic_id: &str,
        canvas_id: &str,
    ) -> Result<Vec<String>, NodeRepositoryError>;

    async fn get_existing_siblings(
        &self,
        topic_id: &str,
        canvas_id: &str,
    ) -> Result<Vec<String>, NodeRepositoryError>;

    async fn get_topic_children(
        &self,
        topic_id: &str,
        canvas_id: &str,
    ) -> Result<Vec<String>, NodeRepositoryError>;

    async fn relationship_exists(
        &self,
        source_id: &str,
        target_id: &str,
    ) -> Result<bool, NodeRepositoryError>;

    async fn create_relationship(
        &self,
        insert_relationship: InsertRelationship,
    ) -> Result<Relationship, NodeRepositoryError>;
} 