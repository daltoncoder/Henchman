use std::fmt::Display;

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
pub struct ApiResponse<T> {
    pub data: T,
}

#[derive(Deserialize, Debug)]
pub struct MentionsResponse {
    pub data: Vec<Tweet>,
    pub includes: IncludesUsers,
    pub meta: Meta,
}

#[derive(Deserialize, Debug)]
pub struct Meta {
    pub newest_id: String,
    pub oldest_id: String,
    pub result_count: u32,
}

#[derive(Deserialize, Debug)]
pub struct Tweet {
    pub id: String,
    pub author_id: String,
    pub text: String,
    pub edit_history_tweet_ids: Vec<String>,
    pub created_at: String,
    pub username: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct TweetsResponse {
    pub data: Vec<Tweet>,
    pub meta: Meta,
}

#[derive(Serialize, Deserialize, Debug)]
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

#[derive(Debug, Deserialize)]
pub struct TimelineResponse {
    pub data: Vec<TimelineTweet>,
    pub includes: IncludesUsers,
    pub meta: TimelineMeta,
}

#[derive(Debug, Deserialize)]
pub struct TimelineTweet {
    pub edit_history_tweet_ids: Vec<String>,
    pub article: Option<Article>,
    pub text: String,
    pub author_id: String,
    pub id: String,
    pub username: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Article {
    pub title: String,
}

#[derive(Debug, Deserialize)]
pub struct IncludesUsers {
    pub users: Vec<User>,
}

#[derive(Debug, Deserialize)]
pub struct TimelineMeta {
    pub next_token: String,
    pub result_count: u32,
    pub newest_id: String,
    pub oldest_id: String,
}

#[derive(Debug, Deserialize)]
pub struct FollowData {
    pub following: bool,
    pub pending_follow: bool,
}

impl Display for Tweet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(username) = &self.username {
            write!(f, "A tweet on my timeline from @{username}: {}", self.text)
        } else {
            write!(f, "New tweet on my timeline: {}", self.text)
        }
    }
}

impl Display for TimelineTweet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(username) = &self.username {
            write!(f, "A tweet on my timeline from @{username}: {}", self.text)
        } else {
            write!(f, "New tweet on my timeline: {}", self.text)
        }
    }
}

impl Display for SentTweet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "A tweet from us: {}", self.text)
    }
}
