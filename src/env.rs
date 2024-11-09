use std::sync::{Arc, Mutex, OnceLock};

use serde::Deserialize;

use axum::http::{
    header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE},
    HeaderValue, Method,
};

use axum::{extract::State, routing::post, Json, Router};

use tokio::sync::oneshot::{self, Receiver, Sender};
use tower_http::cors::CorsLayer;

pub static ENV: OnceLock<EnvVariables> = OnceLock::new();

#[derive(Deserialize, Debug)]
pub struct EnvVariables {
    pub hyperbolic_api_key: String,
    pub open_ai_api_key: String,
}

pub async fn wait_for_api_keys() {
    tracing::info!("Waiting for api keys to be delivered");

    let cors = CorsLayer::new()
        .allow_origin("http://localhost:3000".parse::<HeaderValue>().unwrap())
        .allow_methods([Method::POST])
        .allow_credentials(true)
        .allow_headers([AUTHORIZATION, ACCEPT, CONTENT_TYPE]);

    let (shutdown_sender, shutdown_receiver): (Sender<()>, Receiver<()>) = oneshot::channel();

    let app = Router::new()
        .route("/", post(get_env_variables))
        .layer(cors)
        .with_state(Arc::new(Mutex::new(Some(shutdown_sender))));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:6969").await.unwrap();

    axum::serve(listener, app)
        .with_graceful_shutdown(async {
            shutdown_receiver.await.unwrap();
        })
        .await
        .expect("ENV server panic unexpectedly");
}

async fn get_env_variables(
    State(shutdown_sender): State<Arc<Mutex<Option<Sender<()>>>>>,
    Json(env_variables): Json<EnvVariables>,
) -> String {
    ENV.set(env_variables).expect("Was unable to set ENV");

    tracing::info!("Successfully set the ENV variables, shutting down server");

    //Now that we have the ENV shutdown the server
    // todo: clean this mess up
    shutdown_sender
        .lock()
        .unwrap()
        .take()
        .unwrap()
        .send(())
        .unwrap();

    "Successfully Set ENV variables".into()
}
