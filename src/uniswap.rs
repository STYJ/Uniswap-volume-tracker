extern crate hex;

use super::environment;

use web3::contract::Contract;
use web3::futures::{future, StreamExt};
use web3::types::{Address, FilterBuilder, Bytes, U256, Log};

use std::collections::VecDeque;

#[derive(Debug)]
struct MovingSum {
    interval: u64,
    logs: VecDeque<Log>,  // Not to be confused with `log`
    sum: U256,
}

pub async fn poll() -> web3::contract::Result<()> {
    let url = environment::get_value("ALCHEMY");
    let transport = web3::transports::WebSocket::new(&url).await?;
    let web3 = web3::Web3::new(transport);

    // TODO: Refactor constants into separate file
    let token_name = "UNI";
    let token_decimals = 18;
    let token_address: Address = "1f9840a85d5aF5bf1D1762F925BDADdC4201F984".parse().unwrap();
    let _uni_router_address: Address = "7a250d5630B4cF539739dF2C5dAcb4c659F2488D".parse().unwrap();
    let token_eth_pair_address: Address = "d3d2E2692501A5c9Ca623199D38826e513033a17".parse().unwrap();
    let event_signature_hash = "d78ad95fa46c994b6551d0da85fc275fe613ce37657fb8d5e3d130840159d822";
    let token_eth_pair_instance = Contract::from_json(
        web3.eth(),
        token_eth_pair_address,
        include_bytes!("./build/IUniswapV2Pair.abi"),
    )?;
    
    // I am tracking 10 blocks (~ 2 mins), 100 blocks (~ 20 mins) & 1000 blocks (~ 36 hours) moving sums (ms)
    // If you want to compare the ms of 10 blocks w/ 1000 blocks, you need to divide
    // The sum tracked for 1000 blocks by 100 to get average 10 block sums in the past 1000 blocks
    // TODO: Potentially vert into Vector of moving sums for cleaner code.
    let mut ten_blocks_buy_ms = MovingSum {
        interval: 10,
        logs: VecDeque::new(),
        sum: U256::from(0),
    };
    let mut ten_blocks_sell_ms = MovingSum {
        interval: 10,
        logs: VecDeque::new(),
        sum: U256::from(0),
    };


    let filter = FilterBuilder::default()
        .address(vec![token_eth_pair_instance.address()])
        .topics(
            Some(vec![event_signature_hash
                .parse()
                .unwrap()]),
            None,
            None,
            None,
        )
        .build();
    log::info!("Commencing logger");
    log::info!("-------------------");
    log::info!("Token name    : {}", token_name);
    log::info!("Token address : 0x{}", token_address);
    log::info!("-------------------");


    let sub = web3.eth_subscribe().subscribe_logs(filter).await?;
    sub.for_each(|log| {
        let log = log.unwrap();
        // TODO: 1 VecDeque for log for buys
        // TODO: 1 VecDequeu for log for sells
        // TODO: 1 U256 for sum buys
        // TODO: 1 U256 for sum sells


        
        log::info!("Tx hash : {:?}", log.transaction_hash.unwrap());
        
        let Bytes(data) = log.data;
        let uni_sold: U256 = data[..32].into();
        let uni_bought: U256 = data[64..96].into();

        let action: String = if uni_sold > U256::from(0) { "Sold    ".into() } else { "Bought  ".into() };
        let qty = if uni_sold > U256::from(0) { uni_sold } else { uni_bought };
        // Shadowing variable to make code more readable.
        let qty = qty / U256::from(10).pow(U256::from(token_decimals));
        log::info!("{}: {:?} UNI", action, qty);

        future::ready(())
    })
    .await;
    Ok(())
}