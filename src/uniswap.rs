extern crate hex;

use super::environment;

use web3::contract::Contract;
use web3::futures::{future, StreamExt};
use web3::types::{Address, FilterBuilder, Bytes, U256};

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
        // let H256(from) = log.topics[1];
        // let H256(to) = log.topics[2];
        // log::info!("0x{} -> 0x{} ", hex::encode(&from[12..]), hex::encode(&to[12..]));


        let Bytes(data) = log.data;
        let uni_in: U256 = data[..32].into();
        let eth_in: U256 = data[32..64].into();
        let uni_out: U256 = data[64..96].into();
        let eth_out: U256 = data[96..].into();

        //  / U256::from(10).pow(U256::from(18))
        if uni_in > U256::from(0) {
            log::info!("{:?} UNI -> {:?} ETH", uni_in, eth_out);
        } else {
            log::info!("{:?} ETH -> {:?} UNI", eth_in, uni_out);
        }

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