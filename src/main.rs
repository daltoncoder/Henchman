use std::error::Error;

use config::Config;
use pipeline::Pipeline;
use prompts::Prompts;

pub mod agent;
pub mod config;
pub mod db;
pub mod hyperbolic;
pub mod openai;
pub mod pipeline;
pub mod prompts;
pub mod twitter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let prompts = Prompts::load("./prompts.toml".into());
    let config = Config::load("./config.toml".into());
    let mut pipeline = Pipeline::new(&config, prompts).await;
    pipeline.run().await;

    Ok(())
}
