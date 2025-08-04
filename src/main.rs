use actix_cors::Cors;
use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};

mod controllers;
mod dao;
mod database;
mod middleware;
mod models;
mod services;

use controllers::{auth_controller, canvas_controller, email_controller};
use dao::canvas_dao::CanvasDao;
use dao::canvas_dao_trait::CanvasRepository;
use services::auth_service::AuthService;
use services::auth_service_trait::AuthServiceTrait;
use services::canvas_service::CanvasService;
use services::canvas_service_trait::CanvasServiceTrait;
use services::email_service::EmailService;
use services::email_service_trait::EmailConfig;
use services::email_service_trait::EmailServiceTrait;
use services::dummy_email_service::DummyEmailService;
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
                position_x: 100.0,
                position_y: 200.0,
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
        Arc::new(CanvasService::new(canvas_repository));

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

    HttpServer::new(move || {
        App::new()
            .wrap(
                Cors::default()
                    .allowed_origin("http://localhost:3000")
                    .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"])
                    .allowed_headers(vec!["Content-Type", "Authorization"])
                    .supports_credentials(),
            )
            .app_data(web::Data::new(database.clone()))
            .app_data(web::Data::new(canvas_service.clone()))
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
    })
    .bind((host, port))?
    .run()
    .await
}
