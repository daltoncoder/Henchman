// Client that makes all requests to the twitter client

use anyhow::{anyhow, Result};
use reqwest_oauth1::{Client, DefaultSM, OAuthClientProvider, Secrets, Signer};

pub mod api_types;
use api_types::MentionsResponse;

use crate::twitter::api_types::ApiResponse;

use self::api_types::{FollowData, SentTweet, TimelineResponse, Tweet, TweetsResponse, User};

pub struct TwitterClient {
    client: Client<Signer<'static, Secrets<'static>, DefaultSM>>,
    base_url: String,
}

impl TwitterClient {
    pub fn new(
        url: String,
        x_consumer_key: String,
        x_consumer_secret: String,
        x_access_token: String,
        x_access_token_secret: String,
    ) -> Self {
        let client = reqwest::Client::new();

        let secrets: Secrets = Secrets::new(x_consumer_key, x_consumer_secret)
            .token(x_access_token, x_access_token_secret);

        let client = client.oauth1(secrets);

        Self {
            client,
            base_url: url,
        }
    }

    /// Returns a list of tweets that mention the user with id `user_id`.
    /// The list of tweets is ordered by date created (newest first).
    /// By default, the list contains at most 10 tweets. We can increase this
    /// limit by passing the 'max_results' param. The value has to be between 0 and 1000.
    pub async fn get_mentions(
        &self,
        user_id: &str,
        max_results: Option<u16>,
    ) -> Result<MentionsResponse> {
        let url = if let Some(max_results) = max_results {
            format!(
            "{}/users/{user_id}/mentions?tweet.fields=created_at&expansions=author_id&max_results={max_results}",
            self.base_url
            )
        } else {
            format!(
                "{}/users/{user_id}/mentions?tweet.fields=created_at&expansions=author_id",
                self.base_url
            )
        };

        //let res = self
        //    .client
        //    .get(url)
        //    .send()
        //    .await
        //    .unwrap()
        //    .text()
        //    .await
        //    .unwrap();

        //println!("{res}");

        //todo!()

        self.client
            .get(url)
            .send()
            .await
            .map_err(|e| anyhow!("{e:?}"))?
            .json::<MentionsResponse>()
            .await
            .map_err(|e| anyhow!("{e:?}"))
    }

    /// Returns a list of tweets and retweets posted by the user with `user_id`
    /// or a users this account follows
    /// We can set the  limit for the number of tweets by passing the 'max_results' param.
    pub async fn get_timeline(
        &self,
        user_id: &str,
        max_results: Option<u16>,
    ) -> Result<TimelineResponse> {
        let url = if let Some(max_results) = max_results {
            format!(
                "{}/users/{user_id}/timelines/reverse_chronological?expansions=author_id&max_results={max_results}",
                self.base_url
            )
        } else {
            format!(
                "{}/users/{user_id}/timelines/reverse_chronological?expansions=author_id",
                self.base_url
            )
        };

        self.client
            .get(url)
            .send()
            .await
            .map_err(|e| anyhow!("{e:?}"))?
            .json::<TimelineResponse>()
            .await
            .map_err(|e| anyhow!("{e:?}"))
    }

    /// Retrieves the tweet data for the tweet with id 'tweet_id'.
    pub async fn get_tweet(&self, tweet_id: String) -> Result<Tweet> {
        let url = format!(
            "{}/tweets/{tweet_id}?tweet.fields=author_id,created_at",
            self.base_url
        );

        self.client
            .get(url)
            .send()
            .await
            .map_err(|e| anyhow!("{e:?}"))?
            .json::<ApiResponse<Tweet>>()
            .await
            .map_err(|e| anyhow!("{e:?}"))
            .map(|res| res.data)
    }

    /// Returns a list of tweets created by the user with id `user_id`.
    /// The list of tweets is ordered by date created (newest first).
    /// By default, the list contains at most 10 tweets. We can increase this
    /// limit by passing the 'max_results' param. The value has to be between 0 and 1000.
    pub async fn get_user_tweets(
        &self,
        user_id: String,
        max_results: Option<u16>,
    ) -> Result<TweetsResponse> {
        let url = if let Some(max_results) = max_results {
            format!(
            "{}/users/{user_id}/tweets?tweet.fields=author_id,created_at&max_results={max_results}",
            self.base_url
            )
        } else {
            format!(
                "{}/users/{user_id}/tweets?tweet.fields=author_id,created_at",
                self.base_url
            )
        };

        self.client
            .get(url)
            .send()
            .await
            .map_err(|e| anyhow!("{e:?}"))?
            .json::<TweetsResponse>()
            .await
            .map_err(|e| anyhow!("{e:?}"))
    }

    /// Posts a tweet and returns the tweet data on success.
    pub async fn post_tweet(&self, content: &str) -> Result<SentTweet> {
        let url = format!("{}/tweets", self.base_url);

        let json = serde_json::json!({
            "text": content,
        });

        self.client
            .post(url)
            .header("Content-Type", "application/json")
            .body(json.to_string())
            .send()
            .await
            .map_err(|e| anyhow!("{e:?}"))?
            .json::<ApiResponse<SentTweet>>()
            .await
            .map_err(|e| anyhow!("{e:?}"))
            .map(|res| res.data)
    }

    /// Replies to the tweet with id 'tweet_id'.
    pub async fn reply_to_tweet(&self, content: &str, tweet_id: &str) -> Result<SentTweet> {
        let url = format!("{}/tweets", self.base_url);

        let json = serde_json::json!({
            "text": content,
            "reply": { "in_reply_to_tweet_id": tweet_id }
        });

        self.client
            .post(url)
            .header("Content-Type", "application/json")
            .body(json.to_string())
            .send()
            .await
            .map_err(|e| anyhow!("{e:?}"))?
            .json::<ApiResponse<SentTweet>>()
            .await
            .map_err(|e| anyhow!("{e:?}"))
            .map(|res| res.data)
    }

    /// Retrieves the user info (username, name, user_id) for the user with the specified username.
    pub async fn get_user_info_by_username(&self, username: &str) -> Result<User> {
        let url = format!("{}/users/by/username/{username}", self.base_url);

        self.client
            .get(url)
            .send()
            .await
            .map_err(|e| anyhow!("{e:?}"))?
            .json::<ApiResponse<User>>()
            .await
            .map_err(|e| anyhow!("{e:?}"))
            .map(|res| res.data)
    }

    /// Retrieves the user info (username, name, user_id) for the user with the specified id.
    pub async fn get_user_info_by_id(&self, user_id: &str) -> Result<User> {
        let url = format!("{}/users/{user_id}", self.base_url);

        self.client
            .get(url)
            .send()
            .await
            .map_err(|e| anyhow!("{e:?}"))?
            .json::<ApiResponse<User>>()
            .await
            .map_err(|e| anyhow!("{e:?}"))
            .map(|res| res.data)
    }

    /// Follow the user with id `user_id`.
    pub async fn follow_user(&self, user_id: &str, target_user_id: &str) -> Result<FollowData> {
        let url = format!("{}/users/{user_id}/following", self.base_url);

        let json = serde_json::json!({
            "target_user_id": target_user_id,
        });

        self.client
            .post(url)
            .header("Content-Type", "application/json")
            .body(json.to_string())
            .send()
            .await
            .map_err(|e| anyhow!("{e:?}"))?
            .json::<ApiResponse<FollowData>>()
            .await
            .map_err(|e| anyhow!("{e:?}"))
            .map(|res| res.data)
    }
}

#[cfg(test)]
mod tests {

    use super::TwitterClient;

    fn get_secrets() -> (String, String, String, String) {
        let x_consumer_key = "".to_string();
        let x_consumer_secret = "".to_string();
        let x_access_token = "".to_string();
        let x_access_token_secret = "".to_string();

        (
            x_consumer_key,
            x_consumer_secret,
            x_access_token,
            x_access_token_secret,
        )
    }

    #[ignore]
    #[tokio::test]
    async fn test_get_timeline() {
        let (x_consumer_key, x_consumer_secret, x_access_token, x_access_token_secret) =
            get_secrets();
        let base_url = "https://api.twitter.com/2".to_string();
        let client = TwitterClient::new(
            base_url,
            x_consumer_key,
            x_consumer_secret,
            x_access_token,
            x_access_token_secret,
        );

        let tweets = client
            .get_timeline("1852012860596981761", Some(5))
            .await
            .unwrap();
        println!("users: {:?}", tweets.includes);
        for tweet in tweets.data {
            println!("{tweet:?}");
        }
    }

    #[ignore]
    #[tokio::test]
    async fn test_get_mentions() {
        let (x_consumer_key, x_consumer_secret, x_access_token, x_access_token_secret) =
            get_secrets();
        let base_url = "https://api.twitter.com/2".to_string();
        let client = TwitterClient::new(
            base_url,
            x_consumer_key,
            x_consumer_secret,
            x_access_token,
            x_access_token_secret,
        );

        let mentions = client
            .get_mentions("1852012860596981761", None)
            .await
            .unwrap();
        println!("users: {:?}", mentions.includes);
        for mention in mentions.data {
            println!("{mention:?}");
        }
    }

    #[ignore]
    #[tokio::test]
    async fn test_get_tweet() {
        let (x_consumer_key, x_consumer_secret, x_access_token, x_access_token_secret) =
            get_secrets();
        let base_url = "https://api.twitter.com/2".to_string();
        let client = TwitterClient::new(
            base_url,
            x_consumer_key,
            x_consumer_secret,
            x_access_token,
            x_access_token_secret,
        );

        let tweet = client
            .get_tweet("1852054615954432343".to_string())
            .await
            .unwrap();
        println!("{tweet:?}");
    }

    #[ignore]
    #[tokio::test]
    async fn test_get_user_tweets() {
        let (x_consumer_key, x_consumer_secret, x_access_token, x_access_token_secret) =
            get_secrets();
        let base_url = "https://api.twitter.com/2".to_string();
        let client = TwitterClient::new(
            base_url,
            x_consumer_key,
            x_consumer_secret,
            x_access_token,
            x_access_token_secret,
        );

        let tweets = client
            .get_user_tweets("1851820330513473536".to_string(), None)
            .await
            .unwrap();
        println!("num_tweets: {}", tweets.data.len());
        for tweet in tweets.data {
            println!("{tweet:?}");
        }
    }

    #[ignore]
    #[tokio::test]
    async fn test_post_tweet() {
        let (x_consumer_key, x_consumer_secret, x_access_token, x_access_token_secret) =
            get_secrets();
        let base_url = "https://api.twitter.com/2".to_string();
        let client = TwitterClient::new(
            base_url,
            x_consumer_key,
            x_consumer_secret,
            x_access_token,
            x_access_token_secret,
        );

        let tweet = client.post_tweet("mic check 3").await.unwrap();
        println!("{tweet:?}");
    }

    #[ignore]
    #[tokio::test]
    async fn test_reply_to_tweet() {
        let (x_consumer_key, x_consumer_secret, x_access_token, x_access_token_secret) =
            get_secrets();
        let base_url = "https://api.twitter.com/2".to_string();
        let client = TwitterClient::new(
            base_url,
            x_consumer_key,
            x_consumer_secret,
            x_access_token,
            x_access_token_secret,
        );

        let tweet = client
            .reply_to_tweet("oh really", "1852054615954432343")
            .await
            .unwrap();
        println!("{tweet:?}");
    }

    #[ignore]
    #[tokio::test]
    async fn test_get_user_info_by_username() {
        let (x_consumer_key, x_consumer_secret, x_access_token, x_access_token_secret) =
            get_secrets();
        let base_url = "https://api.twitter.com/2".to_string();
        let client = TwitterClient::new(
            base_url,
            x_consumer_key,
            x_consumer_secret,
            x_access_token,
            x_access_token_secret,
        );

        let user = client
            .get_user_info_by_username("omarskittle")
            .await
            .unwrap();
        println!("{user:?}");
    }

    #[ignore]
    #[tokio::test]
    async fn test_get_user_info_by_id() {
        let (x_consumer_key, x_consumer_secret, x_access_token, x_access_token_secret) =
            get_secrets();
        let base_url = "https://api.twitter.com/2".to_string();
        let client = TwitterClient::new(
            base_url,
            x_consumer_key,
            x_consumer_secret,
            x_access_token,
            x_access_token_secret,
        );

        let user = client
            .get_user_info_by_id("1851820330513473536")
            .await
            .unwrap();
        println!("{user:?}");
    }

    #[ignore]
    #[tokio::test]
    async fn test_follow_user() {
        let (x_consumer_key, x_consumer_secret, x_access_token, x_access_token_secret) =
            get_secrets();
        let base_url = "https://api.twitter.com/2".to_string();
        let client = TwitterClient::new(
            base_url,
            x_consumer_key,
            x_consumer_secret,
            x_access_token,
            x_access_token_secret,
        );

        let res = client
            .follow_user("1852012860596981761", "1851820330513473536")
            .await
            .unwrap();
        println!("{res:?}");
    }
}
