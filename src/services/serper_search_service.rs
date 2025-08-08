use crate::services::internet_search_trait::{InternetSearchTrait, InternetSearchError, SearchResult, SearchRequest, NewsSearchRequest};
use async_trait::async_trait;
use reqwest::Client;
use serde_json::Value;
use std::time::Duration;
use tokio::time::timeout;

pub struct SerperSearchService {
    api_key: String,
    client: Client,
    base_url: String,
}

impl SerperSearchService {
    pub fn new(api_key: String) -> Result<Self, InternetSearchError> {
        if api_key.is_empty() {
            return Err(InternetSearchError::ConfigurationError("SERPER_API_KEY is not set".to_string()));
        }

        let client = Client::builder()
            .timeout(Duration::from_secs(15))
            .build()
            .map_err(|e| InternetSearchError::ConfigurationError(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self {
            api_key,
            client,
            base_url: "https://google.serper.dev".to_string(),
        })
    }

    async fn make_request(&self, endpoint: &str, body: Value) -> Result<Value, InternetSearchError> {
        let url = format!("{}{}", self.base_url, endpoint);
        
        let response = timeout(
            Duration::from_secs(15),
            self.client
                .post(&url)
                .header("Content-Type", "application/json")
                .header("X-API-KEY", &self.api_key)
                .json(&body)
                .send()
        )
        .await
        .map_err(|_| InternetSearchError::TimeoutError("Request timeout".to_string()))?
        .map_err(|e| InternetSearchError::ApiError(format!("HTTP request failed: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(InternetSearchError::ApiError(format!("API error: {}", error_text)));
        }

        let json: Value = response.json().await
            .map_err(|e| InternetSearchError::ApiError(format!("Failed to parse JSON response: {}", e)))?;

        Ok(json)
    }
}

#[async_trait]
impl InternetSearchTrait for SerperSearchService {
    async fn search(&self, request: SearchRequest) -> Result<Vec<SearchResult>, InternetSearchError> {
        let body = serde_json::json!({
            "q": request.query,
            "num": request.max_results.unwrap_or(3),
        });

        let response = self.make_request("/search", body).await?;
        
        let organic_results = response["organic"]
            .as_array()
            .ok_or_else(|| InternetSearchError::ApiError("Invalid response format: missing organic results".to_string()))?;

        let search_results: Vec<SearchResult> = organic_results
            .iter()
            .take(request.max_results.unwrap_or(3) as usize)
            .map(|result| {
                SearchResult {
                    title: result["title"].as_str().unwrap_or("").to_string(),
                    url: result["link"].as_str().unwrap_or("").to_string(),
                    content: result["snippet"].as_str().unwrap_or("").to_string(),
                    published_date: None, // Serper doesn't provide published_date in organic results
                }
            })
            .collect();

        Ok(search_results)
    }

    async fn search_latest_news(&self, request: NewsSearchRequest) -> Result<Vec<SearchResult>, InternetSearchError> {
        let body = serde_json::json!({
            "q": format!("{} news", request.query),
            "num": request.max_results.unwrap_or(5),
            "tbs": "qdr:7d", // Last 7 days
        });

        let response = self.make_request("/search", body).await?;
        
        let organic_results = response["organic"]
            .as_array()
            .ok_or_else(|| InternetSearchError::ApiError("Invalid response format: missing organic results".to_string()))?;

        let search_results: Vec<SearchResult> = organic_results
            .iter()
            .take(request.max_results.unwrap_or(5) as usize)
            .map(|result| {
                SearchResult {
                    title: result["title"].as_str().unwrap_or("").to_string(),
                    url: result["link"].as_str().unwrap_or("").to_string(),
                    content: result["snippet"].as_str().unwrap_or("").to_string(),
                    published_date: None, // Serper doesn't provide published_date in organic results
                }
            })
            .collect();

        Ok(search_results)
    }
}
