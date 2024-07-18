use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    method: String,
    params: Vec<Value>,
    id: u64,
}

#[derive(Debug, Deserialize)]
struct Withdrawal {
    #[serde(alias = "validatorIndex")]
    validator_index: String,
}

pub fn get_highest_validator_index(node_url: &str) -> Result<u64> {
    let block_number = get_latest_block_number(node_url)?;
    let block = get_block_details(node_url, &block_number)?;
    let highest_index = extract_highest_validator_index(block)?;
    Ok(highest_index)
}

fn get_latest_block_number(node_url: &str) -> Result<String> {
    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "eth_blockNumber".to_string(),
        params: vec![],
        id: 1,
    };

    let client = reqwest::blocking::Client::new();
    let response: Value = client.post(node_url).json(&request).send()?.json()?;
    let block_number = response["result"]
        .as_str()
        .context("Failed to get block number")?
        .to_string();
    Ok(block_number)
}

fn get_block_details(node_url: &str, block_number: &str) -> Result<Value> {
    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "eth_getBlockByNumber".to_string(),
        params: vec![Value::String(block_number.to_string()), Value::Bool(true)],
        id: 1,
    };

    let client = reqwest::blocking::Client::new();
    let response = client.post(node_url).json(&request).send()?.json()?;
    Ok(response)
}

fn extract_highest_validator_index(block: Value) -> Result<u64> {
    let withdrawals = block["result"]["withdrawals"]
        .as_array()
        .context("No withdrawals in block")?;

    withdrawals
        .iter()
        .map(|m| {
            let withdrawal: Withdrawal = serde_json::from_value(m.clone())?;
            let index =
                u64::from_str_radix(withdrawal.validator_index.trim_start_matches("0x"), 16)?;
            Ok(index)
        })
        .collect::<Result<Vec<u64>>>()?
        .into_iter()
        .max()
        .context("No valid validator indices found")
}
