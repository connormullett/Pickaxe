use std::net::SocketAddr;

use clap::Parser;
use jsonrpc::{arg, Client};
use serde::Serialize;

#[derive(Parser)]
struct Flags {
    #[clap(long)]
    server_addr: SocketAddr,
    #[clap(long)]
    name: String,
    #[clap(long)]
    password: String,
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

    println!("response {:#?}", response);

    Ok(())
}
