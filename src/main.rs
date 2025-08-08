use actix_cors::Cors;
use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};

mod controllers;
mod dao;
mod database;
mod middleware;
mod models;
mod services;

use controllers::{ai_controller, auth_controller, canvas_controller, email_controller, node_controller};
use dao::canvas_dao::CanvasDao;
use dao::canvas_dao_trait::CanvasRepository;
use dao::node_dao::NodeDao;
use dao::node_dao_trait::NodeRepository;
use services::auth_service::AuthService;
use services::auth_service_trait::AuthServiceTrait;
use services::canvas_service::CanvasService;
use services::canvas_service_trait::CanvasServiceTrait;
use services::node_service::NodeService;
use services::node_service_trait::NodeServiceTrait;
use services::email_service::EmailService;
use services::email_service_trait::EmailConfig;
use services::email_service_trait::EmailServiceTrait;
use services::dummy_email_service::DummyEmailService;
use services::vertex_ai_service::VertexAIService;
use services::vertex_ai_service_trait::VertexAIServiceTrait;
use services::ai_service::AIService;
use services::ai_service_trait::AIServiceTrait;
use std::sync::Arc;

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::canvas::{GraphNode, GraphEdge, GraphData};

    #[test]
    fn test_graph_data_structure() {
        // Test that GraphData can be created and serialized
        let nodes = vec![
            GraphNode {
                id: "node1".to_string(),
                name: "Test Node".to_string(),
                node_type: "original".to_string(),
                description: Some("Test description".to_string()),
                knowledge: None,
                position_x: Some(100.0),
                position_y: Some(200.0),
            }
        ];

        let edges = vec![
            GraphEdge {
                id: "edge1".to_string(),
                source: "node1".to_string(),
                target: "node2".to_string(),
            }
        ];

        let graph_data = GraphData { nodes, edges };
        
        // Test serialization
        let json = serde_json::to_string(&graph_data).unwrap();
        assert!(json.contains("Test Node"));
        assert!(json.contains("node1"));
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load environment variables from .env file
    dotenv::dotenv().ok();

    let port = 8080;
    let host = "0.0.0.0";

    // Initialize Neo4j database connection
    let database = database::init_database()
        .await
        .expect("Failed to connect to Neo4j database");

    // Set up dependency injection
    let canvas_repository: Arc<dyn CanvasRepository> = Arc::new(CanvasDao::new(database.clone()));
    let canvas_service: Arc<dyn CanvasServiceTrait> =
        Arc::new(CanvasService::new(canvas_repository.clone()));

    let node_repository: Arc<dyn NodeRepository> = Arc::new(NodeDao::new(database.clone()));
    let node_service: Arc<dyn NodeServiceTrait> =
        Arc::new(NodeService::new(node_repository.clone(), canvas_repository.clone()));

    // Set up Vertex AI service
    let vertex_ai_service: Arc<dyn VertexAIServiceTrait> = Arc::new(VertexAIService::new(None));
    
    // Set up AI service for keyword generation
    let ai_service: Arc<dyn AIServiceTrait> = Arc::new(AIService::new(
        canvas_repository.clone(),
        node_repository.clone(),
        VertexAIService::new(None),
    ));

    // Set up auth service with Supabase (you can change to JWT+Weviate if needed)
    let auth_service: Arc<dyn AuthServiceTrait> = Arc::new(AuthService::with_supabase(
        services::auth_service::SupabaseConfig {
            url: std::env::var("SUPABASE_URL")
                .unwrap_or_else(|_| "https://your-project.supabase.co".to_string()),
            anon_key: std::env::var("SUPABASE_ANON_KEY")
                .unwrap_or_else(|_| "your-anon-key".to_string()),
            service_role_key: std::env::var("SUPABASE_SERVICE_ROLE_KEY")
                .unwrap_or_else(|_| "your-service-role-key".to_string()),
        },
    ));

    // Set up email service with SMTP
    let email_service: Arc<dyn EmailServiceTrait> = match EmailService::with_smtp(EmailConfig {
        smtp_server: std::env::var("SMTP_SERVER").unwrap_or_else(|_| "mail.privateemail.com".to_string()),
        smtp_port: std::env::var("SMTP_PORT").unwrap_or_else(|_| "587".to_string()).parse().unwrap_or(587),
        smtp_username: std::env::var("SMTP_USERNAME").unwrap_or_else(|_| "".to_string()),
        smtp_password: std::env::var("SMTP_PASSWORD").unwrap_or_else(|_| "".to_string()),
        from_email: std::env::var("FROM_EMAIL").unwrap_or_else(|_| std::env::var("SMTP_USERNAME").unwrap_or_else(|_| "".to_string())),
        domain_url: std::env::var("DOMAIN_URL").unwrap_or_else(|_| "http://localhost:3000".to_string()),
    }) {
        Ok(service) => Arc::new(service),
        Err(_) => {
            eprintln!("Warning: Email service not configured. Email functionality will be disabled.");
            Arc::new(EmailService::new(Arc::new(DummyEmailService {})))
        }
    };

    println!("Connected to Neo4j database successfully!");
    println!("App is listening at port: http://{}:{}", host, port);

    let domain_url = std::env::var("DOMAIN_URL").unwrap_or_else(|_| "http://localhost:3000".to_string());
    HttpServer::new(move || {
        App::new()
            .wrap(
                Cors::default()
                    .allowed_origin(&domain_url)
                    .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"])
                    .allowed_headers(vec!["Content-Type", "Authorization"])
                    .supports_credentials(),
            )
            .app_data(web::Data::new(database.clone()))
            .app_data(web::Data::new(canvas_service.clone()))
            .app_data(web::Data::new(node_service.clone()))
            .app_data(web::Data::new(vertex_ai_service.clone()))
            .app_data(web::Data::new(ai_service.clone()))
            .app_data(web::Data::new(auth_service.clone()))
            .app_data(web::Data::new(email_service.clone()))
            .service(hello)
            // Auth endpoints
            .service(auth_controller::signup)
            .service(auth_controller::login)
            .service(auth_controller::verify_token)
            .service(auth_controller::refresh_token)
            .service(auth_controller::logout)
            .service(auth_controller::verify_oauth_token)
            .service(auth_controller::forgot_password)
            .service(auth_controller::reset_password)
            // .service(auth_controller::get_user_by_id)
            // Email endpoints
            .service(email_controller::send_password_reset_email)
            .service(email_controller::send_password_reset_confirmation_email)
            .service(email_controller::send_email_confirmation)
            // Canvas CRUD operations
            .service(canvas_controller::create_canvas)
            .service(canvas_controller::get_canvas_list)
            .service(canvas_controller::get_canvas)
            .service(canvas_controller::update_canvas)
            .service(canvas_controller::delete_canvas)
            .service(canvas_controller::get_canvas_graph_data)
            // Node CRUD operations
            .service(node_controller::create_node)
            .service(node_controller::get_node_list)
            .service(node_controller::get_node)
            .service(node_controller::update_node)
            .service(node_controller::delete_node)
            .service(node_controller::get_nodes_by_canvas)
            .service(node_controller::delete_nodes_by_canvas)
            // AI endpoints
            .service(ai_controller::generate_ai_content)
            .service(ai_controller::generate_keywords)
    })
    .bind((host, port))?
    .run()
    .await
}
