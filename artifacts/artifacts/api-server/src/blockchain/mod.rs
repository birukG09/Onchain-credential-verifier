use crate::error::AppError;
use reqwest::Client;
use serde_json::{json, Value};
use std::time::Duration;

pub struct BlockchainClient {
    client: Client,
    rpc_url: String,
    contract_address: String,
}

impl BlockchainClient {
    pub fn new(rpc_url: &str, contract_address: &str) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to build HTTP client");

        Self {
            client,
            rpc_url: rpc_url.to_string(),
            contract_address: contract_address.to_string(),
        }
    }

    pub fn is_configured(&self) -> bool {
        self.contract_address != "0x0000000000000000000000000000000000000000"
            && !self.rpc_url.is_empty()
    }

    async fn call_rpc(&self, method: &str, params: Value) -> Result<Value, AppError> {
        let body = json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
            "id": 1
        });

        let response = self
            .client
            .post(&self.rpc_url)
            .json(&body)
            .send()
            .await
            .map_err(|e| AppError::Blockchain(e.to_string()))?;

        let result: Value = response
            .json()
            .await
            .map_err(|e| AppError::Blockchain(e.to_string()))?;

        if let Some(err) = result.get("error") {
            return Err(AppError::Blockchain(err.to_string()));
        }

        Ok(result["result"].clone())
    }

    pub async fn check_connection(&self) -> Result<String, AppError> {
        let result = self.call_rpc("net_version", json!([])).await?;
        Ok(result.to_string())
    }

    pub async fn is_credential_valid(&self, token_id: &str) -> Result<bool, AppError> {
        if !self.is_configured() {
            return Ok(false);
        }

        let selector = "0xd05fd813";
        let padded_token = format!("{:0>64}", token_id.trim_start_matches("0x"));
        let data = format!("{}{}", selector, padded_token);

        let result = self
            .call_rpc(
                "eth_call",
                json!([
                    {
                        "to": self.contract_address,
                        "data": data
                    },
                    "latest"
                ]),
            )
            .await?;

        let raw = result.as_str().unwrap_or("0x");
        Ok(raw != "0x" && raw != "0x0000000000000000000000000000000000000000000000000000000000000000")
    }

    pub async fn get_chain_id(&self) -> Result<u64, AppError> {
        let result = self.call_rpc("eth_chainId", json!([])).await?;
        let hex = result.as_str().unwrap_or("0x0").trim_start_matches("0x");
        u64::from_str_radix(hex, 16).map_err(|e| AppError::Blockchain(e.to_string()))
    }
}
