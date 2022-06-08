use std::{
    net::SocketAddr,
    str::FromStr,
    time::{SystemTime, UNIX_EPOCH},
};

use bitcoin::{
    consensus::{deserialize, serialize},
    hashes::{hex::FromHex, Hash},
    BlockHash, BlockHeader, OutPoint, Script, Transaction, TxIn, TxMerkleNode, TxOut,
};
use clap::Parser;
use jsonrpc::{arg, Client};
use num_bigint::BigUint;
use rs_merkle::{algorithms::Sha256, Hasher, MerkleTree};
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

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
pub struct VbAvailable {
    rulename: Option<i32>,
}

#[allow(dead_code)]
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

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
pub struct CoinbaseAuxValues {
    key: Option<String>,
}

#[allow(dead_code)]
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
    coinbasevalue: u64,
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
    height: u64,
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

    let pub_key_hash = String::from("93ce48570b55c42c2af816aeaba06cfee1224fae");
    let mut transactions =
        vec![create_coinbase(pub_key_hash, template.height, template.coinbasevalue).await];

    let mut transaction_data: Vec<Transaction> = template
        .transactions
        .iter()
        .map(|tx| {
            let tx_hex = Vec::from_hex(&tx.data).unwrap();
            let transaction = deserialize(&tx_hex).unwrap();
            transaction
        })
        .collect();

    transactions.append(&mut transaction_data);

    let leaves: Vec<[u8; 32]> = transactions
        .iter()
        .map(|x| Sha256::hash(&serialize(x)))
        .collect();

    let tree = MerkleTree::<Sha256>::from_leaves(&leaves);
    let root = tree.root().unwrap();

    let start = SystemTime::now();
    let time = start
        .duration_since(UNIX_EPOCH)
        .expect("time went backwards")
        .as_secs();

    let header = BlockHeader {
        version: template.version,
        prev_blockhash: BlockHash::from_hex(&template.previousblockhash).unwrap(),
        merkle_root: TxMerkleNode::from_hex(&hex::encode(root)).unwrap(),
        time: time.try_into().unwrap(),
        bits: u32::from_str_radix(&template.bits, 16).unwrap(),
        nonce: 1,
    };

    let target_bytes = BlockHeader::u256_from_compact_target(header.bits).to_be_bytes();
    let target_value = BigUint::from_bytes_be(&target_bytes);

    let hash = header.block_hash().into_inner();
    let hash_value = BigUint::from_bytes_be(&hash);

    if hash_value < target_value {
        println!("found block at {} with target {}", hash_value, target_value);
    } else {
        println!(
            "didnt find block at {} with target {}",
            hash_value, target_value
        );
    }

    Ok(())
}

pub async fn create_coinbase(
    public_key_hash: String,
    block_height: u64,
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
