mod uniswap;
mod environment;

extern crate pretty_env_logger;
#[macro_use] extern crate log;

#[tokio::main]
async fn main() -> web3::contract::Result<()> {
    pretty_env_logger::init();
    environment::load();
    uniswap::poll().await?;
    Ok(())
}
