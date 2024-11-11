use std::{num::ParseIntError, time::Duration};

use reqwest::Client;
use serde::Deserialize;

use crate::encumber::FullAccountDetails;

pub async fn timelock(account_details: FullAccountDetails, unlock_time: u64, rpc_url: String) {
    let client = Client::new();

    // Get the initial timestamp from ethereum
    let body: String = get_rpc_request();
    let resp: Response = client
        .post(&rpc_url)
        .header("Content-Type", "application/json")
        .body(body)
        .send()
        .await
        .expect("Unable to contact Ethereum for inital timestamp")
        .json()
        .await
        .expect("Unable to contact Ethereum");

    let unlock_timestamp = convert_hex_timestamp(&resp.result.timestamp).unwrap() + unlock_time;

    let wait_time = Duration::from_secs(600);
    loop {
        tokio::time::sleep(wait_time).await;

        let body: String = get_rpc_request();
        match client
            .post(&rpc_url)
            .header("Content-Type", "application/json")
            .body(body)
            .send()
            .await
        {
            Ok(resp) => {
                // parse .result.timestamp
                if let Ok(response) = resp.json::<Response>().await {
                    let timestamp =
                        convert_hex_timestamp(&response.result.timestamp).unwrap_or_default();

                    if timestamp > unlock_timestamp {
                        tracing::error!(
                            "######## Unlock Time is up printing Account Details ########"
                        );
                        tracing::error!("{:?}", account_details);
                        tracing::error!(
                            "############################################################"
                        );
                        break;
                    } else {
                        tracing::warn!(
                            "Not time to unlock Agent. Will unlock at {unlock_timestamp}"
                        );
                    }
                } else {
                    tracing::error!(
                        "Unable to parse rpc response to get timestamp trying again in 10 minutes"
                    );
                }
            }
            Err(err) => {
                tracing::warn!("Unable to contact RPC endoint {err} trying again in 10 minutes")
            }
        }
    }
}

fn get_rpc_request() -> String {
    serde_json::json!({
        "jsonrpc":"2.0","method":"eth_getBlockByNumber","params":["latest", false],"id":1
    })
    .to_string()
}

fn convert_hex_timestamp(timestamp: &str) -> Result<u64, ParseIntError> {
    let without_pre = timestamp.trim_start_matches("0x");

    u64::from_str_radix(without_pre, 16)
}

#[derive(Deserialize, Debug)]
struct Response {
    pub result: ResultJson,
}

#[derive(Deserialize, Debug)]
struct ResultJson {
    pub timestamp: String,
}

#[tokio::test]
async fn test() {
    let rpc_url = "https://rpc.ankr.com/eth".to_string();
    let client = Client::new();

    let body: String = get_rpc_request();

    let resp = client
        .post(&rpc_url)
        .header("Content-Type", "application/json")
        .body(body)
        .send()
        .await
        .unwrap();

    let response: Response = resp.json().await.unwrap();

    let without_pre = &response.result.timestamp.trim_start_matches("0x");
    println!("{without_pre}");
    let number = u64::from_str_radix(without_pre, 16).unwrap();

    println!("{number}");
}
