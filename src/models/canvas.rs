use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Canvas {
    pub id: String,
    pub author_id: String,
    pub name: String,
    pub system_instruction: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: String,
    pub name: String,
    pub node_type: String, // "original" or "generated"
    pub description: Option<String>,
    pub knowledge: Option<String>,
    pub position_x: Option<f64>,
    pub position_y: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    pub id: String,
    pub source: String,
    pub target: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphData {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
}

#[derive(Debug, Deserialize)]
pub struct CreateCanvasRequest {
    pub name: String,
    pub author_id: String,
    pub system_instruction: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCanvasRequest {
    pub name: Option<String>,
    pub system_instruction: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GetCanvasesRequest {
    pub author_id: String,
    pub limit: Option<i32>,
    pub offset: Option<i32>,
}

#[derive(Debug)]
pub struct InsertCanvas {
    pub id: String,
    pub author_id: String,
    pub name: String,
    pub system_instruction: String,
}

impl From<CreateCanvasRequest> for InsertCanvas {
    fn from(req: CreateCanvasRequest) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            author_id: req.author_id,
            name: req.name,
            system_instruction: req.system_instruction.unwrap_or_default(),
        }
    }
}
