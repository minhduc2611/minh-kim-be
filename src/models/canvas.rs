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
