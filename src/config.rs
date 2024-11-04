// The config file for this app acts as a typical .env file. Alot of API keys are needed for this agent

use std::{fs, path::PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Config {
    pub open_router_api_key: String,
    pub hyperbolic_api_key: String,
    pub open_ai_api_key: String,
    pub x_consumer_key: String,
    pub x_consumer_key_secret: String,
    pub x_access_token: String,
    pub x_access_token_secret: String,
    pub x_client_id: String,
    pub x_client_secret: String,
    pub eth_rpc_url: String,
    pub x_api_url: String,
    pub hyperbolic_api_url: String,
    pub x_username: String,
    pub kv_db_path: String,
    pub scroll_sleep: Option<(u64, u64)>,
    pub scroll_duration: Option<(u64, u64)>,
    pub run_sleep: Option<(u64, u64)>,
}

impl Config {
    pub fn load(path: PathBuf) -> Self {
        let raw = fs::read_to_string(path).expect("Unable to read config.toml");
        toml::from_str(&raw).expect("Unable to parse config.toml")
    }
}
