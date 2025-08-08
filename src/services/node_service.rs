use crate::dao::node_dao_trait::{NodeRepository, NodeRepositoryError};
use crate::dao::canvas_dao_trait::{CanvasRepository, CanvasRepositoryError};
use crate::models::node::{CreateNodeRequest, GetNodesRequest, UpdateNodeRequest, InsertNode, InsertRelationship};
use crate::models::canvas::GraphNode;
use crate::models::common::PaginatedResponse;
use crate::services::node_service_trait::{NodeServiceError, NodeServiceTrait};
use async_trait::async_trait;
use std::sync::Arc;

pub struct NodeService {
    repository: Arc<dyn NodeRepository>,
    canvas_repository: Arc<dyn CanvasRepository>,
}

impl NodeService {
    pub fn new(repository: Arc<dyn NodeRepository>, canvas_repository: Arc<dyn CanvasRepository>) -> Self {
        Self { 
            repository,
            canvas_repository,
        }
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
        
        // Check if canvas exists
        let canvas = self.canvas_repository.get_canvas_by_id(&request.canvas_id).await
            .map_err(|e| match e {
                CanvasRepositoryError::DatabaseError(msg) => NodeServiceError::DatabaseError(msg),
                CanvasRepositoryError::InvalidData(msg) => NodeServiceError::ValidationError(msg),
                CanvasRepositoryError::NotFound => NodeServiceError::CanvasNotFound,
            })?;
        
        if canvas.is_none() {
            return Err(NodeServiceError::CanvasNotFound);
        }
        
        // Check if topic already exists
        let existing_topic = self.repository.get_topic_node_by_name_and_canvas(&request.name, &request.canvas_id).await
            .map_err(|e| match e {
                NodeRepositoryError::DatabaseError(msg) => NodeServiceError::DatabaseError(msg),
                NodeRepositoryError::InvalidData(msg) => NodeServiceError::ValidationError(msg),
                NodeRepositoryError::NotFound => NodeServiceError::NotFound,
            })?;
        
        if existing_topic.is_some() {
            return Err(NodeServiceError::TopicAlreadyExists);
        }
        
        // Determine node type based on parent_node_id
        let node_type = if request.parent_node_id.is_some() {
            "generated".to_string()
        } else {
            request.node_type.clone().unwrap_or_else(|| "original".to_string())
        };
        
        // Store values before moving request
        let canvas_id = request.canvas_id.clone();
        let parent_node_id = request.parent_node_id.clone();
        
        // Convert to insert model with updated node type
        let mut insert_node: InsertNode = request.into();
        insert_node.node_type = node_type;
        
        // Create node
        let new_node = self.repository.create_topic_node(insert_node).await.map_err(|e| match e {
            NodeRepositoryError::DatabaseError(msg) => NodeServiceError::DatabaseError(msg),
            NodeRepositoryError::InvalidData(msg) => NodeServiceError::ValidationError(msg),
            NodeRepositoryError::NotFound => NodeServiceError::NotFound,
        })?;
        
        // Create relationship if parent node is specified
        if let Some(parent_node_id) = parent_node_id {
            let relationship = InsertRelationship {
                id: uuid::Uuid::new_v4().to_string(),
                canvas_id: canvas_id,
                source_id: parent_node_id,
                target_id: new_node.id.clone(),
            };
            
            self.repository.create_relationship(relationship).await
                .map_err(|e| match e {
                    NodeRepositoryError::DatabaseError(msg) => NodeServiceError::DatabaseError(msg),
                    NodeRepositoryError::InvalidData(msg) => NodeServiceError::ValidationError(msg),
                    NodeRepositoryError::NotFound => NodeServiceError::NotFound,
                })?;
        }
        
        Ok(new_node)
    }

    async fn get_node_by_id(&self, id: &str) -> Result<GraphNode, NodeServiceError> {
        if id.trim().is_empty() {
            return Err(NodeServiceError::ValidationError("Node ID cannot be empty".to_string()));
        }
        
        let node = self.repository.get_topic_node_by_id(id).await.map_err(|e| match e {
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
        
        self.repository.get_topic_nodes(request).await.map_err(|e| match e {
            NodeRepositoryError::DatabaseError(msg) => NodeServiceError::DatabaseError(msg),
            NodeRepositoryError::InvalidData(msg) => NodeServiceError::ValidationError(msg),
            NodeRepositoryError::NotFound => NodeServiceError::NotFound,
        })
    }

    async fn get_nodes_by_canvas(&self, canvas_id: &str) -> Result<Vec<GraphNode>, NodeServiceError> {
        if canvas_id.trim().is_empty() {
            return Err(NodeServiceError::ValidationError("Canvas ID cannot be empty".to_string()));
        }
        
        self.repository.get_topic_nodes_by_canvas(canvas_id).await.map_err(|e| match e {
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
        
        let node = self.repository.update_topic_node(id, updates).await.map_err(|e| match e {
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
        
        self.repository.delete_topic_node(id).await.map_err(|e| match e {
            NodeRepositoryError::DatabaseError(msg) => NodeServiceError::DatabaseError(msg),
            NodeRepositoryError::InvalidData(msg) => NodeServiceError::ValidationError(msg),
            NodeRepositoryError::NotFound => NodeServiceError::NotFound,
        })
    }

    async fn delete_nodes_by_canvas(&self, canvas_id: &str) -> Result<(), NodeServiceError> {
        if canvas_id.trim().is_empty() {
            return Err(NodeServiceError::ValidationError("Canvas ID cannot be empty".to_string()));
        }
        
        self.repository.delete_topic_nodes_by_canvas(canvas_id).await.map_err(|e| match e {
            NodeRepositoryError::DatabaseError(msg) => NodeServiceError::DatabaseError(msg),
            NodeRepositoryError::InvalidData(msg) => NodeServiceError::ValidationError(msg),
            NodeRepositoryError::NotFound => NodeServiceError::NotFound,
        })
    }
} 