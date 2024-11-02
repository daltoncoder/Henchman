use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct ApiResponse {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<Choice>,
    pub usage: Usage,
}

#[derive(Deserialize, Debug)]
pub struct Choice {
    pub index: usize,
    pub message: Message,
    pub finish_reason: String,
    pub logprobs: Option<Vec<f64>>,
}

#[derive(Deserialize, Debug)]
pub struct Message {
    pub role: String,
    pub content: String,
}

#[derive(Deserialize, Debug)]
pub struct Usage {
    pub prompt_tokens: usize,
    pub total_tokens: usize,
    pub completion_tokens: usize,
}
