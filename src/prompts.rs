use std::{fs, path::PathBuf};

use serde::{Deserialize, Serialize};

/// Loads prompts for certain situations from a config file so it can easily be swapped out and changed
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Prompts {
    /// Template that takes in: Short term memories, Long term memories, external context(recent notifications and tags on twitter), recent posts, and example tweets to generate the AI's tweet
    pub tweet_template: String,
    /// Template takes in: Recent posts, external context to generate an internal monoluge for the ai agent
    pub short_term_memory_template: String,
    /// Template take in a memory and rates makes a prompt that returns a significance score 1-10 on how significant the memory is to the AI agent
    /// todo: Should we also make this take in personality data?
    pub significance_score_template: String,
    /// Template that takes in recent posts that have wallet addresses in them and decides if the AI should take on chain action like transfer them money
    pub wallet_decision_template: String,
    /// List of example tweets we can add to our prompts to give more context
    pub example_tweets: Vec<String>,
}

impl Prompts {
    /// Loads a prompts.toml file to create this struct
    pub fn load(path: PathBuf) -> Self {
        let raw = fs::read_to_string(path).expect("Unable to read prompts.toml");
        toml::from_str(&raw).expect("Unable to parse prompts.toml")
    }
}

// todo: Should we set defaults here incase the prompts.toml doesnt exist??
impl Default for Prompts {
    fn default() -> Self {
        Self {
            tweet_template: Default::default(),
            short_term_memory_template: Default::default(),
            significance_score_template: Default::default(),
            wallet_decision_template: Default::default(),
            example_tweets: Default::default(),
        }
    }
}

#[test]
fn test_our_prompts_toml() {
    let prompts = Prompts::load("./prompts.toml".into());

    println!("Tweet Template:");
    println!("{}", prompts.tweet_template);

    println!("Short Term Memory Template:");
    println!("{}", prompts.short_term_memory_template);

    println!("Signficance Score Template:");
    println!("{}", prompts.significance_score_template);

    println!("Wallet Decision Template:");
    println!("{}", prompts.wallet_decision_template);

    println!("Example Tweets:");
    prompts.example_tweets.iter().for_each(|e| println!("{e}"));
}
