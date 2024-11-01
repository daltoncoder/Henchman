// Client that makes all requests to the twitter client

use anyhow::{anyhow, Result};
use reqwest_oauth1::{Client, DefaultSM, OAuthClientProvider, Secrets, Signer};

mod api_types;
use api_types::MentionsResponse;

use crate::twitter::api_types::ApiResponse;

use self::api_types::{SentTweet, Tweet, TweetsResponse, User};

pub struct TwitterClient<'a> {
    client: Client<Signer<'a, Secrets<'a>, DefaultSM>>,
    base_url: String,
}

impl<'a> TwitterClient<'a> {
    pub fn new(
        url: String,
        x_consumer_key: String,
        x_consumer_secret: String,
        x_access_token: String,
        x_access_token_secret: String,
    ) -> Self {
        let client = reqwest::Client::new();

        let secrets = Secrets::new(x_consumer_key, x_consumer_secret)
            .token(x_access_token, x_access_token_secret);

        let client = client.oauth1(secrets);

        Self {
            client,
            base_url: url,
        }
    }

    pub async fn post_tweet(&self, content: String) -> Result<SentTweet> {
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

    pub async fn get_mentions(&self, user_id: String) -> Result<MentionsResponse> {
        let url = format!(
            "{}/users/{user_id}/mentions?tweet.fields=author_id",
            self.base_url
        );

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

    pub async fn get_tweet(&self, tweet_id: String) -> Result<Tweet> {
        let url = format!("{}/tweets/{tweet_id}?tweet.fields=author_id", self.base_url);

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

    pub async fn get_user_tweets(&self, user_id: String) -> Result<TweetsResponse> {
        let url = format!(
            "{}/users/{user_id}/tweets?tweet.fields=author_id",
            self.base_url
        );
        self.client
            .get(url)
            .send()
            .await
            .map_err(|e| anyhow!("{e:?}"))?
            .json::<TweetsResponse>()
            .await
            .map_err(|e| anyhow!("{e:?}"))
    }

    pub async fn reply_to_tweet(&self, content: String, tweet_id: String) -> Result<SentTweet> {
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

    pub async fn get_user_id(&self, username: String) -> Result<User> {
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
}

#[cfg(test)]
mod tests {

    use super::TwitterClient;

    #[tokio::test]
    async fn test_get_mentions() {
        let base_url = "https://api.twitter.com/2".to_string();
        let x_consumer_key = "0TTOpmPT9ZjdlVWh5Ba1krstm".to_string();
        let x_consumer_secret = "SCKhSvsF5EvuREb5PRaVrzKFcywhuBzWlAMnZSUkJmX5UmHxBE".to_string();
        let x_access_token = "1852012860596981761-sVrVOcEMuskF6mCpbjwPbIZyu2wbkX".to_string();
        let x_access_token_secret = "woK0aqO6YNB37A1E98vzl3rn3dBLUowxphiGcse6pcipJ".to_string();
        let client = TwitterClient::new(
            base_url,
            x_consumer_key,
            x_consumer_secret,
            x_access_token,
            x_access_token_secret,
        );

        let mentions = client
            .get_mentions("1852012860596981761".to_string())
            .await
            .unwrap();
        println!("{mentions:?}");
    }

    #[tokio::test]
    async fn test_get_tweet() {
        let base_url = "https://api.twitter.com/2".to_string();
        let x_consumer_key = "0TTOpmPT9ZjdlVWh5Ba1krstm".to_string();
        let x_consumer_secret = "SCKhSvsF5EvuREb5PRaVrzKFcywhuBzWlAMnZSUkJmX5UmHxBE".to_string();
        let x_access_token = "1852012860596981761-sVrVOcEMuskF6mCpbjwPbIZyu2wbkX".to_string();
        let x_access_token_secret = "woK0aqO6YNB37A1E98vzl3rn3dBLUowxphiGcse6pcipJ".to_string();
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

    #[tokio::test]
    async fn test_get_user_tweets() {
        let base_url = "https://api.twitter.com/2".to_string();
        let x_consumer_key = "0TTOpmPT9ZjdlVWh5Ba1krstm".to_string();
        let x_consumer_secret = "SCKhSvsF5EvuREb5PRaVrzKFcywhuBzWlAMnZSUkJmX5UmHxBE".to_string();
        let x_access_token = "1852012860596981761-sVrVOcEMuskF6mCpbjwPbIZyu2wbkX".to_string();
        let x_access_token_secret = "woK0aqO6YNB37A1E98vzl3rn3dBLUowxphiGcse6pcipJ".to_string();
        let client = TwitterClient::new(
            base_url,
            x_consumer_key,
            x_consumer_secret,
            x_access_token,
            x_access_token_secret,
        );

        let tweets = client
            .get_user_tweets("1852012860596981761".to_string())
            .await
            .unwrap();
        println!("{tweets:?}");
    }

    #[tokio::test]
    async fn test_post_tweet() {
        let base_url = "https://api.twitter.com/2".to_string();
        let x_consumer_key = "0TTOpmPT9ZjdlVWh5Ba1krstm".to_string();
        let x_consumer_secret = "SCKhSvsF5EvuREb5PRaVrzKFcywhuBzWlAMnZSUkJmX5UmHxBE".to_string();
        let x_access_token = "1852012860596981761-sVrVOcEMuskF6mCpbjwPbIZyu2wbkX".to_string();
        let x_access_token_secret = "woK0aqO6YNB37A1E98vzl3rn3dBLUowxphiGcse6pcipJ".to_string();
        let client = TwitterClient::new(
            base_url,
            x_consumer_key,
            x_consumer_secret,
            x_access_token,
            x_access_token_secret,
        );

        let tweet = client.post_tweet("mic check 3".to_string()).await.unwrap();
        println!("{tweet:?}");
    }

    #[tokio::test]
    async fn test_reply_to_tweet() {
        let base_url = "https://api.twitter.com/2".to_string();
        let x_consumer_key = "0TTOpmPT9ZjdlVWh5Ba1krstm".to_string();
        let x_consumer_secret = "SCKhSvsF5EvuREb5PRaVrzKFcywhuBzWlAMnZSUkJmX5UmHxBE".to_string();
        let x_access_token = "1852012860596981761-sVrVOcEMuskF6mCpbjwPbIZyu2wbkX".to_string();
        let x_access_token_secret = "woK0aqO6YNB37A1E98vzl3rn3dBLUowxphiGcse6pcipJ".to_string();
        let client = TwitterClient::new(
            base_url,
            x_consumer_key,
            x_consumer_secret,
            x_access_token,
            x_access_token_secret,
        );

        let tweet = client
            .reply_to_tweet("oh really".to_string(), "1852054615954432343".to_string())
            .await
            .unwrap();
        println!("{tweet:?}");
    }

    #[tokio::test]
    async fn test_get_user_id() {
        let base_url = "https://api.twitter.com/2".to_string();
        let x_consumer_key = "0TTOpmPT9ZjdlVWh5Ba1krstm".to_string();
        let x_consumer_secret = "SCKhSvsF5EvuREb5PRaVrzKFcywhuBzWlAMnZSUkJmX5UmHxBE".to_string();
        let x_access_token = "1852012860596981761-sVrVOcEMuskF6mCpbjwPbIZyu2wbkX".to_string();
        let x_access_token_secret = "woK0aqO6YNB37A1E98vzl3rn3dBLUowxphiGcse6pcipJ".to_string();
        let client = TwitterClient::new(
            base_url,
            x_consumer_key,
            x_consumer_secret,
            x_access_token,
            x_access_token_secret,
        );

        let user = client
            .get_user_id("mavonclarksdale".to_string())
            .await
            .unwrap();
        println!("{user:?}");
    }
}
