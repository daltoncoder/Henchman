use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct ApiResponse<T> {
    pub data: T,
}

#[derive(Deserialize, Debug)]
pub struct MentionsResponse {
    pub data: Vec<Mention>,
    pub meta: Meta,
}

#[derive(Deserialize, Debug)]
pub struct Mention {
    pub id: String,
    pub author_id: String,
    pub edit_history_tweet_ids: Vec<String>,
    pub text: String,
}

#[derive(Deserialize, Debug)]
pub struct Meta {
    pub newest_id: String,
    pub oldest_id: String,
    pub result_count: u32,
}

#[derive(Deserialize, Debug)]
pub struct Tweet {
    pub text: String,
    pub id: String,
    pub author_id: String,
    pub edit_history_tweet_ids: Vec<String>,
}

#[derive(Deserialize, Debug)]
pub struct TweetsResponse {
    pub data: Vec<Tweet>,
    pub meta: Meta,
}

#[derive(Deserialize, Debug)]
pub struct SentTweet {
    pub text: String,
    pub id: String,
    pub edit_history_tweet_ids: Vec<String>,
}

#[derive(Deserialize, Debug)]
pub struct User {
    pub id: String,
    pub name: String,
    pub username: String,
}
