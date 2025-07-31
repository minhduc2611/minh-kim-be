use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};

mod controllers;
mod dao;
mod database;
mod models;
mod services;

use controllers::canvas_controller;
use dao::canvas_dao::CanvasDao;
use dao::canvas_dao_trait::CanvasRepository;
use services::canvas_service::CanvasService;
use services::canvas_service_trait::CanvasServiceTrait;
use std::sync::Arc;

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
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

    println!("Connected to Neo4j database successfully!");
    println!("App is listening at port: http://{}:{}", host, port);

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(database.clone()))
            .app_data(web::Data::new(canvas_service.clone()))
            .service(hello)
            // New Canvas CRUD operations
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
