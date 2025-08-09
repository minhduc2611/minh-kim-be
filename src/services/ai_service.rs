use crate::dao::canvas_dao_trait::CanvasRepository;
use crate::dao::node_dao_trait::NodeRepository;
use crate::models::canvas::GraphNode;
use crate::models::node::{InsertNode, InsertRelationship};
use crate::models::common::{GenerateInsightsRequest, GenerateInsightsResponse, GenerateInsightsForTopicNodeRequest, GenerateInsightsForTopicNodeResponse, SearchResult, DocumentContext};
use crate::services::ai_service_trait::{AIServiceError, AIServiceTrait};
use crate::services::tokio_vertex_ai_service::TokioVertexAIService;
use crate::services::vertex_ai_service::VertexAIService;
use crate::services::vertex_ai_service_trait::{VertexAIRequestConfig, VertexAIServiceTrait};
use crate::services::internet_search_trait::{InternetSearchTrait, SearchRequest as InternetSearchRequest, NewsSearchRequest};
use crate::services::weaviate_client::WeaviateClient;
use async_trait::async_trait;
use google_cloud_aiplatform_v1::model::{Schema, Type};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{Datelike, Utc};

use std::sync::Arc;

#[derive(Debug, Deserialize)]
pub struct GenerateKeywordsRequest {
    pub topic_name: String,
    pub canvas_id: String,
    pub node_count: Option<i32>,
    pub is_automatic: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct GenerateKeywordsResponse {
    pub keywords: Vec<String>,
    pub edges: Vec<String>, // For future use
}

#[derive(Debug, Serialize)]
struct LocalSearchResult {
    filename: String,
    text: String,
}

pub struct AIService {
    canvas_repository: Arc<dyn CanvasRepository + Send + Sync>,
    node_repository: Arc<dyn NodeRepository + Send + Sync>,
    tokio_vertex_ai_service: TokioVertexAIService,
    vertex_ai_service: VertexAIService,
    internet_search_service: Option<Arc<dyn InternetSearchTrait + Send + Sync>>,
    weaviate_client: Option<WeaviateClient>,
}

impl AIService {
    pub fn new(
        canvas_repository: Arc<dyn CanvasRepository + Send + Sync>,
        node_repository: Arc<dyn NodeRepository + Send + Sync>,
        tokio_vertex_ai_service: TokioVertexAIService,
        vertex_ai_service: VertexAIService,
    ) -> Self {
        Self {
            canvas_repository,
            node_repository,
            tokio_vertex_ai_service,
            vertex_ai_service,
            internet_search_service: None,
            weaviate_client: None,
        }
    }

    pub async fn generate_keywords(
        &self,
        request: GenerateKeywordsRequest,
    ) -> Result<GenerateKeywordsResponse, AIServiceError> {
        let node_count = request.node_count.unwrap_or(3);
        let is_automatic = request.is_automatic.unwrap_or(false);

        // Check if canvas exists
        let canvas = self
            .canvas_repository
            .get_canvas_by_id(&request.canvas_id)
            .await
            .map_err(|e| AIServiceError::DatabaseError(format!("Failed to get canvas: {}", e)))?;

        let canvas =
            canvas.ok_or_else(|| AIServiceError::CanvasNotFound(request.canvas_id.clone()))?;

        // Get the source topic node by name
        let source_topic = self
            .get_topic_by_name_and_canvas(&request.topic_name, &request.canvas_id)
            .await
            .map_err(|e| AIServiceError::DatabaseError(e.to_string()))?;

        let source_topic = source_topic
            .ok_or_else(|| AIServiceError::TopicNotFound(request.topic_name.clone()))?;

        // Get the hierarchical path, existing siblings, and children for context
        let topic_path = self
            .get_topic_path(&source_topic.id, &request.canvas_id)
            .await
            .map_err(|e| AIServiceError::DatabaseError(e.to_string()))?;

        let existing_siblings = self
            .get_existing_siblings(&source_topic.id, &request.canvas_id)
            .await
            .map_err(|e| AIServiceError::DatabaseError(e.to_string()))?;

        let topic_children = self
            .get_topic_children(&source_topic.id, &request.canvas_id)
            .await
            .map_err(|e| AIServiceError::DatabaseError(e.to_string()))?;

        // Search for relevant document chunks using Weaviate (placeholder for now)
        let relevant_chunks: Vec<LocalSearchResult> = Vec::new(); // TODO: Implement Weaviate search

        // Build the prompt for AI
        let system_instruction_section = if !canvas.system_instruction.is_empty() {
            format!(
                "<system-instruction>\n{}\n</system-instruction>",
                canvas.system_instruction
            )
        } else {
            String::new()
        };

        // Build document context section
        let document_context_section = if !relevant_chunks.is_empty() {
            let chunks_text = relevant_chunks
                .iter()
                .enumerate()
                .map(|(index, chunk)| {
                    let truncated_text = if chunk.text.len() > 500 {
                        format!("{}...", &chunk.text[..500])
                    } else {
                        chunk.text.clone()
                    };
                    format!(
                        "Document {} ({}):\n\"{}\"",
                        index + 1,
                        chunk.filename,
                        truncated_text
                    )
                })
                .collect::<Vec<_>>()
                .join("\n");

            format!(
                "<document-context>\nThe following are relevant document excerpts from the user's knowledge base related to \"{}\":\n\n{}\n\nUse this context to generate more informed and specific keywords that complement the existing knowledge.\n</document-context>",
                request.topic_name, chunks_text
            )
        } else {
            String::new()
        };

        let automatic_instructions = if is_automatic {
            "- Mode: Automatic (generate an optimal number of keywords, maximum 15, based on the topic complexity, depth, and available context)".to_string()
        } else {
            format!("- Node Count: {}", node_count)
        };

        let children_section = if !topic_children.is_empty() {
            format!(
                "- Children: [{}]",
                topic_children
                    .iter()
                    .map(|c| format!("\"{}\"", c))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        } else {
            String::new()
        };

        let context_info = if !relevant_chunks.is_empty() {
            format!(
                "- Available Context: {} relevant document chunks found",
                relevant_chunks.len()
            )
        } else {
            "- Available Context: No document chunks found for this topic".to_string()
        };

        let topic_path_str = topic_path
            .iter()
            .map(|p| format!("\"{}\"", p))
            .collect::<Vec<_>>()
            .join(", ");

        let children_section_str = if !children_section.is_empty() {
            format!("{}\n", children_section)
        } else {
            String::new()
        };

        let existing_siblings_str = if !existing_siblings.is_empty() {
            existing_siblings
                .iter()
                .map(|s| format!("\"{}\"", s))
                .collect::<Vec<_>>()
                .join(", ")
        } else {
            String::new()
        };

        let input = format!(
            "- Topic: \"{}\"\n- Topic Path: [{}]\n{}- Existing Siblings: [{}]\n{}\n{}",
            request.topic_name,
            topic_path_str,
            children_section_str,
            existing_siblings_str,
            automatic_instructions,
            context_info
        );

        let instructions = format!(
            r#"<persona>
You are an expert in curriculum design and knowledge architecture. Your task is to generate keywords for a knowledge map to help a user learn a topic systematically.
You will be given a 'topic', its hierarchical 'topicPath', existing 'children' (if any), a list of 'existingSiblings' to avoid, {}, and relevant document context from the user's knowledge base.
</persona>
{}
{}
<task-description>
  Your generated keywords MUST follow these rules:
  <hierarchical-specificity>
    The specificity of your keywords must adapt to the depth of the 'topicPath'.
    * Shallow Path (1-2 levels deep): Generate broader, foundational sub-topics.
    * Deep Path (3+ levels deep): Generate more specific, niche concepts, applications, or tools.  
  </hierarchical-specificity>
  <content-rich-mix>
    Provide a mix of core concepts, practical applications, and emerging trends.
    {}
  </content-rich-mix>
  <avoid-redundancy>
    Do not repeat the 'topic' itself, any keywords from the 'existingSiblings' list, or any existing 'children'.
  </avoid-redundancy>
  <children-awareness>
    If the topic already has children, consider the gaps or complementary areas that haven't been covered yet.
  </children-awareness>
  {}
</task-description>
"#,
            if is_automatic {
                "you should determine the optimal number of keywords (maximum 15)"
            } else {
                "the desired 'nodeCount'"
            },
            system_instruction_section,
            document_context_section,
            if !relevant_chunks.is_empty() {
                "Leverage the provided document context to generate keywords that build upon or complement the existing knowledge."
            } else {
                ""
            },
            if is_automatic {
                r#"<automatic-count>
    Since this is automatic mode, determine the optimal number of keywords based on:
    * Topic complexity and breadth
    * Depth in the learning hierarchy
    * Existing siblings count
    * Available document context richness
    * Generate between 3-15 keywords as appropriate, prioritizing quality over quantity
  </automatic-count>"#
            } else {
                ""
            }
        );

        let items:std::boxed::Box<Schema>=Box::new(Schema::default().set_type(Type::String));
        let properties: std::collections::HashMap<std::string::String, Schema> = HashMap::from([(
            "keywords".to_string(),
            Schema::default().set_type(Type::Array).set_items(*items),
        )]);
        let response_schema = Schema::default()
            .set_type(Type::Object)
            .set_properties(properties);

        let request_config = VertexAIRequestConfig {
            model_id: "gemini-2.0-flash-001".to_string(),
            agent_key: None,
            system_prompt: Some(instructions.clone()),
            include_thoughts: false,
            use_google_search: false,
            use_retrieval: false,
            response_schema: Some(response_schema),
        };
        let response = self
            .vertex_ai_service
            .generate_content(
                &format!("{}\n\n{}", instructions, input),
                Some(request_config),
            )
            .await
            .map_err(|e| AIServiceError::AIServiceError(format!("AI service error: {}", e)))?;

        // Parse the AI response
        let ai_result: serde_json::Value = serde_json::from_str(&response).map_err(|e| {
            AIServiceError::InvalidResponseFormat(format!("Failed to parse AI response: {}", e))
        })?;

        let keywords = ai_result
            .get("keywords")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect::<Vec<String>>()
            })
            .unwrap_or_default();

        let mut new_nodes = Vec::new();
        let mut new_edges = Vec::new();

        for keyword in &keywords {
            let keyword_topic = self.node_repository
                .create_topic_node(InsertNode {
                    id: uuid::Uuid::new_v4().to_string(),
                    canvas_id: request.canvas_id.clone(),
                    name: keyword.clone(),
                    node_type: "generated".to_string(),
                    description: None,
                    knowledge: None,
                    position_x: None,
                    position_y: None,
                })
                .await
                .map_err(|e| AIServiceError::DatabaseError(format!("Failed to create keyword topic: {}", e)))?;

            let keyword_topic_id = keyword_topic.id.clone();
            new_nodes.push(keyword_topic);

            let relationship_exists = self.node_repository
                .relationship_exists(&source_topic.id, &keyword_topic_id)
                .await
                .map_err(|e| AIServiceError::DatabaseError(format!("Failed to check relationship existence: {}", e)))?;

            if !relationship_exists {
                let relationship = self.node_repository
                    .create_relationship(InsertRelationship {
                        id: uuid::Uuid::new_v4().to_string(),
                        canvas_id: request.canvas_id.clone(),
                        source_id: source_topic.id.clone(),
                        target_id: keyword_topic_id,
                    })
                    .await
                    .map_err(|e| AIServiceError::DatabaseError(format!("Failed to create relationship: {}", e)))?;

                new_edges.push(relationship.id);
            }
        }

        Ok(GenerateKeywordsResponse {
            keywords,
            edges: new_edges,
        })
    }

    pub async fn generate_insights(
        &self,
        request: GenerateInsightsRequest,
    ) -> Result<GenerateInsightsResponse, AIServiceError> {
        // Build system instruction section
        let system_instruction_section = if let Some(system_instruction) = &request.system_instruction {
            format!(
                "<system-instruction>\n{}\n</system-instruction>",
                system_instruction
            )
        } else {
            String::from(
                r#"<system-instruction>
You are an AI assistant providing comprehensive insights, analysis, and real world examples. 
When given a search query, provide detailed, informative explanations.
</system-instruction>"#
            )
        };

        // Build topic path section
        let topic_path_section = if let Some(topic_path) = &request.topic_path {
            format!(
                "<topic-path>\n{}\n</topic-path>",
                topic_path
            )
        } else {
            String::new()
        };

        // Build document context section
        let document_context_section = if let Some(document_context) = &request.document_context {
            if !document_context.is_empty() {
                let context_text = document_context
                    .iter()
                    .enumerate()
                    .map(|(index, doc)| {
                        let relevance_score = ((1.0 - doc.score) * 100.0).round() as i32;
                        format!(
                            "Document {}: {} - {}\nDescription: {}\nRelevance Score: {}%\nContent: {}\n---",
                            index + 1,
                            doc.filename,
                            doc.name,
                            doc.description,
                            relevance_score,
                            doc.text
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("\n");

                format!(
                    "<user-documents>\n{}\n</user-documents>",
                    context_text
                )
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        // Get current year for search query
        let current_year = chrono::Utc::now().year();

        // For now, we'll use a placeholder for web search results
        // In a real implementation, you would integrate with Tavily or similar search service
        let web_search_results = vec![
            serde_json::json!({
                "title": "Sample search result",
                "link": "https://example.com",
                "knowledge": "This is a placeholder for web search results. In production, this would be populated with actual search results from Tavily or similar service."
            })
        ];

        let web_search_section = format!(
            "<web-search-results>\n{}\n</web-search-results>",
            serde_json::to_string_pretty(&web_search_results)
                .map_err(|e| AIServiceError::InvalidResponseFormat(format!("Failed to serialize web search results: {}", e)))?
        );

        // Build the complete instructions
        let instructions = format!(
            r#"<instructions>
{}
{}
{}
{}
<format>
    Using Markdown format when appropriate.
    ALWAYS reference and prioritize information from user documents when available and relevant.
    Also incorporate relevant information from web search results.
    If user documents contain relevant information, mention them specifically in your response.
    Current time: {}
</format>
</instructions>"#,
            system_instruction_section,
            topic_path_section,
            document_context_section,
            web_search_section,
            current_year,
        );

        // Create Vertex AI request config with Google Search enabled
        let request_config = VertexAIRequestConfig {
            model_id: "gemini-2.5-pro".to_string(),
            agent_key: None,
            system_prompt: Some(instructions.clone()),
            include_thoughts: false,
            use_google_search: true,
            use_retrieval: false,
            response_schema: None,
        };

        // Generate content using Vertex AI
        let response_text = self
            .tokio_vertex_ai_service
            .generate_content(
                &format!("Provide a comprehensive analysis of: {}", request.question),
                Some(request_config),
            )
            .await
            .map_err(|e| AIServiceError::AIServiceError(format!("AI service error: {}", e)))?;

        // Create the response
        let response = GenerateInsightsResponse {
            insights: response_text,
            question: request.question.clone(),
            generated_at: chrono::Utc::now().to_rfc3339(),
        };

        Ok(response)
    }

    pub async fn generate_insights_for_topic_node(
        &self,
        request: GenerateInsightsForTopicNodeRequest,
    ) -> Result<GenerateInsightsForTopicNodeResponse, AIServiceError> {
        // Get the topic node by ID
        let topic_node = self
            .node_repository
            .get_topic_node_by_id(&request.topic_node_id)
            .await
            .map_err(|e| AIServiceError::DatabaseError(format!("Failed to get topic node: {}", e)))?
            .ok_or_else(|| AIServiceError::TopicNotFound(request.topic_node_id.clone()))?;

        let topic_path = self
            .get_topic_path(&topic_node.id, &request.canvas_id)
            .await
            .map_err(|e| AIServiceError::DatabaseError(e.to_string()))?;

        let _existing_siblings = self
            .get_existing_siblings(&topic_node.id, &request.canvas_id)
            .await
            .map_err(|e| AIServiceError::DatabaseError(e.to_string()))?;

        let _topic_children = self
            .get_topic_children(&topic_node.id, &request.canvas_id)
            .await
            .map_err(|e| AIServiceError::DatabaseError(e.to_string()))?;

        // Build system instruction section
        let system_instruction_section = if let Some(system_instruction) = &request.system_instruction {
            format!(
                "<system-instruction>\n{}\n</system-instruction>",
                system_instruction
            )
        } else {
            String::from(
                r#"<system-instruction>
You are an AI assistant providing comprehensive insights, analysis, and real world examples. 
When given a search query, provide detailed, informative explanations.
</system-instruction>"#
            )
        };

        // Build topic path section
        let topic_path_section = if !topic_path.is_empty() {
            format!(
                "<topic-path>\n{}\n</topic-path>",
                topic_path.join(" > ")
            )
        } else {
            String::new()
        };

        // Search for document context using Weaviate if available
        let mut document_context = Vec::new();
        if let Some(weaviate_client) = &self.weaviate_client {
            let search_request = crate::services::weaviate_client::WeaviateSearchRequest {
                query: topic_node.name.clone(),
                class_name: "Document".to_string(),
                limit: Some(5),
                distance: Some(0.7),
                additional_properties: Some(vec!["content".to_string(), "filename".to_string(), "description".to_string()]),
            };

            match weaviate_client.search(search_request).await {
                Ok(results) => {
                    document_context = results
                        .iter()
                        .map(|result| DocumentContext {
                            filename: result.properties["filename"].as_str().unwrap_or("").to_string(),
                            chunk_id: result.id.clone(),
                            name: result.properties["title"].as_str().unwrap_or("").to_string(),
                            description: result.properties["description"].as_str().unwrap_or("").to_string(),
                            text: result.properties["content"].as_str().unwrap_or("").to_string(),
                            score: result.score,
                        })
                        .collect();
                }
                Err(e) => {
                    eprintln!("Weaviate search failed: {}", e);
                }
            }
        }
        println!("Found Contexts documents: {}", document_context.len());
        // Build document context section
        let document_context_section = if !document_context.is_empty() {
            let context_text = document_context
                .iter()
                .enumerate()
                .map(|(index, doc)| {
                    let relevance_score = ((1.0 - doc.score) * 100.0).round() as i32;
                    format!(
                        "Document {}: {} - {}\nDescription: {}\nRelevance Score: {}%\nContent: {}\n---",
                        index + 1,
                        doc.filename,
                        doc.name,
                        doc.description,
                        relevance_score,
                        doc.text
                    )
                })
                .collect::<Vec<_>>()
                .join("\n");

            format!(
                "<user-documents>\n{}\n</user-documents>",
                context_text
            )
        } else {
            String::new()
        };

        // Perform web search if requested
        let mut web_search_results: Option<Vec<SearchResult>> = None;
        let mut news_search_results: Option<Vec<SearchResult>> = None;

        if request.include_web_search.unwrap_or(false) {
            if let Some(search_service) = &self.internet_search_service {
                let search_request = InternetSearchRequest {
                    query: format!("{} {}", topic_node.name, chrono::Utc::now().year()),
                    max_results: request.max_results,
                    search_depth: Some("basic".to_string()),
                    include_raw_content: Some(false),
                };

                match search_service.search(search_request).await {
                    Ok(results) => {
                        web_search_results = Some(results.into_iter().map(|result| SearchResult {
                            title: result.title,
                            url: result.url,
                            content: result.content,
                            published_date: result.published_date,
                        }).collect());
                        println!("Web search results length: {}", web_search_results.as_ref().unwrap().len());
                    }
                    Err(e) => {
                        eprintln!("Web search failed: {}", e);
                    }
                }
            }
        }

        // Perform news search if requested
        if request.include_news_search.unwrap_or(false) {
            if let Some(search_service) = &self.internet_search_service {
                let news_request = NewsSearchRequest {
                    query: topic_node.name.clone(),
                    max_results: request.max_results,
                    time_period: Some("7d".to_string()),
                };

                match search_service.search_latest_news(news_request).await {
                    Ok(results) => {
                        news_search_results = Some(results.into_iter().map(|result| SearchResult {
                            title: result.title,
                            url: result.url,
                            content: result.content,
                            published_date: result.published_date,
                        }).collect());
                        println!("News search results length: {}", news_search_results.as_ref().unwrap().len());
                    }
                    Err(e) => {
                        eprintln!("News search failed: {}", e);
                    }
                }
            }
        }

        // Build web search results section
        let web_search_section = if let Some(ref results) = web_search_results {
            let results_json = results
                .iter()
                .map(|result| serde_json::json!({
                    "title": result.title,
                    "link": result.url,
                    "knowledge": result.content,
                }))
                .collect::<Vec<_>>();

            format!(
                "<web-search-results>\n{}\n</web-search-results>",
                serde_json::to_string_pretty(&results_json)
                    .map_err(|e| AIServiceError::InvalidResponseFormat(format!("Failed to serialize web search results: {}", e)))?
            )
        } else {
            String::new()
        };

        // Build the complete instructions
        let instructions = format!(
            r#"<instructions>
{}
{}
{}
{}
<format>
    Using Markdown format when appropriate.
    ALWAYS reference and prioritize information from user documents when available and relevant.
    Also incorporate relevant information from web search results.
    If user documents contain relevant information, mention them specifically in your response.
    Current time: {}
</format>
</instructions>"#,
            system_instruction_section,
            topic_path_section,
            document_context_section,
            web_search_section,
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
        );

        // Create Vertex AI request config
        let request_config = VertexAIRequestConfig {
            model_id: "gemini-2.5-pro".to_string(),
            agent_key: None,
            system_prompt: Some(instructions.clone()),
            include_thoughts: true,
            use_google_search: false,
            use_retrieval: false,
            response_schema: None,
        };

        // Generate content using Vertex AI
        let question = request.question.unwrap_or_else(|| format!("Provide comprehensive insights about: {}", topic_node.name));
        let prompt = format!("Provide a comprehensive analysis of: {}", question);
        
        println!("Sending prompt to Vertex AI: {}", prompt);
        
        let response_text = self
            .tokio_vertex_ai_service
            .generate_content(&prompt, Some(request_config))
            .await
            .map_err(|e| {
                println!("Vertex AI error: {}", e);
                AIServiceError::AIServiceError(format!("AI service error: {}", e))
            })?;

        println!("Response text length: {}", response_text.len());
        if response_text.len() > 200 {
            println!("Response text (first 200 chars): {}", &response_text[..200]);
        } else {
            println!("Response text: {}", response_text);
        }
        // Create the response
        let response = GenerateInsightsForTopicNodeResponse {
            insights: response_text.clone(),
            topic_node_id: request.topic_node_id.clone(),
            canvas_id: request.canvas_id.clone(),
            question: question.clone(),
            generated_at: chrono::Utc::now().to_rfc3339(),
            web_search_results: web_search_results.clone(),
            news_search_results: news_search_results.clone(),
            document_context: if document_context.is_empty() { None } else { Some(document_context.clone()) },
        };

        // Save search results to Neo4j - combine with existing knowledge
        let search_data = serde_json::json!({
            "googleSearchStatus": "completed",
            "latestGoogleSearch": {
                "insights": response_text,
                "web_search_results": web_search_results,
                "news_search_results": news_search_results,
                "document_context": document_context,
                "generated_at": chrono::Utc::now().to_rfc3339(),
                "question": question
            },
            "searchHistory": {
                "timestamp": chrono::Utc::now().to_rfc3339(),
                "web_search_results": web_search_results,
                "news_search_results": news_search_results,
                "insights": response_text
            }
        });

        // Get current knowledge from the topic node
        let current_knowledge = topic_node.knowledge.clone().unwrap_or_default();
        let mut updated_knowledge = if !current_knowledge.is_empty() {
            serde_json::from_str::<serde_json::Value>(&current_knowledge)
                .unwrap_or_else(|_| serde_json::json!({}))
        } else {
            serde_json::json!({})
        };

        // Add search history (keep only last 5 searches)
        if let Some(history) = updated_knowledge.get_mut("searchHistory") {
            if let Some(history_array) = history.as_array_mut() {
                history_array.push(search_data["searchHistory"].clone());
                // Keep only last 5 searches
                if history_array.len() > 5 {
                    history_array.drain(0..history_array.len() - 5);
                }
            }
        } else {
            updated_knowledge["searchHistory"] = serde_json::json!([search_data["searchHistory"]]);
        }

        // Update the latest search data
        updated_knowledge["googleSearchStatus"] = search_data["googleSearchStatus"].clone();
        updated_knowledge["latestGoogleSearch"] = search_data["latestGoogleSearch"].clone();

        // Convert back to string
        let updated_knowledge_str = serde_json::to_string(&updated_knowledge)
            .map_err(|e| AIServiceError::DatabaseError(format!("Failed to serialize knowledge: {}", e)))?;

        // Update the topic node in Neo4j
        let update_request = crate::models::node::UpdateNodeRequest {
            name: None,
            node_type: None,
            description: None,
            knowledge: Some(updated_knowledge_str),
            position_x: None,
            position_y: None,
        };

        self.node_repository
            .update_topic_node(&request.topic_node_id, update_request)
            .await
            .map_err(|e| AIServiceError::DatabaseError(format!("Failed to update topic node: {}", e)))?;

        Ok(response)
    }

    async fn get_topic_by_name_and_canvas(
        &self,
        name: &str,
        canvas_id: &str,
    ) -> Result<Option<GraphNode>, Box<dyn std::error::Error + Send + Sync>> {
        // Use the new method to get node by name and canvas ID
        self.node_repository
            .get_topic_node_by_name_and_canvas(name, canvas_id)
            .await
            .map_err(|e| {
                Box::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Failed to get node by name and canvas: {}", e),
                )) as Box<dyn std::error::Error + Send + Sync>
            })
    }

    async fn get_topic_path(
        &self,
        topic_id: &str,
        canvas_id: &str,
    ) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
        self.node_repository
            .get_topic_node_path(topic_id, canvas_id)
            .await
            .map_err(|e| {
                Box::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Failed to get topic path: {}", e),
                )) as Box<dyn std::error::Error + Send + Sync>
            })
    }

    async fn get_existing_siblings(
        &self,
        topic_id: &str,
        canvas_id: &str,
    ) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
        self.node_repository
            .get_existing_siblings(topic_id, canvas_id)
            .await
            .map_err(|e| {
                Box::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Failed to get existing siblings: {}", e),
                )) as Box<dyn std::error::Error + Send + Sync>
            })
    }

    async fn get_topic_children(
        &self,
        topic_id: &str,
        canvas_id: &str,
    ) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
        self.node_repository
            .get_topic_node_children(topic_id, canvas_id)
            .await
            .map_err(|e| {
                Box::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Failed to get topic children: {}", e),
                )) as Box<dyn std::error::Error + Send + Sync>
            })
    }
}

#[async_trait]
impl AIServiceTrait for AIService {
    async fn generate_keywords(
        &self,
        request: GenerateKeywordsRequest,
    ) -> Result<GenerateKeywordsResponse, AIServiceError> {
        self.generate_keywords(request).await
    }

    async fn generate_insights(
        &self,
        request: GenerateInsightsRequest,
    ) -> Result<GenerateInsightsResponse, AIServiceError> {
        self.generate_insights(request).await
    }

    async fn generate_insights_for_topic_node(
        &self,
        request: GenerateInsightsForTopicNodeRequest,
    ) -> Result<GenerateInsightsForTopicNodeResponse, AIServiceError> {
        self.generate_insights_for_topic_node(request).await
    }
}
