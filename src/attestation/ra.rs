#![allow(dead_code)]

use axum::{extract::State, response::IntoResponse};
use reqwest::StatusCode;
use tracing::{debug, error, info};
use ed25519_dalek::Signer;

use crate::attestation::server::{QuoteResponse, AppState};

use configfs_tsm::create_quote;

pub const QUOTE_REPORT_DATA_OFFSET: usize = 368;
pub const QUOTE_REPORT_DATA_LENGTH: usize = 64;

pub async fn ra_get_quote(State(state): State<AppState>) -> impl IntoResponse {

    let sign_data = state.twitter_username;

    debug!("QUOTE : report_data token = {}", sign_data);

    let signature = state.keypair.sign(sign_data.as_bytes());
    let public_key = hex::encode(state.keypair.verifying_key());

    match create_quote(signature.to_bytes()) {
        Ok(quote_byte) => {
            info!("QUOTE : success generating quote");
            
            let quote_hex = hex::encode(quote_byte);
            return axum::Json(QuoteResponse {
                status: StatusCode::OK.to_string(),
                public_key,
                quote: quote_hex,
            })
        }

        Err(err) => {
            error!("QUOTE : error generating quote, {:?}", err);
            
            return axum::Json(QuoteResponse {
                status: StatusCode::INTERNAL_SERVER_ERROR.to_string(),
                public_key,
                quote: err.to_string(),
            })
        }
    };
}

