use google_cloud_aiplatform_v1::client::PredictionService;
use google_cloud_aiplatform_v1::model::{
    GenerateContentRequest, GenerationConfig, Part, Content,
};
use google_cloud_aiplatform_v1::model::generation_config::ThinkingConfig;
use google_cloud_aiplatform_v1::model::part::Data;
use serde::{Deserialize, Serialize};
use async_trait::async_trait;
use crate::services::vertex_ai_service_trait::{VertexAIServiceTrait, VertexAIServiceError};

use crate::services::agents_service::get_mock_agents;

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
    pub model_id: String,
    pub include_thoughts: bool,
    pub agent_key: Option<String>,
}

impl Default for VertexAIConfig {
    fn default() -> Self {
        Self {
            project_id: "llm-project-2d719".to_string(),
            location: "us-central1".to_string(),
            model_id: "gemini-2.0-flash-001".to_string(),
            include_thoughts: false,
            agent_key: None,
        }
    }
}

pub struct VertexAIService {
    config: VertexAIConfig,
}

impl VertexAIService {
    pub fn new(config: Option<VertexAIConfig>) -> Self {
        Self {
            config: config.unwrap_or_default(),
        }
    }

    pub async fn generate_content(&self, prompt: &str) -> Result<String, VertexAIServiceError> {
        println!("VertexAIService::generate_content called with prompt: {}", prompt);
        
        let mut model_name = format!(
            "projects/{}/locations/{}/publishers/google/models/{}",
            self.config.project_id, self.config.location, self.config.model_id
        );

        let mut system_prompt = String::new();
        let mut temperature = 0.2;
        let agent_key = self.config.agent_key.as_deref().unwrap_or("code_assistant_pro");
        let agents = get_mock_agents();
        let agent = agents.iter().find(|a| a.key == agent_key);
        if let Some(agent) = agent {
            system_prompt = agent.system_prompt.clone();
            model_name = format!(
                "projects/{}/locations/{}/publishers/google/models/{}",
                self.config.project_id, self.config.location, agent.model
            );
            temperature = agent.temperature;
        }

        // Create the API Client
        let prediction_client = PredictionService::builder().build().await
            .map_err(|e| VertexAIServiceError::ApiError(e.to_string()))?;

        // Construct the Request
        let mut user_content = Content::default();
        user_content.role = "user".to_string();
        
        let mut part = Part::default();
        part.data = Some(Data::Text(prompt.to_string()));
        user_content.parts = vec![part];

        let mut generation_config = GenerationConfig::default();
        generation_config.temperature = Some(temperature);
        generation_config.top_p = Some(1.0);
        generation_config.top_k = Some(40.0);
        generation_config.max_output_tokens = Some(2048);
        if self.config.include_thoughts {
            let mut thinking_config = ThinkingConfig::default();
            thinking_config.include_thoughts = Some(true);
            generation_config.thinking_config = Some(thinking_config);
        }

        let mut request = GenerateContentRequest::default();
        request.model = model_name.clone();
        request.contents = vec![user_content];
        request.generation_config = Some(generation_config);
        request.system_instruction = Some(Content::new()
            .set_role("system")
            .set_parts(
                vec![Part::new().set_data(Data::Text(system_prompt))]
            ));

        // Call the API and Get the Response
        let response = prediction_client
            .generate_content()
            .with_request(request)
            .send()
            .await
            .map_err(|e| VertexAIServiceError::ApiError(e.to_string()))?;

        let mut response_text = String::new();

        for candidate in response.candidates {
            if let Some(content) = candidate.content {
                for part in content.parts {
                    if let Some(Data::Text(text)) = part.data {
                        response_text.push_str(&text);
                    }
                }
            }
        }

        println!("VertexAIService::generate_content returning response");
        Ok(response_text)
    }

    pub async fn chat(&self, request: &ChatRequest) -> Result<ChatResponse, VertexAIServiceError> {
        println!("VertexAIService::chat called with request: {:?}", request);
        
        let response_text = self.generate_content(&request.prompt).await?;
        
        let response = ChatResponse {
            response: response_text,
            prompt: request.prompt.clone(),
            context: request.context.clone(),
            history: request.history.clone(),
            agent_key: request.agent_key.clone(),
        };
        
        println!("VertexAIService::chat returning response");
        Ok(response)
    }
}

#[async_trait]
impl VertexAIServiceTrait for VertexAIService {
    async fn generate_content(&self, prompt: &str) -> Result<String, VertexAIServiceError> {
        println!("VertexAIServiceTrait::generate_content called");
        self.generate_content(prompt).await
    }

    async fn chat(&self, request: &ChatRequest) -> Result<ChatResponse, VertexAIServiceError> {
        println!("VertexAIServiceTrait::chat called");
        self.chat(request).await
    }
}
