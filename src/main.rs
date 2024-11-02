use config::Config;
use prompts::Prompts;

pub mod agent;
pub mod config;
pub mod db;
pub mod hyperbolic;
pub mod openai;
pub mod pipeline;
pub mod prompts;
pub mod twitter;

fn main() {
    let prompts = Prompts::load("./prompts.toml".into());
    let config = Config::load("./config.toml".into());
}
