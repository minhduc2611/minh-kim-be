use serde::{Deserialize};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct CreateNodeRequest {
    pub name: String,
    pub canvas_id: String,
    pub node_type: Option<String>,
    pub description: Option<String>,
    pub knowledge: Option<String>,
    pub position_x: Option<f64>,
    pub position_y: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateNodeRequest {
    pub name: Option<String>,
    pub node_type: Option<String>,
    pub description: Option<String>,
    pub knowledge: Option<String>,
    pub position_x: Option<f64>,
    pub position_y: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct GetNodesRequest {
    pub canvas_id: String,
    pub limit: Option<i32>,
    pub offset: Option<i32>,
}

#[derive(Debug)]
pub struct InsertNode {
    pub id: String,
    pub canvas_id: String,
    pub name: String,
    pub node_type: String,
    pub description: Option<String>,
    pub knowledge: Option<String>,
    pub position_x: Option<f64>,
    pub position_y: Option<f64>,
}

#[derive(Debug)]
pub struct InsertRelationship {
    pub id: String,
    pub canvas_id: String,
    pub source_id: String,
    pub target_id: String,
}

#[derive(Debug, Clone)]
pub struct Relationship {
    pub id: String,
    pub canvas_id: String,
    pub source_id: String,
    pub target_id: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl From<CreateNodeRequest> for InsertNode {
    fn from(req: CreateNodeRequest) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            canvas_id: req.canvas_id,
            name: req.name,
            node_type: req.node_type.unwrap_or_else(|| "original".to_string()),
            description: req.description,
            knowledge: req.knowledge,
            position_x: req.position_x,
            position_y: req.position_y,
        }
    }
} 