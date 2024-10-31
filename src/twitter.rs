// Client that makes all requests to the twitter client

use anyhow::{anyhow, Result};
use reqwest_oauth1::{Client, DefaultSM, OAuthClientProvider, Secrets, Signer};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct MentionsResponse {
    data: Vec<Mention>,
    meta: MentionsMeta,
}

#[derive(Deserialize, Debug)]
pub struct Mention {
    id: String,
    edit_history_tweet_ids: Vec<String>,
    text: String,
}

#[derive(Deserialize, Debug)]
pub struct MentionsMeta {
    newest_id: String,
    oldest_id: String,
    result_count: u32,
}

pub struct TwitterClient<'a> {
    client: Client<Signer<'a, Secrets<'a>, DefaultSM>>,
    base_url: String,
    //x_consumer_key: String,
    //x_consumer_secret: String,
    //x_access_token: String,
    //x_access_token_secret: String,
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
            //x_consumer_key,
            //x_consumer_secret,
            //x_access_token,
            //x_access_token_secret,
        }
    }

    pub async fn post_tweet() {
        todo!()
    }

    pub async fn get_mentions(&self, user_id: String) -> Result<MentionsResponse> {
        let url = format!("{}/users/{user_id}/mentions", self.base_url);

        let res = self
            .client
            .get(url)
            .send()
            .await
            .unwrap()
            .text()
            .await
            .unwrap();

        println!("{res}");

        todo!()
        //self.client
        //    .get(url)
        //    .send()
        //    .await
        //    .map_err(|e| anyhow!("{e:?}"))?
        //    .json::<MentionsResponse>()
        //    .await
        //    .map_err(|e| anyhow!("{e:?}"))
    }

    pub async fn get_users_tweets() {
        todo!()
    }

    pub async fn reply_to_tweet() {
        todo!()
    }

    pub async fn get_user_id() {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use crate::twitter::MentionsMeta;

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
    async fn test_x() {
        let val = "{\"data\":[{\"edit_history_tweet_ids\":[\"1852057407389569179\"],\"text\":\"@developerg23272 can god create a stone that he cannot lift?\",\"id\":\"1852057407389569179\"},{\"edit_history_tweet_ids\":[\"1852054615954432343\"],\"text\":\"@developerg23272 why is ethereum better than solana?\",\"id\":\"1852054615954432343\"},{\"edit_history_tweet_ids\":[\"1852053986368516233\"],\"text\":\"@developerg23272 word on the street is that eth is going to 0 soon\",\"id\":\"1852053986368516233\"},{\"edit_history_tweet_ids\":[\"1852052791901958530\"],\"text\":\"@developerg23272 who do you think is better with the ladies, you or vitalik??\",\"id\":\"1852052791901958530\"},{\"edit_history_tweet_ids\":[\"1852052314925814023\"],\"text\":\"@developerg23272 I heard Solana has better economics than Ethereum\",\"id\":\"1852052314925814023\"},{\"edit_history_tweet_ids\":[\"1852052116946186508\"],\"text\":\"@developerg23272 do you even code bro?\",\"id\":\"1852052116946186508\"}],\"meta\":{\"result_count\":6,\"newest_id\":\"1852057407389569179\",\"oldest_id\":\"1852052116946186508\"}}";
        let json: MentionsMeta = serde_json::from_str(val).expect("JSON was not well-formatted");

        println!("{json:?}");
    }
}
