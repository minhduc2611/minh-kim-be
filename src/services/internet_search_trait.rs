use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, thiserror::Error)]
pub enum InternetSearchError {
    #[error("Search failed: {0}")]
    SearchFailed(String),
    #[error("Configuration error: {0}")]
    ConfigurationError(String),
    #[error("API error: {0}")]
    ApiError(String),
    #[error("Timeout error: {0}")]
    TimeoutError(String),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SearchResult {
    pub title: String,
    pub url: String,
    pub content: String,
    pub published_date: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SearchRequest {
    pub query: String,
    pub max_results: Option<i32>,
    pub search_depth: Option<String>,
    pub include_raw_content: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NewsSearchRequest {
    pub query: String,
    pub max_results: Option<i32>,
    pub time_period: Option<String>, // e.g., "1d", "7d", "1m"
}

#[async_trait]
pub trait InternetSearchTrait: Send + Sync {
    /// Perform a general web search
    /// 
    /// # Arguments
    /// * `request` - The search request containing query and parameters
    /// 
    /// # Returns
    /// * `Ok(Vec<SearchResult>)` - Search results
    /// * `Err(InternetSearchError)` - Error during search
    async fn search(&self, request: SearchRequest) -> Result<Vec<SearchResult>, InternetSearchError>;

    /// Search for latest news
    /// 
    /// # Arguments
    /// * `request` - The news search request containing query and parameters
    /// 
    /// # Returns
    /// * `Ok(Vec<SearchResult>)` - News search results
    /// * `Err(InternetSearchError)` - Error during search
    async fn search_latest_news(&self, request: NewsSearchRequest) -> Result<Vec<SearchResult>, InternetSearchError>;
}
