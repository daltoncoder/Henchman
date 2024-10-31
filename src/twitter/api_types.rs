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

#[derive(Deserialize, Debug)]
pub struct TweetResponse {
    data: Tweet,
}

#[derive(Deserialize, Debug)]
pub struct Tweet {
    text: String,
    id: String,
    edit_history_tweet_ids: Vec<String>,
}
