use neo4rs::{Graph, ConfigBuilder};
use std::env;

#[derive(Clone)]
pub struct Database {
    pub graph: Graph,
}

impl Database {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // Load environment variables
        dotenv::dotenv().ok();

        let uri = env::var("NEO4J_URI").unwrap_or_else(|_| "bolt://localhost:7687".to_string());
        let user = env::var("NEO4J_USER").unwrap_or_else(|_| "neo4j".to_string());
        let password = env::var("NEO4J_PASSWORD").unwrap_or_else(|_| "password".to_string());

        // Create Neo4j connection
        let config = ConfigBuilder::default()
            .uri(&uri)
            .user(&user)
            .password(&password)
            .db("neo4j")
            .build()?;

        let graph = Graph::connect(config).await?;

        Ok(Database { graph })
    }

    pub fn get_graph(&self) -> &Graph {
        &self.graph
    }
}

pub async fn init_database() -> Result<Database, Box<dyn std::error::Error>> {
    Database::new().await
} 