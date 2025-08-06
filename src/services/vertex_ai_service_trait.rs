use async_trait::async_trait;
use serde::{Deserialize, Serialize};

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

#[derive(Debug, Deserialize)]
pub struct ChatRequest {
    pub history: Option<Vec<String>>,
    pub context: Option<String>,
    pub prompt: String,
    pub agent_key: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ChatResponse {
    pub response: String,
    pub prompt: String,
    pub context: Option<String>,
    pub history: Option<Vec<String>>,
    pub agent_key: Option<String>,
}

pub struct VertexAIConfig {
    pub project_id: String,
    pub location: String,
    
}

impl Default for VertexAIConfig {
    fn default() -> Self {
        Self {
            project_id: "llm-project-2d719".to_string(),
            location: "us-central1".to_string(),
        }
    }
}
pub struct VertexAIRequestConfig {
    pub model_id: String,
    pub agent_key: Option<String>,
    pub system_prompt: Option<String>,
    pub include_thoughts: bool,
    pub use_google_search: bool,
    pub use_retrieval: bool,
}


#[async_trait]
pub trait VertexAIServiceTrait: Send + Sync {
    async fn chat(&self, request: &ChatRequest) -> Result<ChatResponse, VertexAIServiceError>;
    async fn generate_content(&self, prompt: &str, request_config: Option<VertexAIRequestConfig>) -> Result<String, VertexAIServiceError>;
} 