use crate::dao::node_dao_trait::{NodeRepository, NodeRepositoryError};
use crate::database::Database;
use crate::models::node::{GetNodesRequest, InsertNode, UpdateNodeRequest, InsertRelationship, Relationship};
use crate::models::canvas::GraphNode;
use crate::models::common::PaginatedResponse;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use neo4rs::query;
use std::collections::HashMap;

pub struct NodeDao {
    database: Database,
}

impl NodeDao {
    pub fn new(database: Database) -> Self {
        Self { database }
    }
}

#[async_trait]
impl NodeRepository for NodeDao {
    async fn create_topic_node(
        &self,
        insert_node: InsertNode,
    ) -> Result<GraphNode, NodeRepositoryError> {
        let graph = self.database.get_graph();

        let cypher = query(
            "MATCH (c:Canvas {id: $canvas_id})
             CREATE (n:Topic {
                 id: $id,
                 canvasId: $canvas_id,
                 name: $name,
                 type: $type,
                 description: $description,
                 knowledge: $knowledge,
                 positionX: $position_x,
                 positionY: $position_y,
                 createdAt: datetime()
             })
             CREATE (c)-[:CONTAINS]->(n)
             RETURN n",
        )
        .param("id", insert_node.id.clone())
        .param("canvas_id", insert_node.canvas_id.clone())
        .param("name", insert_node.name.clone())
        .param("type", insert_node.node_type.clone())
        .param("description", insert_node.description.clone().unwrap_or_default())
        .param("knowledge", insert_node.knowledge.clone().unwrap_or_default())
        .param("position_x", insert_node.position_x.unwrap_or(0.0))
        .param("position_y", insert_node.position_y.unwrap_or(0.0));

        let mut result = graph
            .execute(cypher)
            .await
            .map_err(|e| NodeRepositoryError::DatabaseError(e.to_string()))?;

        if let Some(row) = result
            .next()
            .await
            .map_err(|e| NodeRepositoryError::DatabaseError(e.to_string()))?
        {
            let node = row
                .get::<neo4rs::Node>("n")
                .map_err(|e| NodeRepositoryError::InvalidData(e.to_string()))?;

            Self::node_to_graph_node(node)
        } else {
            Err(NodeRepositoryError::DatabaseError(
                "Failed to create node".to_string(),
            ))
        }
    }

    async fn get_topic_node_by_id(&self, id: &str) -> Result<Option<GraphNode>, NodeRepositoryError> {
        let graph = self.database.get_graph();

        let cypher = query("MATCH (n:Topic {id: $id}) RETURN n")
            .param("id", id);

        let mut result = graph
            .execute(cypher)
            .await
            .map_err(|e| NodeRepositoryError::DatabaseError(e.to_string()))?;

        if let Some(row) = result
            .next()
            .await
            .map_err(|e| NodeRepositoryError::DatabaseError(e.to_string()))?
        {
            let node = row
                .get::<neo4rs::Node>("n")
                .map_err(|e| NodeRepositoryError::InvalidData(e.to_string()))?;

            Ok(Some(Self::node_to_graph_node(node)?))
        } else {
            Ok(None)
        }
    }

    async fn get_topic_nodes(
        &self,
        request: GetNodesRequest,
    ) -> Result<PaginatedResponse<GraphNode>, NodeRepositoryError> {
        let graph = self.database.get_graph();

        let limit = request.limit.unwrap_or(50);
        let offset = request.offset.unwrap_or(0);

        // First query: Get total count
        let count_cypher = query(
            "MATCH (n:Topic {canvasId: $canvas_id})
             RETURN count(n) as total",
        )
        .param("canvas_id", request.canvas_id.clone());

        let mut count_result = graph
            .execute(count_cypher)
            .await
            .map_err(|e| NodeRepositoryError::DatabaseError(e.to_string()))?;

        let total = if let Some(row) = count_result
            .next()
            .await
            .map_err(|e| NodeRepositoryError::DatabaseError(e.to_string()))?
        {
            row.get::<i64>("total")
                .map_err(|e| NodeRepositoryError::InvalidData(e.to_string()))?
        } else {
            0
        };

        // Second query: Get paginated data
        let data_cypher = query(
            "MATCH (n:Topic {canvasId: $canvas_id})
             RETURN n
             ORDER BY n.createdAt ASC
             SKIP $offset
             LIMIT $limit",
        )
        .param("canvas_id", request.canvas_id)
        .param("offset", offset)
        .param("limit", limit);

        let mut data_result = graph
            .execute(data_cypher)
            .await
            .map_err(|e| NodeRepositoryError::DatabaseError(e.to_string()))?;

        let mut nodes = Vec::new();
        while let Some(row) = data_result
            .next()
            .await
            .map_err(|e| NodeRepositoryError::DatabaseError(e.to_string()))?
        {
            let node = row
                .get::<neo4rs::Node>("n")
                .map_err(|e| NodeRepositoryError::InvalidData(e.to_string()))?;

            nodes.push(Self::node_to_graph_node(node)?);
        }

        Ok(PaginatedResponse::new(nodes, total, limit, offset))
    }

    async fn get_topic_nodes_by_canvas(&self, canvas_id: &str) -> Result<Vec<GraphNode>, NodeRepositoryError> {
        let graph = self.database.get_graph();

        let cypher = query(
            "MATCH (n:Topic {canvasId: $canvas_id})
             RETURN n
             ORDER BY n.createdAt ASC",
        )
        .param("canvas_id", canvas_id);

        let mut result = graph
            .execute(cypher)
            .await
            .map_err(|e| NodeRepositoryError::DatabaseError(e.to_string()))?;

        let mut nodes = Vec::new();
        while let Some(row) = result
            .next()
            .await
            .map_err(|e| NodeRepositoryError::DatabaseError(e.to_string()))?
        {
            let node = row
                .get::<neo4rs::Node>("n")
                .map_err(|e| NodeRepositoryError::InvalidData(e.to_string()))?;

            nodes.push(Self::node_to_graph_node(node)?);
        }

        Ok(nodes)
    }

    async fn update_topic_node(
        &self,
        id: &str,
        updates: UpdateNodeRequest,
    ) -> Result<Option<GraphNode>, NodeRepositoryError> {
        let graph = self.database.get_graph();

        let mut set_clauses = Vec::new();
        let mut params: HashMap<String, neo4rs::BoltType> = HashMap::new();
        params.insert("id".to_string(), id.into());

        if let Some(name) = &updates.name {
            set_clauses.push("n.name = $name");
            params.insert("name".to_string(), name.clone().into());
        }

        if let Some(node_type) = &updates.node_type {
            set_clauses.push("n.type = $type");
            params.insert("type".to_string(), node_type.clone().into());
        }

        if let Some(description) = &updates.description {
            set_clauses.push("n.description = $description");
            params.insert("description".to_string(), description.clone().into());
        }

        if let Some(knowledge) = &updates.knowledge {
            set_clauses.push("n.knowledge = $knowledge");
            params.insert("knowledge".to_string(), knowledge.clone().into());
        }

        if let Some(position_x) = updates.position_x {
            set_clauses.push("n.positionX = $position_x");
            params.insert("position_x".to_string(), position_x.into());
        }

        if let Some(position_y) = updates.position_y {
            set_clauses.push("n.positionY = $position_y");
            params.insert("position_y".to_string(), position_y.into());
        }

        if set_clauses.is_empty() {
            return self.get_topic_node_by_id(id).await;
        }

        let cypher_str = format!(
            "MATCH (n:Topic {{id: $id}})
             SET {}
             RETURN n",
            set_clauses.join(", ")
        );

        let mut cypher = query(&cypher_str);
        for (key, value) in params {
            cypher = cypher.param(&key, value);
        }

        let mut result = graph
            .execute(cypher)
            .await
            .map_err(|e| NodeRepositoryError::DatabaseError(e.to_string()))?;

        if let Some(row) = result
            .next()
            .await
            .map_err(|e| NodeRepositoryError::DatabaseError(e.to_string()))?
        {
            let node = row
                .get::<neo4rs::Node>("n")
                .map_err(|e| NodeRepositoryError::InvalidData(e.to_string()))?;

            Ok(Some(Self::node_to_graph_node(node)?))
        } else {
            Ok(None)
        }
    }

    async fn delete_topic_node(&self, id: &str) -> Result<(), NodeRepositoryError> {
        let graph = self.database.get_graph();

        let cypher = query(
            "MATCH (n:Topic {id: $id}) 
             DETACH DELETE n 
             RETURN count(n) as deleted_count",
        )
        .param("id", id);

        let mut result = graph
            .execute(cypher)
            .await
            .map_err(|e| NodeRepositoryError::DatabaseError(e.to_string()))?;

        if let Some(row) = result
            .next()
            .await
            .map_err(|e| NodeRepositoryError::DatabaseError(e.to_string()))?
        {
            let deleted_count = row
                .get::<i64>("deleted_count")
                .map_err(|e| NodeRepositoryError::InvalidData(e.to_string()))?;

            if deleted_count == 0 {
                return Err(NodeRepositoryError::NotFound);
            }

            Ok(())
        } else {
            Err(NodeRepositoryError::DatabaseError(
                "Failed to execute delete query".to_string(),
            ))
        }
    }

    async fn delete_topic_nodes_by_canvas(&self, canvas_id: &str) -> Result<(), NodeRepositoryError> {
        let graph = self.database.get_graph();

        let cypher = query(
            "MATCH (n:Topic {canvasId: $canvas_id}) 
             DETACH DELETE n 
             RETURN count(n) as deleted_count",
        )
        .param("canvas_id", canvas_id);

        let mut result = graph
            .execute(cypher)
            .await
            .map_err(|e| NodeRepositoryError::DatabaseError(e.to_string()))?;

        if let Some(row) = result
            .next()
            .await
            .map_err(|e| NodeRepositoryError::DatabaseError(e.to_string()))?
        {
            let _deleted_count = row
                .get::<i64>("deleted_count")
                .map_err(|e| NodeRepositoryError::InvalidData(e.to_string()))?;

            Ok(())
        } else {
            Err(NodeRepositoryError::DatabaseError(
                "Failed to execute delete query".to_string(),
            ))
        }
    }

    async fn get_topic_node_by_name_and_canvas(
        &self,
        name: &str,
        canvas_id: &str,
    ) -> Result<Option<GraphNode>, NodeRepositoryError> {
        let graph = self.database.get_graph();

        let cypher = query("MATCH (n:Topic {name: $name, canvasId: $canvas_id}) RETURN n")
            .param("name", name)
            .param("canvas_id", canvas_id);
        println!("cypher: MATCH (n:Topic {{name: \"{}\", canvasId: \"{}\"}}) RETURN n", name, canvas_id);
        let mut result = graph
            .execute(cypher)
            .await
            .map_err(|e| NodeRepositoryError::DatabaseError(e.to_string()))?;

        if let Some(row) = result
            .next()
            .await
            .map_err(|e| NodeRepositoryError::DatabaseError(e.to_string()))?
        {
            let node = row
                .get::<neo4rs::Node>("n")
                .map_err(|e| NodeRepositoryError::InvalidData(e.to_string()))?;

            let graph_node = Self::node_to_graph_node(node)?;
            Ok(Some(graph_node))
        } else {
            Ok(None)
        }
    }

    async fn get_topic_node_path(
        &self,
        topic_id: &str,
        canvas_id: &str,
    ) -> Result<Vec<String>, NodeRepositoryError> {
        let graph = self.database.get_graph();

        let cypher = query(
            "MATCH path = (root:Topic)-[:RELATED_TO*0..]->(target:Topic {id: $topic_id})
             WHERE NOT ()-[:RELATED_TO]->(root:Topic {canvasId: $canvas_id})
             AND root.canvasId = $canvas_id
             RETURN [node IN nodes(path) | node.name] AS pathNames
             ORDER BY length(path) ASC
             LIMIT 1",
        )
        .param("topic_id", topic_id)
        .param("canvas_id", canvas_id);

        let mut result = graph
            .execute(cypher)
            .await
            .map_err(|e| NodeRepositoryError::DatabaseError(e.to_string()))?;

        if let Some(row) = result
            .next()
            .await
            .map_err(|e| NodeRepositoryError::DatabaseError(e.to_string()))?
        {
            let path_names = row
                .get::<Vec<String>>("pathNames")
                .map_err(|e| NodeRepositoryError::InvalidData(e.to_string()))?;

            Ok(path_names)
        } else {
            Ok(Vec::new())
        }
    }

    async fn get_existing_siblings(
        &self,
        topic_id: &str,
        canvas_id: &str,
    ) -> Result<Vec<String>, NodeRepositoryError> {
        let graph = self.database.get_graph();

        let cypher = query(
            "MATCH (parent:Topic)-[:RELATED_TO]->(current:Topic {id: $topic_id, canvasId: $canvas_id})
             MATCH (parent)-[:RELATED_TO]->(sibling:Topic)
             WHERE sibling.id <> $topic_id
             RETURN COLLECT(sibling.name) AS siblings",
        )
        .param("topic_id", topic_id)
        .param("canvas_id", canvas_id);

        let mut result = graph
            .execute(cypher)
            .await
            .map_err(|e| NodeRepositoryError::DatabaseError(e.to_string()))?;

        if let Some(row) = result
            .next()
            .await
            .map_err(|e| NodeRepositoryError::DatabaseError(e.to_string()))?
        {
            let siblings = row
                .get::<Vec<String>>("siblings")
                .map_err(|e| NodeRepositoryError::InvalidData(e.to_string()))?;

            Ok(siblings)
        } else {
            Ok(Vec::new())
        }
    }

    async fn get_topic_node_children(
        &self,
        topic_id: &str,
        canvas_id: &str,
    ) -> Result<Vec<String>, NodeRepositoryError> {
        let graph = self.database.get_graph();

        let cypher = query(
            "MATCH (current:Topic {id: $topic_id, canvasId: $canvas_id})-[:RELATED_TO]->(child:Topic)
             RETURN COLLECT(child.name) AS children",
        )
        .param("topic_id", topic_id)
        .param("canvas_id", canvas_id);

        let mut result = graph
            .execute(cypher)
            .await
            .map_err(|e| NodeRepositoryError::DatabaseError(e.to_string()))?;

        if let Some(row) = result
            .next()
            .await
            .map_err(|e| NodeRepositoryError::DatabaseError(e.to_string()))?
        {
            let children = row
                .get::<Vec<String>>("children")
                .map_err(|e| NodeRepositoryError::InvalidData(e.to_string()))?;

            Ok(children)
        } else {
            Ok(Vec::new())
        }
    }


    async fn relationship_exists(
        &self,
        source_id: &str,
        target_id: &str,
    ) -> Result<bool, NodeRepositoryError> {
        let graph = self.database.get_graph();

        let cypher = query(
            "MATCH (s:Topic {id: $source_id})-[r:RELATED_TO]->(t:Topic {id: $target_id})
             RETURN COUNT(r) > 0 AS exists",
        )
        .param("source_id", source_id)
        .param("target_id", target_id);

        let mut result = graph
            .execute(cypher)
            .await
            .map_err(|e| NodeRepositoryError::DatabaseError(e.to_string()))?;

        if let Some(row) = result
            .next()
            .await
            .map_err(|e| NodeRepositoryError::DatabaseError(e.to_string()))?
        {
            let exists = row
                .get::<bool>("exists")
                .map_err(|e| NodeRepositoryError::InvalidData(e.to_string()))?;

            Ok(exists)
        } else {
            Ok(false)
        }
    }

    async fn create_relationship(
        &self,
        insert_relationship: InsertRelationship,
    ) -> Result<Relationship, NodeRepositoryError> {
        let graph = self.database.get_graph();

        let cypher = query(
            "MATCH (source:Topic {id: $source_id})
            MATCH (target:Topic {id: $target_id})
            CREATE (source)-[r:RELATED_TO {
            id: $id,
            canvasId: $canvas_id,
            sourceId: $source_id,
            targetId: $target_id,
            createdAt: datetime()
            }]->(target)
            RETURN r",
        )
        .param("id", insert_relationship.id.clone())
        .param("canvas_id", insert_relationship.canvas_id.clone())
        .param("source_id", insert_relationship.source_id.clone())
        .param("target_id", insert_relationship.target_id.clone());

        let mut result = graph
            .execute(cypher)
            .await
            .map_err(|e| NodeRepositoryError::DatabaseError(e.to_string()))?;

        if let Some(row) = result
            .next()
            .await
            .map_err(|e| NodeRepositoryError::DatabaseError(e.to_string()))?
        {
            // Get the relation from the row
            let relation = row
                .get::<neo4rs::Relation>("r")
                .map_err(|e| NodeRepositoryError::InvalidData(format!("relation: {}", e)))?;
            
            // Extract properties from the relation
            let id = relation
                .get::<String>("id")
                .map_err(|e| NodeRepositoryError::InvalidData(format!("id: {}", e)))?;

            let canvas_id = relation
                .get::<String>("canvasId")
                .map_err(|e| NodeRepositoryError::InvalidData(format!("canvasId: {}", e)))?;

            let source_id = relation
                .get::<String>("sourceId")
                .map_err(|e| NodeRepositoryError::InvalidData(format!("sourceId: {}", e)))?;

            let target_id = relation
                .get::<String>("targetId")
                .map_err(|e| NodeRepositoryError::InvalidData(format!("targetId: {}", e)))?;

            let created_at = relation
                .get::<chrono::DateTime<chrono::Utc>>("createdAt")
                .map_err(|e| NodeRepositoryError::InvalidData(format!("createdAt: {}", e)))?;

            Ok(Relationship {
                id,
                canvas_id,
                source_id,
                target_id,
                created_at,
            })
        } else {
            Err(NodeRepositoryError::DatabaseError(
                "Failed to create relationship".to_string(),
            ))
        }
    }
}

impl NodeDao {
    fn node_to_graph_node(node: neo4rs::Node) -> Result<GraphNode, NodeRepositoryError> {
        let id = node
            .get::<String>("id")
            .map_err(|e| NodeRepositoryError::InvalidData(format!("id: {}", e)))?;

        let name = node
            .get::<String>("name")
            .map_err(|e| NodeRepositoryError::InvalidData(format!("name: {}", e)))?;

        let node_type = node
            .get::<String>("type")
            .unwrap_or_else(|_| "original".to_string());

        let description = node.get::<String>("description").ok();
        let knowledge = node.get::<String>("knowledge").ok();
        let position_x = node.get::<f64>("positionX").ok();
        let position_y = node.get::<f64>("positionY").ok();

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


} 