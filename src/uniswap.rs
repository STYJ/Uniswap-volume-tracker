extern crate hex;

use super::environment;

use web3::contract::Contract;
use web3::futures::{future, StreamExt};
use web3::types::{Address, FilterBuilder, Bytes, H256, U256, U64};

use std::collections::VecDeque;

#[derive(Debug)]
struct MovingSum {
    interval: u64,
    logs: VecDeque<MinimalTx>,  // Not to be confused with `log::info!...`
    sum: U256,
}

#[derive(Debug)]
struct MinimalTx {
    hash: H256,
    block: U64,
    qty: U256,
}

pub async fn poll() -> web3::contract::Result<()> {
    let url = environment::get_value("INFURA");
    let transport = web3::transports::WebSocket::new(&url).await?;
    let web3 = web3::Web3::new(transport);

    // TODO: Refactor constants into separate file
    let token_name = "UNI";
    let _token_decimals = 18;
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
    println!("Commencing logger");
    println!("-------------------");
    println!("Token name    : {}", token_name);
    println!("Token address : 0x{}", token_address);
    println!("-------------------");

    let sub = web3.eth_subscribe().subscribe_logs(filter).await?;
    sub.for_each(|log| {
        let log = log.unwrap();

        let Bytes(data) = log.data;
        let uni_sold: U256 = data[..32].into();
        let uni_bought: U256 = data[64..96].into();
        let mut latest_tx = MinimalTx{
            hash: log.transaction_hash.unwrap(),
            block: log.block_number.unwrap(),
            qty: U256::from(0),
        };

        if uni_sold > U256::from(0) {
            latest_tx.qty = uni_sold;
            update_ms(&mut ten_blocks_sell_ms, latest_tx);
        } else {
            latest_tx.qty = uni_bought;
            update_ms(&mut ten_blocks_buy_ms, latest_tx);
        }
        future::ready(())
    })
    .await;
    Ok(())
}

fn update_ms(moving_sum: &mut MovingSum, minimal_tx: MinimalTx) {
    // Destructure before moving minimal_tx into queue
    let MinimalTx{hash: _, block: last_block, qty} = minimal_tx;

    // Add log to queue
    moving_sum.logs.push_back(minimal_tx);

    // Update sum
    moving_sum.sum += qty;

    // Calculate if the diff in block number between
    // last log and first log is > interval
    // if diff > interval, pop first and repeat until diff is < interval
    loop {
        let first = moving_sum.logs.front_mut().unwrap();
        let first_block = first.block;
        let first_qty = first.qty;
        let block_diff = last_block - first_block;
        if block_diff > U64::from(moving_sum.interval) {
            moving_sum.logs.pop_front();
            moving_sum.sum -= first_qty;
        } else {
            break;
        }
    };
}