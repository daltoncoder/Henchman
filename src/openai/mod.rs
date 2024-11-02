use anyhow::{anyhow, Result};
use reqwest::Client;

use self::api_types::ApiResponse;
mod api_types;

pub struct OpenAIClient {
    base_url: String,
    open_ai_api_key: String,
    client: Client,
}

impl OpenAIClient {
    pub fn new(open_ai_api_key: String, base_url: String) -> Self {
        let client = Client::new();
        Self {
            base_url,
            open_ai_api_key,
            client,
        }
    }

    pub async fn get_text_embedding(&self, text: &str) -> Result<ApiResponse> {
        let url = format!("{}/embeddings", self.base_url);

        let body = serde_json::json!({
            "input": text,
            "model": "text-embedding-3-small"
        });

        self.client
            .post(&url)
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", self.open_ai_api_key))
            .body(body.to_string())
            .send()
            .await
            .map_err(|e| anyhow!("{e:?}"))?
            .json::<ApiResponse>()
            .await
            .map_err(|e| anyhow!("{e:?}"))
    }
}
