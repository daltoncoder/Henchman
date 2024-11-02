use anyhow::{anyhow, Result};
use reqwest::Client;

use self::api_types::ApiResponse;
mod api_types;

pub struct HyperbolicClient {
    base_url: String,
    hyperbolic_api_key: String,
    client: Client,
}

impl HyperbolicClient {
    pub fn new(hyperbolic_api_key: String, base_url: String) -> Self {
        let client = Client::new();
        Self {
            base_url,
            hyperbolic_api_key,
            client,
        }
    }

    pub async fn generate_text(&self, context: &str, prompt: &str) -> Result<ApiResponse> {
        let url = format!("{}/chat/completions", self.base_url);

        // TODO: add these params to the config
        let body = serde_json::json!({
            "messages": [
                {
                    "role": "system",
                    "content": context,
                },
                {
                    "role": "user",
                    "content": prompt
                }
            ],
            "model": "meta-llama/Meta-Llama-3.1-70B-Instruct",
            "max_tokens": 512,
            "temperature": 1,
            "top_p": 0.95,
            "top_k": 40,
            "stream": false,
        });

        self.client
            .post(&url)
            .header("Content-Type", "application/json")
            .header(
                "Authorization",
                format!("Bearer {}", self.hyperbolic_api_key),
            )
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
    use crate::hyperbolic::HyperbolicClient;

    #[ignore]
    #[tokio::test]
    async fn test_generate_text() {
        let base_url = "https://api.hyperbolic.xyz/v1".to_string();
        let hyperbolic_api_key = "".to_string();
        let client = HyperbolicClient::new(hyperbolic_api_key, base_url);

        let res = client
            .generate_text(
                "hey shitalik, when does ethereum go to zero?",
                "write a witty response to this tweet",
            )
            .await
            .unwrap();

        println!("{res:?}");
    }
}
