mod uniswap;
mod environment;

#[tokio::main]
async fn main() -> web3::contract::Result<()> {
    let _ = env_logger::try_init();
    environment::load();
    uniswap::poll().await?;
    Ok(())
}
