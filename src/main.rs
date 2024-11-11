use std::error::Error;

use crate::encumber::encumber;
use config::Config;
use env::wait_for_api_keys;
use pipeline::Pipeline;
use prompts::Prompts;
use release_credentials::timelock;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

pub mod agent;
pub mod attestation;
pub mod config;
pub mod db;
pub mod encumber;
pub mod env;
pub mod hyperbolic;
pub mod openai;
pub mod pipeline;
pub mod prompts;
pub mod release_credentials;
pub mod twitter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
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

    let prompts = Prompts::load();
    let config = Config::load();

    // First wait to be provided the api keys we need to run the AI Agen
    wait_for_api_keys().await;

    // then encumber the account
    tracing::info!("Beginning to encumber Account");
    let account_details = encumber((&config).into());
    tracing::info!("account encumberence succesful");
    // Server for attestation Quote
    tracing::info!("Starting Quote server");
    let quote_server_handle = tokio::task::spawn(attestation::server::quote_server(
        account_details.x_account.x_username.clone(),
    ));
    tracing::info!("Starting account details timelock");
    let timelock_handle = tokio::task::spawn(timelock(
        account_details.clone(),
        config.release_credentials,
        config.eth_rpc_url.clone(),
    ));

    tracing::info!("AI Agent starting");
    let mut pipeline = Pipeline::new(config, prompts, account_details).await;
    pipeline.run().await;

    // if pipeline stopped running we can shut her down
    quote_server_handle.abort();
    timelock_handle.abort();

    Ok(())
}
