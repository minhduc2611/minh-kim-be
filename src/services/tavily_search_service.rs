use crate::services::internet_search_trait::{InternetSearchTrait, InternetSearchError, SearchResult, SearchRequest, NewsSearchRequest};
use async_trait::async_trait;
use reqwest::Client;
use serde_json::Value;
use std::time::Duration;
use tokio::time::timeout;

pub struct TavilySearchService {
    api_key: String,
    client: Client,
    base_url: String,
}

impl TavilySearchService {
    pub fn new(api_key: String) -> Result<Self, InternetSearchError> {
        if api_key.is_empty() {
            return Err(InternetSearchError::ConfigurationError("TAVILY_API_KEY is not set".to_string()));
        }

        let client = Client::builder()
            .timeout(Duration::from_secs(15))
            .build()
            .map_err(|e| InternetSearchError::ConfigurationError(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self {
            api_key,
            client,
            base_url: "https://api.tavily.com".to_string(),
        })
    }

    async fn make_request(&self, endpoint: &str, body: Value) -> Result<Value, InternetSearchError> {
        let url = format!("{}{}", self.base_url, endpoint);
        
        let response = timeout(
            Duration::from_secs(15),
            self.client
                .post(&url)
                .header("Content-Type", "application/json")
                .header("Authorization", format!("Bearer {}", self.api_key))
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
impl InternetSearchTrait for TavilySearchService {
    async fn search(&self, request: SearchRequest) -> Result<Vec<SearchResult>, InternetSearchError> {
        let body = serde_json::json!({
            "query": request.query,
            "search_depth": request.search_depth.unwrap_or_else(|| "basic".to_string()),
            "include_raw_content": request.include_raw_content.unwrap_or(false),
            "max_results": request.max_results.unwrap_or(3),
        });

        let response = self.make_request("/search", body).await?;
        
        let results = response["results"]
            .as_array()
            .ok_or_else(|| InternetSearchError::ApiError("Invalid response format: missing results".to_string()))?;

        let search_results: Vec<SearchResult> = results
            .iter()
            .map(|result| {
                SearchResult {
                    title: result["title"].as_str().unwrap_or("").to_string(),
                    url: result["url"].as_str().unwrap_or("").to_string(),
                    content: result["content"].as_str().unwrap_or("").to_string(),
                    published_date: result["published_date"].as_str().map(|s| s.to_string()),
                }
            })
            .collect();

        Ok(search_results)
    }

    async fn search_latest_news(&self, request: NewsSearchRequest) -> Result<Vec<SearchResult>, InternetSearchError> {
        let body = serde_json::json!({
            "query": request.query,
            "search_depth": "basic",
            "include_raw_content": false,
            "max_results": request.max_results.unwrap_or(5),
            "include_domains": ["news.google.com", "reuters.com", "bbc.com", "cnn.com", "techcrunch.com"],
            "time_period": request.time_period.unwrap_or_else(|| "7d".to_string()),
        });

        let response = self.make_request("/search", body).await?;
        
        let results = response["results"]
            .as_array()
            .ok_or_else(|| InternetSearchError::ApiError("Invalid response format: missing results".to_string()))?;

        let search_results: Vec<SearchResult> = results
            .iter()
            .map(|result| {
                SearchResult {
                    title: result["title"].as_str().unwrap_or("").to_string(),
                    url: result["url"].as_str().unwrap_or("").to_string(),
                    content: result["content"].as_str().unwrap_or("").to_string(),
                    published_date: result["published_date"].as_str().map(|s| s.to_string()),
                }
            })
            .collect();

        Ok(search_results)
    }
}
