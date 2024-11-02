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

#[cfg(test)]
mod tests {
    use crate::{db::types::Embedding, openai::OpenAIClient};

    #[ignore]
    #[tokio::test]
    async fn test_get_text_embedding() {
        let base_url = "https://api.openai.com/v1".to_string();
        let open_ai_api_key = "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".to_string();
        let client = OpenAIClient::new(open_ai_api_key, base_url);

        let mut emb = client
            .get_text_embedding("ethereum is the best")
            .await
            .unwrap();
        let emb1 = Embedding::new(emb.data.pop().unwrap().embedding);

        let mut emb = client.get_text_embedding("ethereum sucks").await.unwrap();
        let emb2 = Embedding::new(emb.data.pop().unwrap().embedding);

        let sim = emb1.cosine_similarity(&emb2);
        println!("sim: {sim}");
    }
}
