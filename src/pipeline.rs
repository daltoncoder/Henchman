// Get a different name for this other than pipeline
// purpose of "pipeline" is to simulate twitter browsing
// Bot should enter active stage and start "scrolling" twitter for a random amount of time around 15 minutes
// During this time bot might make several posts
// Afterwards it should sleep for for a random amount of time to simulate getting off twitter

use ethsign::SecretKey;
use rand::Rng;
use std::{
    cell::OnceCell,
    sync::{Arc, Mutex, OnceLock},
    time::Duration,
};
use tokio::{
    select,
    sync::oneshot::{self, Receiver, Sender},
};

use axum::{
    extract::State,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};

use crate::attestation::ra::ra_get_quote;

use serde::{Deserialize, Serialize};

use axum::http::{
    header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE},
    HeaderValue, Method,
};

use tower_http::cors::CorsLayer;

use crate::{agent::Agent, config::Config, encumber::encumber, prompts::Prompts};

pub static ENV: OnceLock<EnvVariables> = OnceLock::new();

pub struct Pipeline {
    /// The Ai Agent
    config: PipelineConfig,
    agent: Agent,
}

impl Pipeline {
    pub async fn new(config: Config, prompts: Prompts) -> Self {
        // First wait to be provided the api keys we need to run the AI Agen
        wait_for_api_keys().await;

        // then encumber the account
        let account_details = encumber((&config).into());

        let pipeline_config: PipelineConfig = (&config).into();
        let agent: Agent = Agent::new(
            account_details.x_account,
            config,
            generate_eth_private_key(),
            prompts,
        )
        .await
        .expect("Failed to create Agent");

        Self {
            agent,
            config: pipeline_config,
        }
    }

    /// Should not return until agent is shut down
    pub async fn run(&mut self) {
        // Generate Ethereum Address

        // Do an initial run
        loop {
            let scroll_wait_time = self.config.get_scroll_sleep_time();

            println!("Waiting to start scrolling : {:?}", scroll_wait_time);

            tokio::time::sleep(scroll_wait_time).await;

            let scroll_duration = self.config.get_scroll_duration_time();
            println!("Starting Scrolling for: {:?}", scroll_duration);
            let scroll_duration_fut = tokio::time::sleep(scroll_duration);

            // todo: I think tokio Sleep is cancel safe we probably dont need to pin here
            tokio::pin!(scroll_duration_fut);
            loop {
                let run_sleep_time = self.config.get_run_sleep_time();
                println!("Next run in: {:?}", run_sleep_time);

                let run_sleep_fut = tokio::time::sleep(run_sleep_time);
                select! {
                    _ = &mut scroll_duration_fut => {
                        println!("End Scrolling");
                        break;
                    }

                    _ = run_sleep_fut => {
                        if let Err(e) = self.agent.run().await {
                            println!("Error while running error: {e:?}");
                        };
                    }
                }
            }
        }
    }
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

#[derive(Deserialize, Debug)]
pub struct EnvVariables {
    pub hyperbolic_api_key: String,
    pub open_ai_api_key: String,
}

struct PipelineConfig {
    scroll_sleep_min: u64,
    scroll_sleep_max: u64,
    scroll_duration_min: u64,
    scroll_duration_max: u64,
    run_sleep_min: u64,
    run_sleep_max: u64,
}

impl PipelineConfig {
    /// Returns random time in the scroll sleep range
    pub fn get_scroll_sleep_time(&self) -> Duration {
        let mut rng = rand::thread_rng();

        let time_to_wait = rng.gen_range(self.scroll_sleep_min..=self.scroll_sleep_max);

        Duration::from_secs(time_to_wait)
    }

    /// Returns a random time do scroll for
    pub fn get_scroll_duration_time(&self) -> Duration {
        let mut rng = rand::thread_rng();

        let time_to_scroll = rng.gen_range(self.scroll_duration_min..=self.scroll_duration_max);

        Duration::from_secs(time_to_scroll)
    }

    /// Returns random time in the run sleep range
    pub fn get_run_sleep_time(&self) -> Duration {
        let mut rng = rand::thread_rng();

        let time_to_wait = rng.gen_range(self.run_sleep_min..=self.run_sleep_max);

        Duration::from_secs(time_to_wait)
    }
}

impl From<&Config> for PipelineConfig {
    fn from(value: &Config) -> Self {
        // 0-30min default
        let (scroll_sleep_min, scroll_sleep_max) = value.scroll_sleep.unwrap_or((0, 1800));
        // 15-20min default
        let (scroll_duration_min, scroll_duration_max) =
            value.scroll_duration.unwrap_or((900, 1200));
        // 30sec-3min default
        let (run_sleep_min, run_sleep_max) = value.run_sleep.unwrap_or((30, 180));

        Self {
            scroll_sleep_min,
            scroll_sleep_max,
            scroll_duration_min,
            scroll_duration_max,
            run_sleep_min,
            run_sleep_max,
        }
    }
}

impl Default for PipelineConfig {
    fn default() -> Self {
        // scroll sleep 0-30 min
        // scroll duration 15-20min
        // run sleep 30sec-3 min
        Self {
            scroll_sleep_min: 0,
            scroll_sleep_max: 1800,
            scroll_duration_min: 900,
            scroll_duration_max: 1200,
            run_sleep_min: 30,
            run_sleep_max: 180,
        }
    }
}

fn generate_eth_private_key() -> SecretKey {
    let mut rng = rand::thread_rng();
    let mut random_bytes = [0u8; 32];

    rng.fill(&mut random_bytes);

    // todo i think there is a nonzero chance this could be out of range of the eliptic curve
    SecretKey::from_raw(&random_bytes).expect("Failed to generate ethereum key")
}

#[test]
fn test_random_sleep_time() {
    let config = PipelineConfig::default();
    let scroll_min = Duration::from_secs(0);
    let scroll_max = Duration::from_secs(1800);
    let run_min = Duration::from_secs(30);
    let run_max = Duration::from_secs(180);

    for _ in 0..100 {
        let run_sleep = config.get_run_sleep_time();
        let scroll_sleep = config.get_scroll_sleep_time();

        println!("run_sleep: {:?}", run_sleep);
        println!("scroll_sleep: {:?}", scroll_sleep);

        assert!(run_sleep <= run_max && run_sleep >= run_min);
        assert!(scroll_sleep <= scroll_max && scroll_sleep >= scroll_min);
    }
}
