use ethsign::SecretKey;
use std::path::PathBuf;
use std::time::UNIX_EPOCH;
use std::{collections::HashMap, time::SystemTime};

use crate::encumber::XAccountDetails;
use crate::env::ENV;
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
const X_API_URL: &str = "https://api.twitter.com/2";
const HYPERBOLIC_API_URL: &str = "https://api.hyperbolic.xyz/v1";
const OPEN_AI_API_URL: &str = "https://api.openai.com/v1";

pub struct Agent {
    prompts: Prompts,
    twitter_client: TwitterClient,
    hyperbolic_client: HyperbolicClient,
    openai_client: OpenAIClient,
    database: Database,
    user_id: String,
    _eth_private_key: SecretKey,
    config: AgentConfig,
}

impl Agent {
    pub async fn new(
        account_details: XAccountDetails,
        config: Config,
        eth_private_key: SecretKey,
        prompts: Prompts,
    ) -> Result<Self> {
        let agent_config = AgentConfig::from(&config);

        let XAccountDetails {
            x_consumer_key,
            x_consumer_secret,
            x_access_token,
            x_access_token_secret,
            x_username,
            ..
        } = account_details;

        let env = ENV.get().expect("unreachable");

        let twitter_client = TwitterClient::new(
            X_API_URL.into(),
            x_consumer_key,
            x_consumer_secret,
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

        let hyperbolic_client =
            HyperbolicClient::new(env.hyperbolic_api_key.clone(), HYPERBOLIC_API_URL.into());
        let openai_client = OpenAIClient::new(env.open_ai_api_key.clone(), OPEN_AI_API_URL.into());

        // Create collection for vector db. By default the embedding size will be 1536.
        // See: https://platform.openai.com/docs/guides/embeddings
        database.create_collection(LONG_TERM_MEMORY, 1536).await?;

        // Create/seed database of long term memories

        // Do initial run

        Ok(Self {
            prompts,
            twitter_client,
            hyperbolic_client,
            openai_client,
            database,
            user_id,
            _eth_private_key: eth_private_key,
            config: agent_config,
        })
    }

    pub async fn run(&self) -> Result<()> {
        // Step 1: retrieve own recent posts
        tracing::info!("Reading recent posts...");
        let recent_tweets = self
            .database
            .get_recent_memories(self.config.num_recent_posts)?;

        // Step 2: Fetch External Context(Notifications, timelines, and reply trees)
        // Step 2.1: filter all of the notifications for ones that haven't been seen before
        // Step 2.2: add to database every tweet id you have seen
        tracing::info!("Get timeline tweets...");
        let timeline_tweets = self
            .get_timeline_tweets(self.config.max_timeline_tweets)
            .await?;
        tracing::info!("Get mentions...");
        let mentions = self.get_mentions(self.config.max_num_mentions).await?;
        tracing::info!("{mentions:?}");

        let mut context = Vec::with_capacity(timeline_tweets.len() + mentions.len());
        timeline_tweets
            .iter()
            .for_each(|t| context.push(t.to_string()));
        mentions.iter().for_each(|t| context.push(t.to_string()));
        tracing::info!("{context:?}");

        // Step 2.3: Check wallet address in posts and decide if we should take onchain action

        // Step 2.4: Decide to follow any users
        if let Err(e) = self.follow_users(&timeline_tweets, &mentions).await {
            tracing::info!("Failed to follow user: {e:?}");
        }

        // Step 3: Generate Short-term memory
        let short_term_memory = self.generate_short_term_memory(context.clone()).await?;
        tracing::info!("Short term memory:");
        tracing::info!("{short_term_memory}");

        // Step 4: Create embedding for short term memory
        // Step 5: Retrieve relevent long-term memories
        let long_term_memories = self
            .get_long_term_memories(&short_term_memory, self.config.num_long_term_memories)
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
        tracing::info!("Proposed tweet:");
        tracing::info!("{tweet}");

        // Step 7: Score siginigicance of the new post
        let tweet_score = self.score_tweet(&tweet, 3).await?;
        tracing::info!("Tweet score:");
        tracing::info!("{tweet_score}");

        // Step 8: Store the new post in long term memory if significant enough
        if tweet_score >= self.config.min_storing_memory_score {
            tracing::info!("Storing tweet in memory");
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
        if tweet_score >= self.config.min_posting_score {
            tracing::info!("Posting tweet");
            self.twitter_client.post_tweet(&tweet).await?;
        }

        // Step 10: Respond to mentions
        self.respond_to_mentions(&mentions, &tweet_prompt, 3)
            .await?;

        Ok(())
    }

    /// Retrieves the latest tweets from the timeline.
    /// Filters out tweets that have already been seen.
    /// Marks retrieved tweets as seen.
    pub async fn get_timeline_tweets(
        &self,
        max_timeline_tweets: usize,
    ) -> Result<Vec<TimelineTweet>> {
        let mut tweets = self
            .twitter_client
            // always get the max number of tweets (100) in case we already seen the newer ones
            // before
            .get_timeline(&self.user_id, Some(100))
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

        // TODO: we can buffer the unused tweets here to reduce API calls and prevent rate limiting
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
            .take(max_timeline_tweets)
            .collect::<Vec<TimelineTweet>>();

        for tweet in tweets.iter() {
            self.database.insert_tweet_id(&tweet.id)?;
        }

        Ok(tweets)
    }

    /// Retrieves the latest mentions.
    /// Filters out tweets that have already been seen.
    /// Marks retrieved tweets as seen.
    pub async fn get_mentions(&self, max_num_mentions: usize) -> Result<Vec<Tweet>> {
        let mut mentions = self
            .twitter_client
            // always get the max number of tweets (100) in case we already seen the newer ones
            // before
            .get_mentions(&self.user_id, Some(100))
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
            .take(max_num_mentions)
            .collect::<Vec<Tweet>>();

        for tweet in tweets.iter() {
            self.database.insert_tweet_id(&tweet.id)?;
        }

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

        tracing::info!("Deciding which user to follow...");
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
            return Err(anyhow!("Invalid username selected: {username}"));
        };
        tracing::info!("Following user: {username}");

        self.twitter_client
            .follow_user(&self.user_id, target_user_id)
            .await?;
        // Mark user_id as followed
        self.database.insert_user_id(target_user_id)?;

        Ok(())
    }

    pub async fn score_tweet(&self, tweet: &str, max_tries: u32) -> Result<u16> {
        let mut tries = 0;
        while tries < max_tries {
            let Ok(mut score) = self
                .hyperbolic_client
                .generate_text(
                    tweet,
                    "Respond with a score from 1 to 10 for the given memory. Your answer should only contain an integer.",
                )
                .await
            else {
                continue;
            };
            if score.choices.is_empty() {
                continue;
            }
            let score = score.choices.swap_remove(0).message.content;
            if let Ok(score) = score.parse::<u16>() {
                return Ok(score);
            }

            tries += 1;
        }

        Err(anyhow!("Failed to generate tweet score"))
    }

    pub async fn respond_to_mentions(
        &self,
        mentions: &[Tweet],
        context: &str,
        max_tries: u32,
    ) -> Result<()> {
        let mut mentions_list = Vec::with_capacity(mentions.len());
        let mut mentions_map = HashMap::with_capacity(mentions.len());
        for mention in mentions {
            mentions_list.push(format!("id: {}, tweet: {}", mention.id, mention.text));
            mentions_map.insert(&mention.id, mention);
        }
        let prompt_context = self.prompts.get_mentions_prompt(mentions_list);

        let mut tries = 0;
        while tries < max_tries {
            let Ok(mut res) = self.hyperbolic_client
            .generate_text(&prompt_context, "Give a score from 1 to 10 for each of these tweets. Your response should be in the CSV format, where the first column is the id and the second column is the score. There should not be a headline.")
            .await else {
                continue;
            };
            if res.choices.is_empty() {
                continue;
            }
            let scores = res.choices.swap_remove(0).message.content;
            let scores = scores.split("\n").collect::<Vec<&str>>();

            let mut max_score = 0;
            let mut max_id = "";
            for score in scores {
                let mut iter = score.split(",");
                let Some(id) = iter.next() else {
                    continue;
                };
                let id = id.trim();
                let Some(score) = iter.next() else {
                    continue;
                };
                let score = score.trim();
                let Ok(score) = score.parse::<u8>() else {
                    continue;
                };
                if score >= max_score {
                    max_score = score;
                    max_id = id;
                }
            }
            if max_score < self.config.min_mention_score {
                tracing::info!("No mentions found that are worth our time.");
                return Ok(());
            }
            let max_id = max_id.to_owned();
            let Some(mention) = mentions_map.get(&max_id) else {
                continue;
            };

            let Ok(mut res) = self
                .hyperbolic_client
                .generate_text(
                    context,
                    &format!("Write a witty response to this tweet: {mention}"),
                )
                .await
            else {
                continue;
            };
            if res.choices.is_empty() {
                continue;
            }
            let tweet = res.choices.swap_remove(0).message.content;

            if self
                .twitter_client
                .reply_to_tweet(&tweet, &mention.id)
                .await
                .is_ok()
            {
                tracing::info!("Sent response: {tweet}");
                return Ok(());
            }

            tries += 1;
        }

        Ok(())
    }
}

struct AgentConfig {
    max_num_mentions: usize,
    max_timeline_tweets: usize,
    num_long_term_memories: u64,
    min_storing_memory_score: u16,
    min_posting_score: u16,
    num_recent_posts: usize,
    min_mention_score: u8,
}

impl From<&Config> for AgentConfig {
    fn from(value: &Config) -> Self {
        Self {
            max_num_mentions: value.max_num_mentions,
            max_timeline_tweets: value.max_timeline_tweets,
            num_long_term_memories: value.num_long_term_memories,
            min_storing_memory_score: value.min_storing_memory_score,
            min_posting_score: value.min_posting_score,
            num_recent_posts: value.num_recent_posts,
            min_mention_score: value.min_mention_score,
        }
    }
}
