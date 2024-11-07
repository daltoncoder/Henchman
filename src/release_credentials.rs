use std::time::Duration;

use reqwest::Client;

use crate::encumber::AccountDetails;

pub async fn timelock(account_details: AccountDetails, unlock_time: u64, rpc_url: String) {
    // todo!()
    let client = Client::new();

    let one_hour = Duration::from_secs(3600);
    loop {
        tokio::time::sleep(one_hour).await;

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
                todo!()
            }
            Err(err) => {
                println!("Unable to contact RPC endoint {err} trying again in an hour")
            }
        }
    }
}

fn get_rpc_request() -> String {
    serde_json::json!({
        "jsonrpc":"2.0","method":"eth_getBlockByNumber","params":["latest", true],"id":1
    })
    .to_string()
}
