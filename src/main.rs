use std::{net::SocketAddr, str::FromStr};

use bitcoin::{OutPoint, Script, Transaction, TxIn, TxOut};
use clap::Parser;
use jsonrpc::{arg, Client};
use serde::{Deserialize, Serialize};

#[derive(Parser)]
struct Flags {
    #[clap(long)]
    server_addr: SocketAddr,
    #[clap(long)]
    name: String,
    #[clap(long)]
    password: String,
}

#[derive(Deserialize, Debug)]
pub struct VbAvailable {
    rulename: Option<i32>,
}

#[derive(Deserialize, Debug)]
#[serde(tag = "transaction")]
pub struct Tx {
    data: String,
    txid: String,
    hash: String,
    depends: Vec<i32>,
    fee: i32,
    sigops: i32,
    weight: i32,
}

#[derive(Deserialize, Debug)]
pub struct CoinbaseAuxValues {
    key: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct GetBlockTemplateReturn {
    capabilities: Vec<String>,
    version: i32,
    rules: Vec<String>,
    vbavailable: Option<VbAvailable>,
    vbrequired: i32,
    previousblockhash: String,
    transactions: Vec<Tx>,
    coinbaseaux: Option<CoinbaseAuxValues>,
    coinbasevalue: i32,
    longpollid: String,
    target: String,
    mintime: i128,
    mutable: Vec<String>,
    noncerange: String,
    sigoplimit: i32,
    sizelimit: i32,
    weightlimit: i32,
    curtime: i128,
    bits: String,
    height: i32,
    default_witness_commitment: String,
}

#[derive(Serialize)]
struct GetBlockTemplateParams {
    rules: Vec<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let flags = Flags::parse();

    let client = Client::simple_http("127.0.0.1", Some(flags.name), Some(flags.password))
        .expect("client creation error");

    let params = GetBlockTemplateParams {
        rules: vec!["segwit".to_string()],
    };

    let args = vec![arg(params)];

    let request = client.build_request("getblocktemplate", &args);

    let response = client.send_request(request).unwrap();

    let template: GetBlockTemplateReturn = response.result().unwrap();

    println!("{:#?}", template.transactions.get(0).unwrap());

    Ok(())
}

pub async fn create_coinbase(
    public_key_hash: String,
    block_height: i32,
    value: u64,
) -> Transaction {
    let outpoint = OutPoint::null();

    let signature = format!(
        "{}{}",
        hex::encode(block_height.to_string().as_bytes()),
        hex::encode(b"pickaxe-miner")
    );

    let script_sig = format!("{}{}", signature.bytes().len(), signature);

    let input = TxIn {
        previous_output: outpoint,
        script_sig: Script::from_str(&script_sig).expect("coinbase script sig creation error"),
        sequence: u32::max_value(),
        ..Default::default()
    };

    let script_pubkey = format!("76a914{}88ac", public_key_hash);

    let output = TxOut {
        value,
        script_pubkey: Script::from_str(&script_pubkey).expect("coinbase script pubkey failed"),
    };

    let tx = Transaction {
        version: 1,
        lock_time: 0,
        input: vec![input],
        output: vec![output],
    };

    tx
}
