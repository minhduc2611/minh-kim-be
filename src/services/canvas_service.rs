use crate::dao::canvas_dao_trait::{CanvasRepository, CanvasRepositoryError};
use crate::models::canvas::{Canvas, CreateCanvasRequest, InsertCanvas, UpdateCanvasRequest};
use crate::services::canvas_service_trait::{CanvasServiceTrait, CanvasServiceError};
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
    async fn create_canvas(&self, request: CreateCanvasRequest) -> Result<Canvas, CanvasServiceError> {
        // Validate request
        Self::validate_create_request(&request)?;

        // Convert to insert model
        let insert_canvas = InsertCanvas::from(request);

        // Create via repository
        self.repository.create_canvas(insert_canvas)
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

    async fn get_canvases_by_author(&self, author_id: &str) -> Result<Vec<Canvas>, CanvasServiceError> {
        // Validate author ID
        Self::validate_id(author_id)?;

        self.repository.get_canvases_by_author(author_id)
            .await
            .map_err(Self::map_repository_error)
    }

    async fn update_canvas(&self, id: &str, updates: UpdateCanvasRequest) -> Result<Canvas, CanvasServiceError> {
        // Validate inputs
        Self::validate_id(id)?;
        Self::validate_update_request(&updates)?;

        match self.repository.update_canvas(id, updates).await {
            Ok(Some(canvas)) => Ok(canvas),
            Ok(None) => Err(CanvasServiceError::NotFound),
            Err(e) => Err(Self::map_repository_error(e)),
        }
    }

    async fn delete_canvas(&self, id: &str) -> Result<(), CanvasServiceError> {
        // Validate ID
        Self::validate_id(id)?;

        // Check if canvas exists
        self.get_canvas_by_id(id).await?;

        // Delete via repository
        self.repository.delete_canvas(id)
            .await
            .map_err(Self::map_repository_error)
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
} 