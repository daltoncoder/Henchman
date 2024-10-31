use crate::{prompts::Prompts, twitter::TwitterClient};

/// The AI agent that tweets
/// Should contain short term memory, long term memory, external context

pub struct Agent {
    prompts: Prompts,
    twitter_client: TwitterClient,
}

impl Agent {
    pub fn new() -> Self {
        // Create/seed database of long term memories

        // Do initial run

        todo!()
    }
}
