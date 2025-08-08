use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::Duration;
use tokio::time::timeout;

#[derive(Debug, thiserror::Error)]
pub enum WeaviateError {
    #[error("Configuration error: {0}")]
    ConfigurationError(String),
    #[error("API error: {0}")]
    ApiError(String),
    #[error("Timeout error: {0}")]
    TimeoutError(String),
    #[error("Search failed: {0}")]
    SearchFailed(String),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WeaviateSearchResult {
    pub id: String,
    pub score: f64,
    pub properties: Value,
    pub metadata: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WeaviateSearchRequest {
    pub query: String,
    pub class_name: String,
    pub limit: Option<i32>,
    pub distance: Option<f64>,
    pub additional_properties: Option<Vec<String>>,
}

pub struct WeaviateClient {
    url: String,
    api_key: Option<String>,
    client: Client,
}

impl WeaviateClient {
    pub fn new(url: String, api_key: Option<String>) -> Result<Self, WeaviateError> {
        if url.is_empty() {
            return Err(WeaviateError::ConfigurationError("WEAVIATE_URL is not set".to_string()));
        }

        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| WeaviateError::ConfigurationError(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self {
            url: url.trim_end_matches('/').to_string(),
            api_key,
            client,
        })
    }

    async fn make_request(&self, endpoint: &str, method: &str, body: Option<Value>) -> Result<Value, WeaviateError> {
        let url = format!("{}{}", self.url, endpoint);
        
        let mut request_builder = match method {
            "GET" => self.client.get(&url),
            "POST" => self.client.post(&url),
            "PUT" => self.client.put(&url),
            "DELETE" => self.client.delete(&url),
            _ => return Err(WeaviateError::ConfigurationError(format!("Unsupported HTTP method: {}", method))),
        };

        // Add headers
        request_builder = request_builder.header("Content-Type", "application/json");
        
        if let Some(api_key) = &self.api_key {
            request_builder = request_builder.header("Authorization", format!("Bearer {}", api_key));
        }

        // Add body if provided
        if let Some(body_data) = body {
            request_builder = request_builder.json(&body_data);
        }

        let response = timeout(
            Duration::from_secs(30),
            request_builder.send()
        )
        .await
        .map_err(|_| WeaviateError::TimeoutError("Request timeout".to_string()))?
        .map_err(|e| WeaviateError::ApiError(format!("HTTP request failed: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(WeaviateError::ApiError(format!("API error: {}", error_text)));
        }

        let json: Value = response.json().await
            .map_err(|e| WeaviateError::ApiError(format!("Failed to parse JSON response: {}", e)))?;

        Ok(json)
    }

    pub async fn search(&self, request: WeaviateSearchRequest) -> Result<Vec<WeaviateSearchResult>, WeaviateError> {
        let limit = request.limit.unwrap_or(10);
        let distance = request.distance.unwrap_or(0.7);

        let body = serde_json::json!({
            "class": request.class_name,
            "properties": request.additional_properties.unwrap_or_else(|| vec!["content".to_string(), "filename".to_string(), "description".to_string()]),
            "vector": self.generate_embedding(&request.query).await?,
            "limit": limit,
            "distance": distance,
        });

        let response = self.make_request("/v1/objects", "GET", Some(body)).await?;
        
        let results = response["data"]["Get"][&request.class_name]
            .as_array()
            .ok_or_else(|| WeaviateError::ApiError("Invalid response format: missing results".to_string()))?;

        let search_results: Vec<WeaviateSearchResult> = results
            .iter()
            .map(|result| {
                WeaviateSearchResult {
                    id: result["id"].as_str().unwrap_or("").to_string(),
                    score: result["_additional"]["distance"].as_f64().unwrap_or(0.0),
                    properties: result["properties"].clone(),
                    metadata: result["_additional"]["metadata"].as_object().map(|m| serde_json::to_value(m).unwrap_or_default()),
                }
            })
            .collect();

        Ok(search_results)
    }

    async fn generate_embedding(&self, text: &str) -> Result<Vec<f64>, WeaviateError> {
        // For now, we'll use a simple placeholder embedding
        // In a real implementation, you would call an embedding service
        // This is a placeholder that returns a dummy embedding
        Ok(vec![0.1; 1536]) // OpenAI embedding dimension
    }

    pub async fn health_check(&self) -> Result<bool, WeaviateError> {
        let response = self.make_request("/v1/meta", "GET", None).await?;
        Ok(response["hostname"].is_string())
    }
}
