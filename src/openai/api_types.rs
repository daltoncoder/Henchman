use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct ApiResponse {
    pub object: String,
    pub data: Vec<EmbeddingData>,
    pub model: String,
    pub usage: Usage,
}

#[derive(Deserialize, Debug)]
pub struct EmbeddingData {
    pub object: String,
    pub index: usize,
    pub embedding: Vec<f64>,
}

#[derive(Deserialize, Debug)]
pub struct Usage {
    pub prompt_tokens: usize,
    pub total_tokens: usize,
}
