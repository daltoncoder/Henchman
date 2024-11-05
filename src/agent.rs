use ethsign::SecretKey;
use std::collections::HashMap;
use std::path::PathBuf;

use crate::{
    config::Config,
    db::Database,
    hyperbolic::HyperbolicClient,
    prompts::Prompts,
    twitter::{
        api_types::{TimelineTweet, Tweet},
        TwitterClient,
    },
};
use anyhow::Result;

/// The AI agent that tweets
/// Should contain short term memory, long term memory, external context

pub struct Agent {
    prompts: Prompts,
    twitter_client: TwitterClient,
    hyperbolic_client: HyperbolicClient,
    database: Database,
    user_id: String,
    eth_private_key: SecretKey,
}

impl Agent {
    pub async fn new(config: Config, eth_private_key: SecretKey) -> Result<Self> {
        let Config {
            x_api_url,
            x_consumer_key,
            x_consumer_key_secret,
            x_access_token,
            x_access_token_secret,
            x_username,
            ..
        } = config;

        let twitter_client = TwitterClient::new(
            x_api_url,
            x_consumer_key,
            x_consumer_key_secret,
            x_access_token,
            x_access_token_secret,
        );
        let user_id = twitter_client
            .get_user_info_by_username(&x_username)
            .await?
            .id;

        // TODO: we should use Docker compose to start the DB before starting this agent.
        // For testing, pull the DB docker image with
        // `docker pull qdrant/qdrant`
        // and then run it with
        // `docker run -p 6333:6333 -p 6334:6334 qdrant/qdrant`
        let database = Database::new("http://localhost:6334", PathBuf::from(&config.kv_db_path))?; // TODO: get url from config

        let hyperbolic_client = HyperbolicClient::new(
            config.hyperbolic_api_key.clone(),
            config.hyperbolic_api_url.clone(),
        );

        // Create/seed database of long term memories

        // Do initial run

        Ok(Self {
            prompts: Prompts::default(),
            twitter_client,
            hyperbolic_client,
            database,
            user_id,
            eth_private_key,
        })
    }

    pub async fn run(&self) {
        // Step 1: retrieve own recent posts
        let recent_tweets = self.database.get_sent_tweets(20).unwrap();
        // Step 2: Fetch External Context(Notifications, timelines, and reply trees)
        // Step 2.1: filter all of the notifications for ones that haven't been seen before
        // Step 2.2: add to database every tweet id you have seen
        // Step 2.3: Check wallet address in posts and decide if we should take onchain action
        // Step 2.4: Decide to follow any users
        // Step 3: Generate Short-term memory
        // Step 4: Create embedding for short term memory
        // Step 5: Retrieve relevent long-term memories
        // Step 6: Generate new post or reply
        // Step 7: Score siginigicance of the new post
        // Step 8: Store the new post in long term memory if significant enough
        // Step 9: Submit Post

        todo!()
    }

    pub async fn get_timeline_tweets(&self) -> Result<Vec<TimelineTweet>> {
        let max_timeline_tweets = 50; // TODO: make config
        let mut tweets = self
            .twitter_client
            .get_timeline(&self.user_id, Some(max_timeline_tweets))
            .await?;

        let usernames: HashMap<&String, &String> = tweets
            .includes
            .users
            .iter()
            .map(|u| (&u.id, &u.username))
            .collect();

        for tweet in tweets.data.iter_mut() {
            if let Some(username) = usernames.get(&tweet.author_id) {
                tweet.username = Some(String::from(*username));
            }
        }
        let tweets = tweets
            .data
            .into_iter()
            .filter(|t| {
                !self
                    .database
                    .tweet_id_exists(&t.id)
                    // TODO: how should we handle errors in the main loop?
                    .expect("failed to read tweet id from db")
                    && t.username.is_some()
            })
            .collect();

        Ok(tweets)
    }

    pub async fn get_mentions(&self) -> Result<Vec<Tweet>> {
        let max_num_mentions = 50; // TODO: make config
        let mut mentions = self
            .twitter_client
            .get_mentions(&self.user_id, Some(max_num_mentions))
            .await?;

        let usernames: HashMap<&String, &String> = mentions
            .includes
            .users
            .iter()
            .map(|u| (&u.id, &u.username))
            .collect();

        for tweet in mentions.data.iter_mut() {
            if let Some(username) = usernames.get(&tweet.author_id) {
                tweet.username = Some(String::from(*username));
            }
        }
        let tweets = mentions
            .data
            .into_iter()
            .filter(|t| {
                !self
                    .database
                    .tweet_id_exists(&t.id)
                    // TODO: how should we handle errors in the main loop?
                    .expect("failed to read tweet id from db")
                    && t.username.is_some()
            })
            .collect();

        Ok(tweets)
    }

    pub async fn respond_to_mentions(&self) -> Result<()> {
        //let max_num_tweets = 50; // TODO: make config

        //let mentions = self
        //    .twitter_client
        //    .get_mentions(&self.user_id, Some(max_num_mentions))
        //    .await?;

        //for mention in mentions.data {
        //    let recent_tweets = self
        //        .twitter_client
        //        .get_user_tweets(mention.author_id, Some(max_num_tweets))
        //        .await?;
        //    // TODO: look for tweets that mention the bot?
        //    // TODO: get context from bot's timeline?
        //    // TODO: make tweets machine readable
        //    // TODO: get username for each tweet
        //}

        Ok(())
    }
}
