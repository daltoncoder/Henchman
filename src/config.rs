// The config file for this app acts as a typical .env file. Alot of API keys are needed for this agent

use std::{fs, path::PathBuf};

use serde::{Deserialize, Serialize};

const CONFIG: &str = include_str!("../config.toml");

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Config {
    pub email: String,
    pub email_password: String,
    pub x_username: String,
    pub x_password: String,
    pub eth_rpc_url: String,
    pub kv_db_path: String,
    pub min_storing_memory_score: u16,
    pub min_posting_score: u16,
    pub max_num_mentions: usize,
    pub max_timeline_tweets: usize,
    pub num_long_term_memories: u64,
    pub num_recent_posts: usize,
    pub scroll_sleep: Option<(u64, u64)>,
    pub scroll_duration: Option<(u64, u64)>,
    pub run_sleep: Option<(u64, u64)>,
    pub release_credentials: u64,
}

impl Config {
    pub fn load() -> Self {
        toml::from_str(CONFIG).expect("Unable to parse config.toml")
    }
}

#[test]
fn test_load() {
    Config::load();
}
