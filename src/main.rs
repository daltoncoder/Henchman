use std::error::Error;

use crate::encumber::encumber;
use config::Config;
use pipeline::Pipeline;
use prompts::Prompts;
use release_credentials::timelock;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

pub mod agent;
pub mod attestation;
pub mod config;
pub mod db;
pub mod encumber;
pub mod hyperbolic;
pub mod openai;
pub mod pipeline;
pub mod prompts;
pub mod release_credentials;
pub mod twitter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let prompts = Prompts::load();
    let config = Config::load();

    let mut pipeline = Pipeline::new(config, prompts).await;
    pipeline.run().await;

    // Quote Server logs config
    let fmt_layer = fmt::layer()
        .with_target(true)
        .with_level(true)
        .with_thread_ids(false)
        .with_thread_names(true);

    let filter_layer = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new::<String>("Debug".into()))
        .expect("Error tracing subscriber filter layer");

    tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer)
        .init();

    // Server for attestation Quote
    attestation::server::quote_server().await;

    Ok(())
}
