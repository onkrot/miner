use chrono::serde::ts_seconds;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct SimpleBlock {
    pub height: u32,
    pub hash: String,
    #[serde(with = "ts_seconds")]
    pub time: DateTime<Utc>,
    pub main_chain: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LatestBlock {
    pub hash: String,
    #[serde(with = "ts_seconds")]
    pub time: DateTime<Utc>,
    pub block_index: u32,
    pub height: u32,
    pub tx_indexes: Vec<u64>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UnspentOutput {
    pub tx_hash: String,
    pub tx_index: u64,
    pub tx_output_n: u32,
    pub script: String,
    pub value: u64,
    pub value_hex: String,
    pub confirmations: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Address {
    pub hash160: String,
    pub address: String,
    pub n_tx: u32,
    pub total_received: u64,
    pub total_sent: u64,
    pub final_balance: u64,
    #[serde(rename = "txs")]
    pub transactions: Vec<Transaction>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Input {
    pub prev_out: Option<TX>,
    #[serde(rename = "script")]
    pub script_sig: String,
    pub sequence: u64,
    pub witness: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TX {
    pub n: u32,
    pub value: u64,
    #[serde(rename = "addr")]
    pub address: Option<String>,
    pub tx_index: u64,
    #[serde(rename = "type")]
    pub tx_type: u32,
    pub script: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Output {
    pub n: u32,
    pub value: u64,
    #[serde(rename = "addr")]
    pub address: Option<String>,
    pub tx_index: u64,
    pub script: String,
    pub spent: bool,
    pub addr_tag_link: Option<String>,
    pub addr_tag: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Transaction {
    pub hash: String,
    #[serde(rename = "ver")]
    pub version: u32,
    pub vin_sz: u32,
    pub vout_sz: u32,
    pub lock_time: u64,
    pub size: u64,
    pub relayed_by: String,
    pub block_height: Option<u32>,
    pub tx_index: u64,
    pub inputs: Vec<Input>,
    #[serde(rename = "out")]
    pub outputs: Vec<Output>,
    #[serde(default)]
    pub double_spend: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Block {
    pub hash: String,
    #[serde(rename = "ver")]
    pub version: u32,
    #[serde(rename = "prev_block")]
    pub previous_block: String,
    #[serde(rename = "mrkl_root")]
    pub merkle_root: String,
    #[serde(with = "ts_seconds")]
    pub time: DateTime<Utc>,
    pub bits: u32,
    pub nonce: u32,
    pub fee: u64,
    pub n_tx: u32,
    pub size: u32,
    pub block_index: u32,
    pub main_chain: bool,
    pub height: u32,
    pub received_time: Option<DateTime<Utc>>,
    pub relayed_by: Option<String>,
    #[serde(rename = "tx")]
    pub transactions: Vec<Transaction>,
}
