use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct MentionsResponse {
    data: Vec<Mention>,
    meta: Meta,
}

#[derive(Deserialize, Debug)]
pub struct Mention {
    id: String,
    author_id: String,
    edit_history_tweet_ids: Vec<String>,
    text: String,
}

#[derive(Deserialize, Debug)]
pub struct Meta {
    newest_id: String,
    oldest_id: String,
    result_count: u32,
}

#[derive(Deserialize, Debug)]
pub struct TweetResponse {
    data: Tweet,
}

#[derive(Deserialize, Debug)]
pub struct Tweet {
    text: String,
    id: String,
    author_id: String,
    edit_history_tweet_ids: Vec<String>,
}

#[derive(Deserialize, Debug)]
pub struct TweetsResponse {
    data: Vec<Tweet>,
    meta: Meta,
}
