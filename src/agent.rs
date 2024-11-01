use crate::{config::Config, prompts::Prompts, twitter::TwitterClient};
use anyhow::Result;

/// The AI agent that tweets
/// Should contain short term memory, long term memory, external context

pub struct Agent<'a> {
    prompts: Prompts,
    twitter_client: TwitterClient<'a>,
    user_id: String,
}

impl<'a> Agent<'a> {
    pub async fn new(config: &'a Config) -> Result<Self> {
        let twitter_client = TwitterClient::new(
            config.x_api_url.clone(),
            &config.x_consumer_key,
            &config.x_consumer_key_secret,
            &config.x_access_token,
            &config.x_access_token_secret,
        );
        let user_id = twitter_client.get_user_id(&config.x_username).await?.id;

        // Create/seed database of long term memories

        // Do initial run

        Ok(Self {
            prompts: Prompts::default(),
            twitter_client,
            user_id,
        })
    }

    pub async fn run(&self) {
        todo!()
    }

    pub async fn respond_to_mentions(&self) -> Result<()> {
        let mentions = self
            .twitter_client
            .get_mentions(&self.user_id, None)
            .await?;

        for mention in mentions.data {}

        Ok(())
    }
}
