extern crate hex;

use super::environment;

use web3::contract::Contract;
use web3::futures::{future, StreamExt};
use web3::types::{Address, FilterBuilder, Bytes, H256};

pub async fn poll() -> web3::contract::Result<()> {
    let url = environment::get_value("ALCHEMY");
    let transport = web3::transports::WebSocket::new(&url).await?;
    let web3 = web3::Web3::new(transport);

    let _uni_address: Address = "1f9840a85d5aF5bf1D1762F925BDADdC4201F984".parse().unwrap();
    let _uni_router_address: Address = "7a250d5630B4cF539739dF2C5dAcb4c659F2488D".parse().unwrap();
    let uni_eth_pair_address: Address = "d3d2E2692501A5c9Ca623199D38826e513033a17".parse().unwrap();
    let uni_eth_pair_instance = Contract::from_json(
        web3.eth(),
        uni_eth_pair_address,
        include_bytes!("./build/IUniswapV2Pair.abi"),
    )?;

    let filter = FilterBuilder::default()
        .address(vec![uni_eth_pair_instance.address()])
        .topics(
            Some(vec!["d78ad95fa46c994b6551d0da85fc275fe613ce37657fb8d5e3d130840159d822"
                .parse()
                .unwrap()]),
            None,
            None,
            None,
        )
        .build();

    let sub = web3.eth_subscribe().subscribe_logs(filter).await?;

    sub.for_each(|log| {
        let log = log.unwrap();
        log::info!("Transaction hash: {:?}", log.transaction_hash.unwrap());
        let H256(from) = log.topics[1];
        let H256(to) = log.topics[2];
        // I don't really need to know To and From. I just need to be able to parse the data.
        // amount0 = Uni, amount1 = Token
        // either 1, 4 or 2, 3.
        // 1, 4 means sell uni for eth
        // 2, 3 means buy uni with eth 
        log::info!("0x{} -> 0x{} ", hex::encode(&from[12..]), hex::encode(&to[12..]));
        let Bytes(data) = log.data;
        log::info!("{}", hex::encode(&data));
        future::ready(())
    })
    .await;

    // let filter = web3.eth_filter().create_logs_filter(filter).await?;

    // let logs_stream = filter.stream(time::Duration::from_secs(5));
    // futures::pin_mut!(logs_stream);
    
    // loop {
    //     let log = logs_stream.next().await.unwrap();
    //     log::info!("got log: {:?}", log.unwrap());
    // }
    Ok(())
}