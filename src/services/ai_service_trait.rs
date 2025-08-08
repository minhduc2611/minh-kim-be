use crate::services::ai_service::{GenerateKeywordsRequest, GenerateKeywordsResponse};
use crate::models::common::{GenerateInsightsRequest, GenerateInsightsResponse, GenerateInsightsForTopicNodeRequest, GenerateInsightsForTopicNodeResponse};
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
    #[error("Search service error: {0}")]
    SearchServiceError(String),
    #[error("Weaviate error: {0}")]
    WeaviateError(String),
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

    /// Generate comprehensive insights using AI with web search and document context
    /// 
    /// This method takes a question and optional context, then uses AI to generate
    /// comprehensive insights by combining web search results and document context.
    /// 
    /// # Arguments
    /// * `request` - The request containing question, system instruction, topic path, and document context
    /// 
    /// # Returns
    /// * `Ok(GenerateInsightsResponse)` - Successfully generated insights
    /// * `Err(AIServiceError)` - Error during insights generation
    async fn generate_insights(
        &self,
        request: GenerateInsightsRequest,
    ) -> Result<GenerateInsightsResponse, AIServiceError>;

    /// Generate comprehensive insights for a specific topic node using AI with web search, news search, and document context
    /// 
    /// This method takes a topic node ID and canvas ID, then uses AI to generate
    /// comprehensive insights by combining web search results, news search results, and document context.
    /// 
    /// # Arguments
    /// * `request` - The request containing topic node ID, canvas ID, and generation parameters
    /// 
    /// # Returns
    /// * `Ok(GenerateInsightsForTopicNodeResponse)` - Successfully generated insights
    /// * `Err(AIServiceError)` - Error during insights generation
    async fn generate_insights_for_topic_node(
        &self,
        request: GenerateInsightsForTopicNodeRequest,
    ) -> Result<GenerateInsightsForTopicNodeResponse, AIServiceError>;
}
