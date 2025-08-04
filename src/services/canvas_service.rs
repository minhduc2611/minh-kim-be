use crate::dao::canvas_dao_trait::{CanvasRepository, CanvasRepositoryError};
use crate::models::canvas::{
    Canvas, CreateCanvasRequest, GetCanvasesRequest, InsertCanvas, UpdateCanvasRequest, GraphData,
};
use crate::models::common::PaginatedResponse;
use crate::services::canvas_service_trait::{CanvasServiceError, CanvasServiceTrait};
use async_trait::async_trait;
use std::sync::Arc;

pub struct CanvasService {
    repository: Arc<dyn CanvasRepository>,
}

impl CanvasService {
    pub fn new(repository: Arc<dyn CanvasRepository>) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl CanvasServiceTrait for CanvasService {
    async fn create_canvas(
        &self,
        request: CreateCanvasRequest,
    ) -> Result<Canvas, CanvasServiceError> {
        // Validate request
        Self::validate_create_request(&request)?;

        // Convert to insert model
        let insert_canvas = InsertCanvas::from(request);

        // Create via repository
        self.repository
            .create_canvas(insert_canvas)
            .await
            .map_err(|e| match e {
                CanvasRepositoryError::DatabaseError(msg) => CanvasServiceError::DatabaseError(msg),
                CanvasRepositoryError::NotFound => CanvasServiceError::NotFound,
                CanvasRepositoryError::InvalidData(msg) => CanvasServiceError::DatabaseError(msg),
            })
    }

    async fn get_canvas_by_id(&self, id: &str) -> Result<Canvas, CanvasServiceError> {
        // Validate ID format
        Self::validate_id(id)?;

        match self.repository.get_canvas_by_id(id).await {
            Ok(Some(canvas)) => Ok(canvas),
            Ok(None) => Err(CanvasServiceError::NotFound),
            Err(e) => Err(Self::map_repository_error(e)),
        }
    }

    async fn get_canvases(
        &self,
        request: GetCanvasesRequest,
    ) -> Result<PaginatedResponse<Canvas>, CanvasServiceError> {
        // Validate request
        Self::validate_get_canvases_request(&request)?;

        self.repository
            .get_canvases(request)
            .await
            .map_err(Self::map_repository_error)
    }

    async fn update_canvas(
        &self,
        id: &str,
        updates: UpdateCanvasRequest,
    ) -> Result<Canvas, CanvasServiceError> {
        // Validate inputs
        Self::validate_id(id)?;
        Self::validate_update_request(&updates)?;

        match self.repository.update_canvas(id, updates).await {
            Ok(Some(canvas)) => Ok(canvas),
            Ok(None) => Err(CanvasServiceError::NotFound),
            Err(e) => Err(Self::map_repository_error(e)),
        }
    }

    /// Deletes a canvas by its ID
    ///
    /// This method validates the canvas ID format and then delegates to the repository
    /// for deletion. It handles proper error mapping from repository errors to service errors.
    ///
    /// # Arguments
    /// * `id` - The unique identifier of the canvas to delete
    ///
    /// # Returns
    /// * `Ok(())` - Canvas was successfully deleted
    /// * `Err(CanvasServiceError::ValidationError)` - Invalid ID format
    /// * `Err(CanvasServiceError::NotFound)` - Canvas with given ID does not exist
    /// * `Err(CanvasServiceError::DatabaseError)` - Database operation failed
    ///
    /// # Performance Note
    /// This implementation avoids the overhead of checking canvas existence before deletion.
    /// The repository layer handles existence checking efficiently as part of the delete operation.
    async fn delete_canvas(&self, id: &str) -> Result<(), CanvasServiceError> {
        // Validate ID format (empty, whitespace-only IDs are rejected)
        Self::validate_id(id)?;

        // Delete via repository - the repository will return NotFound if canvas doesn't exist
        // This approach is more efficient than checking existence first, then deleting
        match self.repository.delete_canvas(id).await {
            Ok(()) => Ok(()),
            Err(CanvasRepositoryError::NotFound) => Err(CanvasServiceError::NotFound),
            Err(e) => Err(Self::map_repository_error(e)),
        }
    }

    async fn get_graph_data(&self, canvas_id: &str) -> Result<GraphData, CanvasServiceError> {
        // Validate canvas ID format
        Self::validate_id(canvas_id)?;

        // Get topics and relationships from repository
        let topics = self.repository
            .get_topics_by_canvas(canvas_id)
            .await
            .map_err(Self::map_repository_error)?;
        println!("topics: Done, length: {}", topics.len());
        let relationships = self.repository
            .get_relationships_by_canvas(canvas_id)
            .await
            .map_err(Self::map_repository_error)?;
        println!("relationships: Done, length: {}", relationships.len());
        Ok(GraphData {
            nodes: topics,
            edges: relationships,
        })
    }
}

impl CanvasService {
    // Helper method to map repository errors to service errors
    fn map_repository_error(error: CanvasRepositoryError) -> CanvasServiceError {
        match error {
            CanvasRepositoryError::DatabaseError(msg) => CanvasServiceError::DatabaseError(msg),
            CanvasRepositoryError::NotFound => CanvasServiceError::NotFound,
            CanvasRepositoryError::InvalidData(msg) => CanvasServiceError::DatabaseError(msg),
        }
    }

    // Validation helpers
    fn validate_create_request(request: &CreateCanvasRequest) -> Result<(), CanvasServiceError> {
        if request.name.trim().is_empty() {
            return Err(CanvasServiceError::ValidationError(
                "Canvas name cannot be empty".to_string(),
            ));
        }

        if request.name.len() > 100 {
            return Err(CanvasServiceError::ValidationError(
                "Canvas name cannot exceed 100 characters".to_string(),
            ));
        }

        if request.author_id.trim().is_empty() {
            return Err(CanvasServiceError::ValidationError(
                "Author ID cannot be empty".to_string(),
            ));
        }

        Ok(())
    }

    fn validate_update_request(request: &UpdateCanvasRequest) -> Result<(), CanvasServiceError> {
        if let Some(name) = &request.name {
            if name.trim().is_empty() {
                return Err(CanvasServiceError::ValidationError(
                    "Canvas name cannot be empty".to_string(),
                ));
            }

            if name.len() > 100 {
                return Err(CanvasServiceError::ValidationError(
                    "Canvas name cannot exceed 100 characters".to_string(),
                ));
            }
        }

        Ok(())
    }

    fn validate_id(id: &str) -> Result<(), CanvasServiceError> {
        if id.trim().is_empty() {
            return Err(CanvasServiceError::ValidationError(
                "ID cannot be empty".to_string(),
            ));
        }

        Ok(())
    }

    fn validate_get_canvases_request(
        request: &GetCanvasesRequest,
    ) -> Result<(), CanvasServiceError> {
        // Validate author ID
        Self::validate_id(&request.author_id)?;

        // Validate limit
        if let Some(limit) = request.limit {
            if limit <= 0 {
                return Err(CanvasServiceError::ValidationError(
                    "Limit must be greater than 0".to_string(),
                ));
            }
            if limit > 100 {
                return Err(CanvasServiceError::ValidationError(
                    "Limit cannot exceed 100".to_string(),
                ));
            }
        }

        // Validate offset
        if let Some(offset) = request.offset {
            if offset < 0 {
                return Err(CanvasServiceError::ValidationError(
                    "Offset cannot be negative".to_string(),
                ));
            }
        }

        Ok(())
    }
}
