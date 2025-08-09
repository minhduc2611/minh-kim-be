use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use crate::services::vertex_ai_service_trait::{
    VertexAIServiceTrait, VertexAIServiceError, VertexAIRequestConfig, 
    VertexAIConfig, ChatRequest, ChatResponse
};

/// Helper function to get a fresh access token
fn get_fresh_access_token() -> String {
    // First try to get from environment variable
    if let Ok(token) = std::env::var("GOOGLE_ACCESS_TOKEN") {
        return token;
    }
    
    // Try to get from gcloud if available
    match std::process::Command::new("gcloud")
        .arg("auth")
        .arg("print-access-token")
        .output() {
            Ok(output) => {
                if output.status.success() {
                    String::from_utf8(output.stdout).unwrap_or_else(|_| "dummy-token".to_string()).trim().to_string()
                } else {
                    "dummy-token".to_string()
                }
            }
            Err(_) => "dummy-token".to_string(),
        }
}

/// Request structure for Vertex AI API
#[derive(Debug, Serialize)]
struct VertexAIRequest {
    contents: Vec<Content>,
    generation_config: Option<GenerationConfig>,
    model: Option<String>,
    system_instruction: Option<SystemInstruction>,
}

/// Content structure for Vertex AI API
#[derive(Debug, Serialize)]
struct Content {
    parts: Vec<Part>,
    role: String,
}

/// Part structure for Vertex AI API
#[derive(Debug, Serialize)]
struct Part {
    text: String,
}

/// Generation configuration for Vertex AI API
#[derive(Debug, Serialize)]
struct GenerationConfig {
    #[serde(rename = "thinking_config")]
    thinking_config: Option<ThinkingConfig>,
    temperature: Option<f32>,
    #[serde(rename = "topK")]
    top_k: Option<f32>,
    #[serde(rename = "topP")]
    top_p: Option<f32>,
}

/// Thinking configuration for Vertex AI API
#[derive(Debug, Serialize)]
struct ThinkingConfig {
    #[serde(rename = "include_thoughts")]
    include_thoughts: bool,
}

/// System instruction structure for Vertex AI API
#[derive(Debug, Serialize)]
struct SystemInstruction {
    parts: Vec<Part>,
    role: String,
}

/// Response structure from Vertex AI API
#[derive(Debug, Deserialize)]
struct VertexAIResponse {
    candidates: Option<Vec<Candidate>>,
    #[serde(rename = "promptFeedback")]
    prompt_feedback: Option<PromptFeedback>,
}

/// Candidate structure from Vertex AI API response
#[derive(Debug, Deserialize)]
struct Candidate {
    content: Option<ResponseContent>,
    #[serde(rename = "finishReason")]
    finish_reason: Option<String>,
    #[serde(rename = "safetyRatings")]
    safety_ratings: Option<Vec<SafetyRating>>,
}

/// Response content structure from Vertex AI API
#[derive(Debug, Deserialize)]
struct ResponseContent {
    parts: Vec<ResponsePart>,
    role: String,
}

/// Response part structure from Vertex AI API
#[derive(Debug, Deserialize)]
struct ResponsePart {
    text: Option<String>,
}

/// Prompt feedback structure from Vertex AI API
#[derive(Debug, Deserialize)]
struct PromptFeedback {
    #[serde(rename = "safetyRatings")]
    safety_ratings: Option<Vec<SafetyRating>>,
}

/// Safety rating structure from Vertex AI API
#[derive(Debug, Deserialize)]
struct SafetyRating {
    category: Option<String>,
    probability: Option<String>,
}

/// Tokio-based Vertex AI service that implements the VertexAIServiceTrait
/// 
/// This service uses tokio and reqwest to make HTTP requests to the Vertex AI API.
/// It's an alternative to the google-cloud-aiplatform-v1 client library.
/// 
/// # Example
/// ```rust
/// use crate::services::tokio_vertex_ai_service::TokioVertexAIService;
/// use crate::services::vertex_ai_service_trait::{VertexAIConfig, VertexAIServiceTrait};
/// 
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let config = VertexAIConfig {
///         project_id: "your-project-id".to_string(),
///         location: "us-central1".to_string(),
///         verbose: true,
///     };
///     
///     let service = TokioVertexAIService::new(Some(config));
///     
///     let response = service.generate_content("Hello, world!", None).await?;
///     println!("Response: {}", response);
///     
///     Ok(())
/// }
/// ```
pub struct TokioVertexAIService {
    config: VertexAIConfig,
    client: reqwest::Client,
}

impl TokioVertexAIService {
    /// Creates a new TokioVertexAIService instance
    /// 
    /// # Arguments
    /// * `config` - Optional VertexAIConfig. If None, uses default values
    pub fn new(config: Option<VertexAIConfig>) -> Self {
        let client = reqwest::Client::new();

        Self {
            config: config.unwrap_or_default(),
            client,
        }
    }

    /// Sets verbose mode for the service
    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.config.verbose = verbose;
        self
    }

    /// Builds the URL for the Vertex AI API request
    fn build_url(&self, model_id: &str) -> String {
        format!(
            "https://{}-aiplatform.googleapis.com/v1/projects/{}/locations/{}/publishers/google/models/{}:generateContent",
            self.config.location,
            self.config.project_id,
            self.config.location,
            model_id
        )
    }

    /// Makes the actual HTTP request to the Vertex AI API
    async fn make_request(&self, request: VertexAIRequest, model_id: &str) -> Result<String, VertexAIServiceError> {
        let url = self.build_url(model_id);
        
        if self.config.verbose {
            println!("=== TOKIO VERTEX AI VERBOSE MODE ===");
            println!("Making request to: {}", url);
            println!("Request body: {}", serde_json::to_string_pretty(&request)
                .unwrap_or_else(|_| "{}".to_string()));
        }

        // Serialize the request to JSON first
        let json_body = serde_json::to_string(&request)
            .map_err(|e| VertexAIServiceError::ApiError(format!("Failed to serialize request: {}", e)))?;

        // Get fresh access token for each request
        let fresh_access_token = get_fresh_access_token();
        let auth_header_value = format!("Bearer {}", fresh_access_token.trim());

        // Build the request step by step to identify where the issue is
        let request_builder = self.client
            .post(&url)
            .header("content-type", "application/json")
            .header("authorization", auth_header_value);



        let response = request_builder
            .body(json_body)
            .send()
            .await
            .map_err(|e| VertexAIServiceError::ApiError(format!("HTTP request failed: {} - URL: {}", e, url)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(VertexAIServiceError::ApiError(format!(
                "API request failed with status {}: {} - URL: {}", 
                status, 
                error_text,
                url
            )));
        }

        let response_data: VertexAIResponse = response.json().await
            .map_err(|e| VertexAIServiceError::ApiError(format!("Failed to parse response: {}", e)))?;

        if let Some(candidates) = response_data.candidates {
            if let Some(first_candidate) = candidates.first() {
                if let Some(content) = &first_candidate.content {
                    let mut response_text = String::new();
                    for part in &content.parts {
                        if let Some(text) = &part.text {
                            response_text.push_str(text);
                        }
                    }
                    return Ok(response_text);
                }
            }
        }

        Err(VertexAIServiceError::GenerationFailed("No content found in response".to_string()))
    }
}

#[async_trait]
impl VertexAIServiceTrait for TokioVertexAIService {
    async fn chat(&self, request: &ChatRequest) -> Result<ChatResponse, VertexAIServiceError> {
        let response_text = self.generate_content(&request.prompt, None).await?;
        
        Ok(ChatResponse {
            response: response_text,
            prompt: request.prompt.clone(),
            context: request.context.clone(),
            history: request.history.clone(),
            agent_key: request.agent_key.clone(),
        })
    }

    async fn generate_content(&self, prompt: &str, request_config: Option<VertexAIRequestConfig>) -> Result<String, VertexAIServiceError> {
        let request_config = request_config.unwrap_or(VertexAIRequestConfig {
            model_id: "gemini-2.5-pro".to_string(),
            agent_key: None,
            system_prompt: None,
            include_thoughts: true,
            use_google_search: false,
            use_retrieval: false,
            response_schema: None,
        });

        // Build the request
        let contents = vec![Content {
            parts: vec![Part {
                text: prompt.to_string(),
            }],
            role: "user".to_string(),
        }];

        let generation_config = GenerationConfig {
            thinking_config: if request_config.include_thoughts {
                Some(ThinkingConfig {
                    include_thoughts: true,
                })
            } else {
                None
            },
            temperature: Some(0.2),
            top_k: Some(40.0),
            top_p: Some(1.0),
        };

        let mut system_instruction = None;
        if let Some(system_prompt) = &request_config.system_prompt {
            system_instruction = Some(SystemInstruction {
                parts: vec![Part {
                    text: system_prompt.clone(),
                }],
                role: "system".to_string(),
            });
        }

        let vertex_request = VertexAIRequest {
            contents,
            generation_config: Some(generation_config),
            model: Some(format!(
                "projects/{}/locations/{}/publishers/google/models/{}",
                self.config.project_id,
                self.config.location,
                request_config.model_id
            )),
            system_instruction,
        };

        self.make_request(vertex_request, &request_config.model_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_build_url() {
        let config = VertexAIConfig {
            project_id: "llm-project-2d719".to_string(),
            location: "us-central1".to_string(),
            verbose: true,
        };

        let service = TokioVertexAIService::new(Some(config));
        
        let url = service.build_url("gemini-2.5-pro");

        let request = VertexAIRequest {
            contents: vec![Content {
                parts: vec![Part {
                    text: "what is the capital of France?".to_string(),
                }],
                role: "user".to_string(),
            }],
            generation_config: Some(GenerationConfig {
                thinking_config: Some(ThinkingConfig {
                    include_thoughts: true,
                }),
                temperature: Some(0.2),
                top_k: Some(40.0),
                top_p: Some(1.0),
            }),
            model: Some(url),
            system_instruction: Some(SystemInstruction {
                parts: vec![Part {
                    text: "You are a helpful assistant.".to_string(),
                }],
                role: "system".to_string(),
            }),
        };

        let response = service.make_request(request, "gemini-2.5-pro").await;
        println!("-----> Response: {:?}", response);
    }
}
