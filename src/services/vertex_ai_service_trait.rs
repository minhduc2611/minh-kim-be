use async_trait::async_trait;
use crate::services::vertex_ai_service::{ChatRequest, ChatResponse};

#[derive(Debug, thiserror::Error)]
pub enum VertexAIServiceError {
    #[error("Generation failed: {0}")]
    GenerationFailed(String),
    #[error("Configuration error: {0}")]
    ConfigurationError(String),
    #[error("API error: {0}")]
    ApiError(String),
    #[error("Agent not found: {0}")]
    AgentNotFound(String),
}

#[async_trait]
pub trait VertexAIServiceTrait: Send + Sync {
    async fn chat(&self, request: &ChatRequest) -> Result<ChatResponse, VertexAIServiceError>;
    async fn generate_content(&self, prompt: &str) -> Result<String, VertexAIServiceError>;
} 