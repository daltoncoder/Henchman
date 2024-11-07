use aho_corasick::AhoCorasick;
use std::{fs, path::PathBuf};

use serde::{Deserialize, Serialize};

/// Loads prompts for certain situations from a config file so it can easily be swapped out and changed
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct Prompts {
    /// Template that takes in a list of usernames.
    pub follow_template: String,
    /// Template that takes in: Short term memories, Long term memories, external context(recent notifications and tags on twitter), recent posts, and example tweets to generate the AI's tweet
    pub tweet_template: String,
    /// Template takes in: Recent posts, external context to generate an internal monoluge for the ai agent
    pub short_term_memory_template: String,
    /// Template take in a memory and rates makes a prompt that returns a significance score 1-10 on how significant the memory is to the AI agent
    /// todo: Should we also make this take in personality data?
    pub significance_score_template: String,
    /// Template that takes in recent posts that have wallet addresses in them and decides if the AI should take on chain action like transfer them money
    pub wallet_decision_template: String,
    /// Template that formats or chooses the tweet to post if multiple are in the prompt. Takes in the original prompt as an arg
    pub formatter_template: String,
    /// List of example tweets we can add to our prompts to give more context
    pub example_tweets: Vec<String>,
}

impl Prompts {
    /// Loads a prompts.toml file to create this struct
    pub fn load(path: PathBuf) -> Self {
        let raw = fs::read_to_string(path).expect("Unable to read prompts.toml");
        toml::from_str(&raw).expect("Unable to parse prompts.toml")
    }

    pub fn get_follow_prompt(&self, usernames: Vec<String>) -> String {
        let patterns = &["{usernames}"];
        let replace_with = &[usernames.join("\n")];

        let ac = AhoCorasick::new(patterns).unwrap();

        ac.replace_all(&self.tweet_template, replace_with)
    }

    pub fn get_tweet_prompt(
        &self,
        short_term_memory: String,
        long_term_memories: Vec<String>,
        recent_posts: Vec<String>,
        external_context: Vec<String>,
    ) -> String {
        let patterns = &[
            "{short_term_memories}",
            "{long_term_memories}",
            "{external_context}",
            "{recent_posts}",
            "{example_tweets}",
        ];
        let replace_with = &[
            short_term_memory,
            long_term_memories.join("\n"),
            external_context.join("\n"),
            recent_posts.join("\n"),
            self.example_tweets.join("\n"),
        ];

        let ac = AhoCorasick::new(patterns).unwrap();

        ac.replace_all(&self.tweet_template, replace_with)
    }

    pub fn get_short_term_memory_prompt(&self, external_context: Vec<String>) -> String {
        let ac = AhoCorasick::new(["{external_context}"]).unwrap();
        ac.replace_all(
            &self.short_term_memory_template,
            &[external_context.join("\n")],
        )
    }

    pub fn get_significance_prompt(&self, memory: &str) -> String {
        let ac = AhoCorasick::new(["{memory}"]).unwrap();

        ac.replace_all(&self.significance_score_template, &[memory])
    }

    pub fn get_wallet_decision_prompt(
        &self,
        posts: Vec<String>,
        wallet_matches: Vec<String>,
        wallet_balance: String,
    ) -> String {
        let patterns = &["{posts}", "{matches}", "{wallet_balance}"];
        let replace_with = &[posts.join("\n"), wallet_matches.join("\n"), wallet_balance];

        let ac = AhoCorasick::new(patterns).unwrap();

        ac.replace_all(&self.wallet_decision_template, replace_with)
    }

    pub fn get_formatter_prompt(&self, original_prompt: &str) -> String {
        let ac = AhoCorasick::new(["{prompt}"]).unwrap();

        ac.replace_all(&self.formatter_template, &[original_prompt])
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

    println!("Formatter Template");
    println!("{}", prompts.formatter_template);

    println!("Example Tweets:");
    prompts.example_tweets.iter().for_each(|e| println!("{e}"));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_prompts() -> Prompts {
        Prompts::load("./prompts.toml".into())
    }

    fn get_vec_of_strings(text: &str) -> Vec<String> {
        [format!("{text}1"), format!("{text}2"), format!("{text}3")].to_vec()
    }
    #[test]
    fn test_get_tweet_prompt() {
        let prompts = get_prompts();

        let tweet = prompts.get_tweet_prompt(
            "This is my short term memory".to_string(),
            get_vec_of_strings("memory"),
            get_vec_of_strings("recent"),
            get_vec_of_strings("context"),
        );

        println!("{tweet}");
    }

    #[test]
    fn test_get_short_memory_prompt() {
        let prompts = get_prompts();

        let prompt = prompts.get_short_term_memory_prompt(get_vec_of_strings("context"));

        println!("{prompt}");
    }

    #[test]
    fn test_get_significance_prompt() {
        let prompts = get_prompts();

        let prompt = prompts.get_significance_prompt("this is my memory");

        println!("{prompt}");
    }

    #[test]
    fn test_get_wallet_decision_prompt() {
        let prompts = get_prompts();

        let prompt = prompts.get_wallet_decision_prompt(
            get_vec_of_strings("posts"),
            get_vec_of_strings("0x12..34"),
            "69.430".to_string(),
        );

        println!("{prompt}");
    }

    #[test]
    fn test_get_formatter_prompt() {
        let prompts = get_prompts();

        let prompt = prompts.get_formatter_prompt("This is the original prompt.");

        println!("{prompt}");
    }
}
