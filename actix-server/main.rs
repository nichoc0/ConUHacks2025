use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use std::env;

// Import our DB module; adjust the path as needed based on your project structure.
use db::NetworkDB;

// Existing endpoints
#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}

async fn manual_hello() -> impl Responder {
    HttpResponse::Ok().body("Hey there!")
}

// NEW: API endpoint that returns all normal network events
#[get("/api/events")]
async fn api_events(db: web::Data<NetworkDB>) -> impl Responder {
    match db.get_normal_events().await {
        Ok(events) => HttpResponse::Ok().json(events),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

// NEW: API endpoint that returns all suspicious network events
#[get("/api/suspicious")]
async fn api_suspicious(db: web::Data<NetworkDB>) -> impl Responder {
    match db.get_suspicious_events().await {
        Ok(events) => HttpResponse::Ok().json(events),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

#[derive(serde::Deserialize)]
struct ReportRequest {
    event_id: String,
}

// NEW: API endpoint for report generation (placeholder)
#[post("/api/report")]
async fn api_report(item: web::Json<ReportRequest>) -> impl Responder {
    let report = format!("Placeholder report for event with id: {}", item.event_id);
    HttpResponse::Ok().json(serde_json::json!({ "report": report }))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize MongoDB connection
    let db = NetworkDB::new().await.expect("Failed to connect to MongoDB");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(db.clone()))
            .service(hello)
            .service(echo)
            .service(api_events)
            .service(api_suspicious)
            .service(api_report)
            .route("/hey", web::get().to(manual_hello))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
