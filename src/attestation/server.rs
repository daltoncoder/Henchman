use axum::{
    response::IntoResponse, routing::get, Router, Json,
};

use crate::attestation::ra::ra_get_quote;

use serde::Serialize;

use axum::http::{
    header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE},
    HeaderValue, Method,
};

use tower_http::cors::CorsLayer;


#[derive(Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub message: String,
}

#[derive(Serialize, Debug)]
pub struct QuoteResponse {
    pub status: String,
    pub quote: String,
}

pub async fn health_checker_handler() -> impl IntoResponse {
    const MESSAGE: &str = "Enclave is healthy!";

    let json_response = serde_json::json!({
        "status": "success",
        "message": MESSAGE
    });

    Json(json_response)
}

pub fn create_router() -> Router {

    Router::new()
        .route("/api/healthchecker", get(health_checker_handler))
        .route("/api/quote", get(ra_get_quote))
}


pub async fn quote_server() {
    let cors = CorsLayer::new()
        .allow_origin("http://localhost:3000".parse::<HeaderValue>().unwrap())
        .allow_methods([Method::GET])
        .allow_credentials(true)
        .allow_headers([AUTHORIZATION, ACCEPT, CONTENT_TYPE]);
 
    let app = create_router().layer(cors);

    println!("🚀 Quote Server started successfully");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}