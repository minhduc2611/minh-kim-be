use actix_cors::Cors;
use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};

mod controllers;
mod dao;
mod database;
mod middleware;
mod models;
mod services;

use controllers::{auth_controller, canvas_controller};
use dao::canvas_dao::CanvasDao;
use dao::canvas_dao_trait::CanvasRepository;
use services::auth_service::AuthService;
use services::auth_service_trait::AuthServiceTrait;
use services::canvas_service::CanvasService;
use services::canvas_service_trait::CanvasServiceTrait;
use std::sync::Arc;

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
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
            .service(hello)
            // Auth endpoints
            .service(auth_controller::signup)
            .service(auth_controller::login)
            .service(auth_controller::verify_token)
            .service(auth_controller::refresh_token)
            .service(auth_controller::logout)
            .service(auth_controller::verify_oauth_token)
            // .service(auth_controller::get_user_by_id)
            // Canvas CRUD operations
            .service(canvas_controller::create_canvas)
            .service(canvas_controller::get_canvas_list)
            .service(canvas_controller::get_canvas)
            .service(canvas_controller::update_canvas)
            .service(canvas_controller::delete_canvas)
    })
    .bind((host, port))?
    .run()
    .await
}
