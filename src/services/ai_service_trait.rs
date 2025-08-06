use crate::services::ai_service::{GenerateKeywordsRequest, GenerateKeywordsResponse};
use async_trait::async_trait;

#[derive(Debug, thiserror::Error)]
pub enum AIServiceError {
    #[error("Generation failed: {0}")]
    GenerationFailed(String),
    #[error("Canvas not found: {0}")]
    CanvasNotFound(String),
    #[error("Topic not found: {0}")]
    TopicNotFound(String),
    #[error("Database error: {0}")]
    DatabaseError(String),
    #[error("AI service error: {0}")]
    AIServiceError(String),
    #[error("Invalid response format: {0}")]
    InvalidResponseFormat(String),
}

#[async_trait]
pub trait AIServiceTrait: Send + Sync {
    /// Generate keywords for a topic using AI
    /// 
    /// This method takes a topic name and canvas ID, then uses AI to generate
    /// relevant keywords that can be used to expand the knowledge map.
    /// 
    /// # Arguments
    /// * `request` - The request containing topic name, canvas ID, and generation parameters
    /// 
    /// # Returns
    /// * `Ok(GenerateKeywordsResponse)` - Successfully generated keywords
    /// * `Err(AIServiceError)` - Error during keyword generation
    async fn generate_keywords(
        &self,
        request: GenerateKeywordsRequest,
    ) -> Result<GenerateKeywordsResponse, AIServiceError>;
}
