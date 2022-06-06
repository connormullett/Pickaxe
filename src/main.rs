#![allow(dead_code)]
use std::net::SocketAddr;

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
pub struct Transaction {
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
    transactions: Vec<Transaction>,
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
