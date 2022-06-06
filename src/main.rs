use clap::Parser;
use std::net::SocketAddr;
use tarpc::{client, context, tokio_serde::formats::Json};

#[derive(Parser)]
struct Flags {
    #[clap(long)]
    server_addr: SocketAddr,
    #[clap(long)]
    name: String,
}

#[tarpc::service]
pub trait BitcoinRpc {
    async fn get_block_template(template_request: String) -> String;
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let flags = Flags::parse();

    let transport = tarpc::serde_transport::tcp::connect(flags.server_addr, Json::default);

    let client = BitcoinRpcClient::new(client::Config::default(), transport.await?).spawn();

    let _block_template = async move {
        tokio::select! {template = client.get_block_template(context::current(), String::from("{}")) => {template}}
    }
    .await;

    Ok(())
}
