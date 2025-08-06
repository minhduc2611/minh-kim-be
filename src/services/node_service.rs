use crate::dao::node_dao_trait::{NodeRepository, NodeRepositoryError};
use crate::models::node::{CreateNodeRequest, GetNodesRequest, UpdateNodeRequest, InsertNode};
use crate::models::canvas::GraphNode;
use crate::models::common::PaginatedResponse;
use crate::services::node_service_trait::{NodeServiceError, NodeServiceTrait};
use async_trait::async_trait;
use std::sync::Arc;

pub struct NodeService {
    repository: Arc<dyn NodeRepository>,
}

impl NodeService {
    pub fn new(repository: Arc<dyn NodeRepository>) -> Self {
        Self { repository }
    }

    fn validate_create_request(&self, request: &CreateNodeRequest) -> Result<(), NodeServiceError> {
        if request.name.trim().is_empty() {
            return Err(NodeServiceError::ValidationError("Node name cannot be empty".to_string()));
        }
        
        if request.name.len() > 100 {
            return Err(NodeServiceError::ValidationError("Node name cannot exceed 100 characters".to_string()));
        }
        
        if request.canvas_id.trim().is_empty() {
            return Err(NodeServiceError::ValidationError("Canvas ID cannot be empty".to_string()));
        }
        
        if let Some(node_type) = &request.node_type {
            if node_type != "original" && node_type != "generated" {
                return Err(NodeServiceError::ValidationError("Node type must be 'original' or 'generated'".to_string()));
            }
        }
        
        Ok(())
    }

    fn validate_update_request(&self, updates: &UpdateNodeRequest) -> Result<(), NodeServiceError> {
        if let Some(name) = &updates.name {
            if name.trim().is_empty() {
                return Err(NodeServiceError::ValidationError("Node name cannot be empty".to_string()));
            }
            
            if name.len() > 100 {
                return Err(NodeServiceError::ValidationError("Node name cannot exceed 100 characters".to_string()));
            }
        }
        
        if let Some(node_type) = &updates.node_type {
            if node_type != "original" && node_type != "generated" {
                return Err(NodeServiceError::ValidationError("Node type must be 'original' or 'generated'".to_string()));
            }
        }
        
        Ok(())
    }
}

#[async_trait]
impl NodeServiceTrait for NodeService {
    async fn create_node(
        &self,
        request: CreateNodeRequest,
    ) -> Result<GraphNode, NodeServiceError> {
        // Validate request
        self.validate_create_request(&request)?;
        
        // Convert to insert model
        let insert_node: InsertNode = request.into();
        
        // Create node
        self.repository.create_topic(insert_node).await.map_err(|e| match e {
            NodeRepositoryError::DatabaseError(msg) => NodeServiceError::DatabaseError(msg),
            NodeRepositoryError::InvalidData(msg) => NodeServiceError::ValidationError(msg),
            NodeRepositoryError::NotFound => NodeServiceError::NotFound,
        })
    }

    async fn get_node_by_id(&self, id: &str) -> Result<GraphNode, NodeServiceError> {
        if id.trim().is_empty() {
            return Err(NodeServiceError::ValidationError("Node ID cannot be empty".to_string()));
        }
        
        let node = self.repository.get_topic_by_id(id).await.map_err(|e| match e {
            NodeRepositoryError::DatabaseError(msg) => NodeServiceError::DatabaseError(msg),
            NodeRepositoryError::InvalidData(msg) => NodeServiceError::ValidationError(msg),
            NodeRepositoryError::NotFound => NodeServiceError::NotFound,
        })?;
        
        node.ok_or(NodeServiceError::NotFound)
    }

    async fn get_nodes(
        &self,
        request: GetNodesRequest,
    ) -> Result<PaginatedResponse<GraphNode>, NodeServiceError> {
        if request.canvas_id.trim().is_empty() {
            return Err(NodeServiceError::ValidationError("Canvas ID cannot be empty".to_string()));
        }
        
        // Validate pagination parameters
        if let Some(limit) = request.limit {
            if limit <= 0 || limit > 100 {
                return Err(NodeServiceError::ValidationError("Limit must be between 1 and 100".to_string()));
            }
        }
        
        if let Some(offset) = request.offset {
            if offset < 0 {
                return Err(NodeServiceError::ValidationError("Offset cannot be negative".to_string()));
            }
        }
        
        self.repository.get_topics(request).await.map_err(|e| match e {
            NodeRepositoryError::DatabaseError(msg) => NodeServiceError::DatabaseError(msg),
            NodeRepositoryError::InvalidData(msg) => NodeServiceError::ValidationError(msg),
            NodeRepositoryError::NotFound => NodeServiceError::NotFound,
        })
    }

    async fn get_nodes_by_canvas(&self, canvas_id: &str) -> Result<Vec<GraphNode>, NodeServiceError> {
        if canvas_id.trim().is_empty() {
            return Err(NodeServiceError::ValidationError("Canvas ID cannot be empty".to_string()));
        }
        
        self.repository.get_topics_by_canvas(canvas_id).await.map_err(|e| match e {
            NodeRepositoryError::DatabaseError(msg) => NodeServiceError::DatabaseError(msg),
            NodeRepositoryError::InvalidData(msg) => NodeServiceError::ValidationError(msg),
            NodeRepositoryError::NotFound => NodeServiceError::NotFound,
        })
    }

    async fn update_node(
        &self,
        id: &str,
        updates: UpdateNodeRequest,
    ) -> Result<GraphNode, NodeServiceError> {
        if id.trim().is_empty() {
            return Err(NodeServiceError::ValidationError("Node ID cannot be empty".to_string()));
        }
        
        // Validate updates
        self.validate_update_request(&updates)?;
        
        let node = self.repository.update_topic(id, updates).await.map_err(|e| match e {
            NodeRepositoryError::DatabaseError(msg) => NodeServiceError::DatabaseError(msg),
            NodeRepositoryError::InvalidData(msg) => NodeServiceError::ValidationError(msg),
            NodeRepositoryError::NotFound => NodeServiceError::NotFound,
        })?;
        
        node.ok_or(NodeServiceError::NotFound)
    }

    async fn delete_node(&self, id: &str) -> Result<(), NodeServiceError> {
        if id.trim().is_empty() {
            return Err(NodeServiceError::ValidationError("Node ID cannot be empty".to_string()));
        }
        
        self.repository.delete_topic(id).await.map_err(|e| match e {
            NodeRepositoryError::DatabaseError(msg) => NodeServiceError::DatabaseError(msg),
            NodeRepositoryError::InvalidData(msg) => NodeServiceError::ValidationError(msg),
            NodeRepositoryError::NotFound => NodeServiceError::NotFound,
        })
    }

    async fn delete_nodes_by_canvas(&self, canvas_id: &str) -> Result<(), NodeServiceError> {
        if canvas_id.trim().is_empty() {
            return Err(NodeServiceError::ValidationError("Canvas ID cannot be empty".to_string()));
        }
        
        self.repository.delete_topics_by_canvas(canvas_id).await.map_err(|e| match e {
            NodeRepositoryError::DatabaseError(msg) => NodeServiceError::DatabaseError(msg),
            NodeRepositoryError::InvalidData(msg) => NodeServiceError::ValidationError(msg),
            NodeRepositoryError::NotFound => NodeServiceError::NotFound,
        })
    }
} 