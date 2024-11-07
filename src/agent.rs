use ethsign::SecretKey;
use std::path::PathBuf;
use std::time::UNIX_EPOCH;
use std::{collections::HashMap, time::SystemTime};

use crate::{
    config::Config,
    db::{
        types::{Embedding, Memory, MemoryData},
        Database,
    },
    hyperbolic::HyperbolicClient,
    openai::OpenAIClient,
    prompts::Prompts,
    twitter::{
        api_types::{TimelineTweet, Tweet},
        TwitterClient,
    },
};
use anyhow::{anyhow, Result};

/// The AI agent that tweets
/// Should contain short term memory, long term memory, external context

const LONG_TERM_MEMORY: &str = "long-term-memory";

pub struct Agent {
    prompts: Prompts,
    twitter_client: TwitterClient,
    hyperbolic_client: HyperbolicClient,
    openai_client: OpenAIClient,
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
        let openai_client = OpenAIClient::new(config.open_ai_api_key, config.open_ai_api_url);

        // Create/seed database of long term memories

        // Do initial run

        Ok(Self {
            prompts: Prompts::default(),
            twitter_client,
            hyperbolic_client,
            openai_client,
            database,
            user_id,
            eth_private_key,
        })
    }

    pub async fn run(&self) -> Result<()> {
        let max_num_mentions = 50;
        let max_timeline_tweets = 50;
        let num_long_term_memories = 5;
        let min_storing_memory_score = 2;
        let min_posting_score = 3;
        let num_recent_posts = 20;

        // Step 1: retrieve own recent posts
        let recent_tweets = self.database.get_recent_memories(num_recent_posts)?;

        // Step 2: Fetch External Context(Notifications, timelines, and reply trees)
        // Step 2.1: filter all of the notifications for ones that haven't been seen before
        // Step 2.2: add to database every tweet id you have seen
        let timeline_tweets = self.get_timeline_tweets(Some(max_timeline_tweets)).await?;
        let mentions = self.get_mentions(Some(max_num_mentions)).await?;

        let mut context = Vec::with_capacity(timeline_tweets.len() + mentions.len());
        timeline_tweets
            .iter()
            .for_each(|t| context.push(t.to_string()));
        mentions.iter().for_each(|t| context.push(t.to_string()));

        // Step 2.3: Check wallet address in posts and decide if we should take onchain action

        // Step 2.4: Decide to follow any users
        self.follow_users(&timeline_tweets, &mentions).await?;

        // Step 3: Generate Short-term memory
        let short_term_memory = self.generate_short_term_memory(context.clone()).await?;

        // Step 4: Create embedding for short term memory
        // Step 5: Retrieve relevent long-term memories
        let long_term_memories = self
            .get_long_term_memories(&short_term_memory, num_long_term_memories)
            .await?;

        // Step 6: Generate new post or reply
        let recent_posts = recent_tweets
            .iter()
            .map(|t| t.to_string())
            .collect::<Vec<String>>();
        let tweet_prompt = self.prompts.get_tweet_prompt(
            short_term_memory,
            long_term_memories,
            recent_posts,
            context,
        );
        let mut tweet_res = self
            .hyperbolic_client
            .generate_text(
                &tweet_prompt,
                "Write a tweet that is less than 240 characters based on the context",
            )
            .await?;
        if tweet_res.choices.is_empty() {
            return Err(anyhow!("Failed to generate tweet"));
        }
        let tweet = tweet_res.choices.swap_remove(0).message.content;

        // Step 7: Score siginigicance of the new post
        let mut tweet_score = self
            .hyperbolic_client
            .generate_text(
                &tweet,
                "Respond only with a score from 1 to 10 for the given memory",
            )
            .await?;
        if tweet_score.choices.is_empty() {
            return Err(anyhow!("Failed to generate significance score for tweet"));
        }
        let tweet_score = tweet_score.choices.swap_remove(0).message.content;
        let tweet_score = tweet_score.parse::<u16>()?;

        // Step 8: Store the new post in long term memory if significant enough
        if tweet_score >= min_storing_memory_score {
            let mut embd_res = self.openai_client.get_text_embedding(&tweet).await?;
            if embd_res.data.is_empty() {
                return Err(anyhow!("Embedding data missing from OpenAI API response"));
            }
            let start = SystemTime::now();
            let since_the_epoch = start.duration_since(UNIX_EPOCH)?;
            let tweet_id = since_the_epoch.as_millis();
            let tweet_embd = Embedding::new(tweet_id, embd_res.data.swap_remove(0).embedding);
            self.database
                .upsert_memories(
                    LONG_TERM_MEMORY,
                    vec![Memory {
                        data: MemoryData {
                            id: tweet_id,
                            score: tweet_score,
                            content: tweet.clone(),
                        },
                        embedding: tweet_embd,
                    }],
                )
                .await?;
        }

        // Step 9: Submit Post
        if tweet_score >= min_posting_score {
            self.twitter_client.post_tweet(&tweet).await?;
        }

        Ok(())
    }

    /// Retrieves the latest tweets from the timeline.
    /// Filters out tweets that have already been seen.
    /// Marks retrieved tweets as seen.
    pub async fn get_timeline_tweets(
        &self,
        max_timeline_tweets: Option<u16>,
    ) -> Result<Vec<TimelineTweet>> {
        let mut tweets = self
            .twitter_client
            .get_timeline(&self.user_id, max_timeline_tweets)
            .await?;

        let usernames: HashMap<&String, &String> = tweets
            .includes
            .users
            .iter()
            .map(|u| (&u.id, &u.username))
            .collect();

        for tweet in tweets.data.iter_mut() {
            self.database.insert_tweet_id(&tweet.id)?;
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

    /// Retrieves the latest mentions.
    /// Filters out tweets that have already been seen.
    /// Marks retrieved tweets as seen.
    pub async fn get_mentions(&self, max_num_mentions: Option<u16>) -> Result<Vec<Tweet>> {
        let mut mentions = self
            .twitter_client
            .get_mentions(&self.user_id, max_num_mentions)
            .await?;

        let usernames: HashMap<&String, &String> = mentions
            .includes
            .users
            .iter()
            .map(|u| (&u.id, &u.username))
            .collect();

        for tweet in mentions.data.iter_mut() {
            self.database.insert_tweet_id(&tweet.id)?;
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

    pub async fn generate_short_term_memory(&self, context: Vec<String>) -> Result<String> {
        let prompt_context = self.prompts.get_short_term_memory_prompt(context);
        // TODO: try multiple times?
        let mut res = self
            .hyperbolic_client
            .generate_text(
                &prompt_context,
                "Respond only with your internal monologue based on the given context.",
            )
            .await?;
        if res.choices.is_empty() {
            return Err(anyhow!("Failed to generate short term memory"));
        }
        Ok(res.choices.swap_remove(0).message.content)
    }

    pub async fn get_long_term_memories(
        &self,
        short_term_memory: &str,
        num_long_term_memories: u64,
    ) -> Result<Vec<String>> {
        let mut embd_res = self
            .openai_client
            .get_text_embedding(short_term_memory)
            .await?;
        if embd_res.data.is_empty() {
            return Err(anyhow!("Embedding data missing from OpenAI API response"));
        }
        // 0 serves as a dummy id
        let short_term_mem_embd = Embedding::new(0, embd_res.data.swap_remove(0).embedding);
        let long_term_memories = self
            .database
            .get_k_most_similar_memories(
                LONG_TERM_MEMORY,
                short_term_mem_embd,
                num_long_term_memories,
            )
            .await?
            .into_iter()
            .map(|m| m.content)
            .collect();
        Ok(long_term_memories)
    }

    pub async fn follow_users(
        &self,
        timeline_tweets: &[TimelineTweet],
        mentions: &[Tweet],
    ) -> Result<()> {
        let mut username_to_id = HashMap::new();
        for tweet in timeline_tweets {
            if let Some(username) = &tweet.username {
                // Check if we are following this user already
                if !self.database.user_id_exists(&tweet.author_id)? {
                    username_to_id.insert(username.clone(), &tweet.author_id);
                }
            }
        }
        for tweet in mentions {
            if let Some(username) = &tweet.username {
                // Check if we are following this user already
                if !self.database.user_id_exists(&tweet.author_id)? {
                    username_to_id.insert(username.clone(), &tweet.author_id);
                }
            }
        }
        let usernames = username_to_id.keys().cloned().collect::<Vec<String>>();
        let follow_prompt = self.prompts.get_follow_prompt(usernames);

        let mut res = self
            .hyperbolic_client
            .generate_text(
                &follow_prompt,
                "Respond with one username from the list. The response should only contain the username.",
            )
            .await?;
        if res.choices.is_empty() {
            return Err(anyhow!("Failed to generate username to follow"));
        }
        let username = res.choices.swap_remove(0).message.content;
        let Some(target_user_id) = username_to_id.get(&username) else {
            return Err(anyhow!("Invalid username selected"));
        };

        self.twitter_client
            .follow_user(&self.user_id, target_user_id)
            .await?;

        Ok(())
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
