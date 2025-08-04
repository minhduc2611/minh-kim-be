use crate::dao::canvas_dao_trait::{CanvasRepository, CanvasRepositoryError};
use crate::database::Database;
use crate::models::canvas::{Canvas, GetCanvasesRequest, InsertCanvas, UpdateCanvasRequest, GraphNode, GraphEdge};
use crate::models::common::PaginatedResponse;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use neo4rs::query;
use std::collections::HashMap;

pub struct CanvasDao {
    database: Database,
}

impl CanvasDao {
    pub fn new(database: Database) -> Self {
        Self { database }
    }
}

#[async_trait]
impl CanvasRepository for CanvasDao {
    async fn create_canvas(
        &self,
        insert_canvas: InsertCanvas,
    ) -> Result<Canvas, CanvasRepositoryError> {
        let graph = self.database.get_graph();

        let cypher = query(
            "CREATE (c:Canvas {
                id: $id,
                authorId: $author_id,
                name: $name,
                systemInstruction: $system_instruction,
                createdAt: datetime(),
                updatedAt: datetime()
            })
            RETURN c",
        )
        .param("id", insert_canvas.id.clone())
        .param("author_id", insert_canvas.author_id.clone())
        .param("name", insert_canvas.name.clone())
        .param(
            "system_instruction",
            insert_canvas.system_instruction.clone(),
        );

        let mut result = graph
            .execute(cypher)
            .await
            .map_err(|e| CanvasRepositoryError::DatabaseError(e.to_string()))?;

        if let Some(row) = result
            .next()
            .await
            .map_err(|e| CanvasRepositoryError::DatabaseError(e.to_string()))?
        {
            let node = row
                .get::<neo4rs::Node>("c")
                .map_err(|e| CanvasRepositoryError::InvalidData(e.to_string()))?;

            Self::node_to_canvas(node)
        } else {
            Err(CanvasRepositoryError::DatabaseError(
                "Failed to create canvas".to_string(),
            ))
        }
    }

    async fn get_canvas_by_id(&self, id: &str) -> Result<Option<Canvas>, CanvasRepositoryError> {
        // Get the Neo4j graph database connection from the database instance
        let graph = self.database.get_graph();

        // Create a Cypher query to find a Canvas node with the specified id
        // The query matches a Canvas node where the id property equals the provided parameter
        let cypher = query("MATCH (c:Canvas {id: $id}) RETURN c")
            // Bind the id parameter to the query to prevent SQL injection
            .param("id", id);

        // Execute the Cypher query asynchronously against the Neo4j database
        // Convert any execution errors to CanvasRepositoryError::DatabaseError
        let mut result = graph
            .execute(cypher)
            .await
            .map_err(|e| CanvasRepositoryError::DatabaseError(e.to_string()))?;

        // Check if the query returned any results by getting the next row
        // Convert any iteration errors to CanvasRepositoryError::DatabaseError
        if let Some(row) = result
            .next()
            .await
            .map_err(|e| CanvasRepositoryError::DatabaseError(e.to_string()))?
        {
            // Extract the Canvas node from the result row using the alias "c"
            // Convert any extraction errors to CanvasRepositoryError::InvalidData
            let node = row
                .get::<neo4rs::Node>("c")
                .map_err(|e| CanvasRepositoryError::InvalidData(e.to_string()))?;

            // Convert the Neo4j node to a Canvas struct and wrap in Some
            // The ? operator propagates any conversion errors from node_to_canvas
            Ok(Some(Self::node_to_canvas(node)?))
        } else {
            // No canvas found with the given id, return None wrapped in Ok
            Ok(None)
        }
    }

    async fn get_canvases(
        &self,
        request: GetCanvasesRequest,
    ) -> Result<PaginatedResponse<Canvas>, CanvasRepositoryError> {
        let graph = self.database.get_graph();

        // Set default values for pagination
        let limit = request.limit.unwrap_or(50); // Default limit of 50
        let offset = request.offset.unwrap_or(0); // Default offset of 0

        // First query: Get total count
        let count_cypher = query(
            "MATCH (c:Canvas {authorId: $author_id})
            RETURN count(c) as total",
        )
        .param("author_id", request.author_id.clone());

        let mut count_result = graph
            .execute(count_cypher)
            .await
            .map_err(|e| CanvasRepositoryError::DatabaseError(e.to_string()))?;

        let total = if let Some(row) = count_result
            .next()
            .await
            .map_err(|e| CanvasRepositoryError::DatabaseError(e.to_string()))?
        {
            row.get::<i64>("total")
                .map_err(|e| CanvasRepositoryError::InvalidData(e.to_string()))?
        } else {
            0
        };

        // Second query: Get paginated data
        let data_cypher = query(
            "MATCH (c:Canvas {authorId: $author_id})
            RETURN c
            ORDER BY c.updatedAt DESC
            SKIP $offset
            LIMIT $limit",
        )
        .param("author_id", request.author_id)
        .param("offset", offset)
        .param("limit", limit);

        let mut data_result = graph
            .execute(data_cypher)
            .await
            .map_err(|e| CanvasRepositoryError::DatabaseError(e.to_string()))?;

        let mut canvases = Vec::new();
        while let Some(row) = data_result
            .next()
            .await
            .map_err(|e| CanvasRepositoryError::DatabaseError(e.to_string()))?
        {
            let node = row
                .get::<neo4rs::Node>("c")
                .map_err(|e| CanvasRepositoryError::InvalidData(e.to_string()))?;

            canvases.push(Self::node_to_canvas(node)?);
        }

        Ok(PaginatedResponse::new(canvases, total, limit, offset))
    }

    async fn update_canvas(
        &self,
        id: &str,
        updates: UpdateCanvasRequest,
    ) -> Result<Option<Canvas>, CanvasRepositoryError> {
        let graph = self.database.get_graph();

        let mut set_clauses = Vec::new();
        let mut params: HashMap<String, neo4rs::BoltType> = HashMap::new();
        params.insert("id".to_string(), id.into());

        if let Some(name) = &updates.name {
            set_clauses.push("c.name = $name");
            params.insert("name".to_string(), name.clone().into());
        }

        if let Some(system_instruction) = &updates.system_instruction {
            set_clauses.push("c.systemInstruction = $system_instruction");
            params.insert(
                "system_instruction".to_string(),
                system_instruction.clone().into(),
            );
        }

        if set_clauses.is_empty() {
            return self.get_canvas_by_id(id).await;
        }

        set_clauses.push("c.updatedAt = datetime()");

        let cypher_str = format!(
            "MATCH (c:Canvas {{id: $id}})
            SET {}
            RETURN c",
            set_clauses.join(", ")
        );

        let mut cypher = query(&cypher_str);
        for (key, value) in params {
            cypher = cypher.param(&key, value);
        }

        let mut result = graph
            .execute(cypher)
            .await
            .map_err(|e| CanvasRepositoryError::DatabaseError(e.to_string()))?;

        if let Some(row) = result
            .next()
            .await
            .map_err(|e| CanvasRepositoryError::DatabaseError(e.to_string()))?
        {
            let node = row
                .get::<neo4rs::Node>("c")
                .map_err(|e| CanvasRepositoryError::InvalidData(e.to_string()))?;

            Ok(Some(Self::node_to_canvas(node)?))
        } else {
            Ok(None)
        }
    }

    async fn delete_canvas(&self, id: &str) -> Result<(), CanvasRepositoryError> {
        let graph = self.database.get_graph();

        // Use a single query that both deletes and returns the count of deleted nodes
        // This approach is atomic and avoids race conditions
        let cypher = query(
            "MATCH (c:Canvas {id: $id}) 
             DETACH DELETE c 
             RETURN count(c) as deleted_count",
        )
        .param("id", id);

        let mut result = graph
            .execute(cypher)
            .await
            .map_err(|e| CanvasRepositoryError::DatabaseError(e.to_string()))?;

        // Check the result to verify deletion
        if let Some(row) = result
            .next()
            .await
            .map_err(|e| CanvasRepositoryError::DatabaseError(e.to_string()))?
        {
            let deleted_count = row
                .get::<i64>("deleted_count")
                .map_err(|e| CanvasRepositoryError::InvalidData(e.to_string()))?;

            if deleted_count == 0 {
                return Err(CanvasRepositoryError::NotFound);
            }

            Ok(())
        } else {
            Err(CanvasRepositoryError::DatabaseError(
                "Failed to execute delete query".to_string(),
            ))
        }
    }

    async fn get_topics_by_canvas(&self, canvas_id: &str) -> Result<Vec<GraphNode>, CanvasRepositoryError> {
        let graph = self.database.get_graph();

        // Query to get all Topic nodes that belong to the specified canvas
        let cypher = query(
            "MATCH (c:Canvas {id: $canvas_id})-[:BELONGS_TO]->(t:Topic)
             RETURN t",
        )
        .param("canvas_id", canvas_id);

        let mut result = graph
            .execute(cypher)
            .await
            .map_err(|e| CanvasRepositoryError::DatabaseError(e.to_string()))?;

        let mut topics = Vec::new();
        while let Some(row) = result
            .next()
            .await
            .map_err(|e| CanvasRepositoryError::DatabaseError(e.to_string()))?
        {
            let node = row
                .get::<neo4rs::Node>("t")
                .map_err(|e| CanvasRepositoryError::InvalidData(e.to_string()))?;

            topics.push(Self::node_to_graph_node(node)?);
        }

        Ok(topics)
    }

    async fn get_relationships_by_canvas(&self, canvas_id: &str) -> Result<Vec<GraphEdge>, CanvasRepositoryError> {
        let graph = self.database.get_graph();

        // Query to get all relationships between topics that belong to the specified canvas
        // Instead of trying to get the relationship object, we'll get the relationship properties
        let cypher = query(
            "MATCH (c:Canvas {id: $canvas_id})-[:BELONGS_TO]->(t1:Topic)-[r:RELATES_TO]->(t2:Topic)<-[:BELONGS_TO]-(c)
             RETURN t1.id as source_id, t2.id as target_id, r.id as relationship_id",
        )
        .param("canvas_id", canvas_id);

        let mut result = graph
            .execute(cypher)
            .await
            .map_err(|e| CanvasRepositoryError::DatabaseError(e.to_string()))?;

        let mut relationships = Vec::new();
        while let Some(row) = result
            .next()
            .await
            .map_err(|e| CanvasRepositoryError::DatabaseError(e.to_string()))?
        {
            let source_id = row
                .get::<String>("source_id")
                .map_err(|e| CanvasRepositoryError::InvalidData(e.to_string()))?;

            let target_id = row
                .get::<String>("target_id")
                .map_err(|e| CanvasRepositoryError::InvalidData(e.to_string()))?;

            let relationship_id = row
                .get::<String>("relationship_id")
                .unwrap_or_else(|_| format!("{}-{}", source_id, target_id));

            relationships.push(GraphEdge {
                id: relationship_id,
                source: source_id,
                target: target_id,
            });
        }

        Ok(relationships)
    }
}

impl CanvasDao {
    fn node_to_canvas(node: neo4rs::Node) -> Result<Canvas, CanvasRepositoryError> {
        let id = node
            .get::<String>("id")
            .map_err(|e| CanvasRepositoryError::InvalidData(format!("id: {}", e)))?;

        let author_id = node
            .get::<String>("authorId")
            .map_err(|e| CanvasRepositoryError::InvalidData(format!("authorId: {}", e)))?;

        let name = node
            .get::<String>("name")
            .map_err(|e| CanvasRepositoryError::InvalidData(format!("name: {}", e)))?;

        let system_instruction = node.get::<String>("systemInstruction").unwrap_or_default();

        let created_at_raw = node
            .get::<String>("createdAt")
            .map_err(|e| CanvasRepositoryError::InvalidData(format!("createdAt: {}", e)))?;

        let updated_at_raw = node
            .get::<String>("updatedAt")
            .map_err(|e| CanvasRepositoryError::InvalidData(format!("updatedAt: {}", e)))?;

        Ok(Canvas {
            id,
            author_id,
            name,
            system_instruction,
            created_at: Self::parse_neo4j_datetime(&created_at_raw)?,
            updated_at: Self::parse_neo4j_datetime(&updated_at_raw)?,
        })
    }

    fn node_to_graph_node(node: neo4rs::Node) -> Result<GraphNode, CanvasRepositoryError> {
        let id = node
            .get::<String>("id")
            .map_err(|e| CanvasRepositoryError::InvalidData(format!("id: {}", e)))?;

        let name = node
            .get::<String>("name")
            .map_err(|e| CanvasRepositoryError::InvalidData(format!("name: {}", e)))?;

        let node_type = node
            .get::<String>("type")
            .unwrap_or_else(|_| "original".to_string());

        let description = node.get::<String>("description").ok();

        let knowledge = node.get::<String>("knowledge").ok();

        let position_x = node
            .get::<f64>("positionX")
            .unwrap_or(0.0);

        let position_y = node
            .get::<f64>("positionY")
            .unwrap_or(0.0);

        Ok(GraphNode {
            id,
            name,
            node_type,
            description,
            knowledge,
            position_x,
            position_y,
        })
    }

    fn parse_neo4j_datetime(datetime_str: &str) -> Result<DateTime<Utc>, CanvasRepositoryError> {
        // Neo4j datetime() returns ISO 8601 format that chrono can parse
        datetime_str.parse::<DateTime<Utc>>().map_err(|e| {
            CanvasRepositoryError::InvalidData(format!("Failed to parse datetime: {}", e))
        })
    }
}
