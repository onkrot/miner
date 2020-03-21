use crate::api::structs::{Block, Transaction};
use serde_json::{Result, Value};

static BASE_URL: &str = "https://blockchain.info/";

fn call_api(resource: &str) -> std::result::Result<String, reqwest::Error> {
    let body = reqwest::blocking::get(&(BASE_URL.to_owned() + resource))?.text()?;

    Ok(body)
}

pub fn get_block(block_id: String, api_key: Option<String>) -> Result<Block> {
    let mut resource = "rawblock/".to_owned() + &block_id;
    if let Some(api_code) = api_key {
        resource += format!("?api_code={}", api_code).as_str();
    }
    let response = call_api(&resource);
    let v: Block = serde_json::from_str(&response.unwrap())?;
    Ok(v)
}

pub fn get_unconfirmed_tx(api_key: Option<String>) -> Result<Vec<Transaction>> {
    let mut resource = "unconfirmed-transactions?format=json".to_owned();
    if let Some(api_code) = api_key {
        resource += format!("?api_code={}", api_code).as_str();
    }
    let response = call_api(&resource).unwrap();
    let v: Value = serde_json::from_str(&response)?;
    let v = v["txs"].as_array().unwrap();
    let txs = v
        .iter()
        .map(|tx| serde_json::from_value(tx.clone()).unwrap())
        .collect();

    Ok(txs)
}
