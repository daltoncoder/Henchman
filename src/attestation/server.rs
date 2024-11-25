use axum::{response::IntoResponse, routing::get, Json, Router};

use crate::attestation::ra::ra_get_quote;

use serde::Serialize;

use axum::http::{
    header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE},
    Method, //HeaderValue, 
};

use tower_http::cors::{Any, CorsLayer};
use ed25519_dalek::*;
use rand::rngs::OsRng;

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub message: String,
}

#[derive(Serialize, Debug)]
pub struct QuoteResponse {
    pub status: String,
    pub public_key: String,
    pub quote: String,
}

#[derive(Clone)]
pub struct AppState {
    pub twitter_username: String,
    pub keypair: SigningKey,
}

impl AppState {
    pub fn new(twitter_username: String) -> Self {
        let mut csprng = OsRng{};

        AppState {
            twitter_username,
            keypair: SigningKey::generate(&mut csprng),
        }
    }
}

pub async fn health_checker_handler() -> impl IntoResponse {
    const MESSAGE: &str = "Enclave is healthy!";

    let json_response = serde_json::json!({
        "status": "success",
        "message": MESSAGE
    });

    Json(json_response)
}

pub fn create_router(app_state: AppState) -> Router {
    let cors = CorsLayer::new()
        //.allow_origin("http://localhost:3000".parse::<HeaderValue>().unwrap())
        .allow_origin(Any)
        .allow_methods([Method::GET])
        .allow_credentials(true)
        .allow_headers([AUTHORIZATION, ACCEPT, CONTENT_TYPE]);

    Router::new()
        .route("/api/healthchecker", get(health_checker_handler))
        .route("/api/quote", get(ra_get_quote))
        .layer(cors)
        .with_state(app_state)
}

pub async fn quote_server(twitter_username: String) {
    let app_state = AppState::new(twitter_username);
    let app = create_router(app_state);

    tracing::info!("🚀 Quote Server started successfully");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
