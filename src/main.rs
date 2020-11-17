use std::time;
use web3::contract::{Contract, Options};
use web3::futures::StreamExt;
use web3::types::{Address, FilterBuilder};

#[tokio::main]
async fn main() -> web3::contract::Result<()> {
    let _ = env_logger::try_init();
    let transport = web3::transports::WebSocket::new("wss://mainnet.infura.io/ws/v3/b955965c36374388a7c874e91b511d79").await?;
    let web3 = web3::Web3::new(transport);
    log::info!("{:?}", web3);

    // let address: Address = "52bc44d5378309EE2abF1539BF71dE1b7d7bE3b5".parse().unwrap();
    // let balance = web3.eth().balance(address, Some(BlockNumber::Latest)).await?;
    // log::info!("{:?}", balance);

    // let me: Address = "267be1C1D684F78cb4F6a176C4911b741E4Ffdc0".parse().unwrap();
    let uni_contract_address: Address = "1f9840a85d5aF5bf1D1762F925BDADdC4201F984".parse().unwrap();
    let contract = Contract::from_json(
        web3.eth(),
        uni_contract_address,
        include_bytes!("./build/IERC20.abi"),
    )?;
    // let balance: U256 = contract.query("balanceOf", (me,), None, Options::default(), None).await?;
    // log::info!("{:?}", balance);

    let filter = FilterBuilder::default()
        .address(vec![contract.address()])
        .topics(
            Some(vec!["ddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef"
                .parse()
                .unwrap()]),
            None,
            None,
            None,
        )
        .build();

    let filter = web3.eth_filter().create_logs_filter(filter).await?;

    let logs_stream = filter.stream(time::Duration::from_secs(1));
    futures::pin_mut!(logs_stream);

    let log = logs_stream.next().await.unwrap();
    println!("got log: {:?}", log);

    Ok(())
}