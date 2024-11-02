use crate::{config::Config, db::DB, prompts::Prompts, twitter::TwitterClient};
use anyhow::Result;

/// The AI agent that tweets
/// Should contain short term memory, long term memory, external context

pub struct Agent<'a> {
    prompts: Prompts,
    twitter_client: TwitterClient<'a>,
    db: DB,
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
        let user_id = twitter_client
            .get_user_info_by_username(&config.x_username)
            .await?
            .id;
        let db = DB::new("http://localhost:6334")?; // TODO: get from config

        // Create/seed database of long term memories

        // Do initial run

        Ok(Self {
            prompts: Prompts::default(),
            twitter_client,
            db,
            user_id,
        })
    }

    pub async fn run(&self) {
        todo!()
    }

    pub async fn respond_to_mentions(&self) -> Result<()> {
        let max_num_mentions = 50; // TODO: make config
        let max_num_tweets = 50; // TODO: make config

        let mentions = self
            .twitter_client
            .get_mentions(&self.user_id, Some(max_num_mentions))
            .await?;

        for mention in mentions.data {
            let recent_tweets = self
                .twitter_client
                .get_user_tweets(mention.author_id, Some(max_num_tweets))
                .await?;
            // TODO: look for tweets that mention the bot?
            // TODO: get context from bot's timeline?
            // TODO: make tweets machine readable
            // TODO: get username for each tweet
        }

        Ok(())
    }
}
