use std::collections::VecDeque;

use crate::environment;

mod constants;
use constants::*;

use web3::contract::Contract;
use web3::futures::{future, StreamExt};
use web3::types::{Address, BlockNumber, Bytes, Filter, FilterBuilder, Log, H256, U256, U64};

extern crate hex;

pub async fn poll() -> web3::contract::Result<()> {
    let url = environment::get_value("INFURA");
    let transport = web3::transports::WebSocket::new(&url).await?;
    let web3 = web3::Web3::new(transport);
    let token_eth_pair_address: Address = TOKEN_ETH_PAIR_ADDRESS.parse().unwrap();

    let token_eth_pair_instance = Contract::from_json(
        web3.eth(),
        token_eth_pair_address,
        include_bytes!("../build/IUniswapV2Pair.abi"),
    )?;

    println!("Commencing logger");
    println!("-------------------");
    println!("Token name    : {}", TOKEN_NAME);
    println!("Token address : 0x{}", TOKEN_ADDRESS);
    println!("-------------------");

    let (mut buy_moving_sum, mut sell_moving_sum) = init_moving_sums();
    add_past_transactions_to_moving_sums(&mut buy_moving_sum, &mut sell_moving_sum, &web3).await?;
    subscribe(&mut buy_moving_sum, &mut sell_moving_sum, &web3).await?;
    Ok(())
}

// Initialises an empty vector of MovingSums
// Intervals are 10, 100 and 1000
fn init_moving_sums() -> (MovingSum, MovingSum) {
    println!("1. Initialise moving sums");
    let buy_moving_sum: MovingSum = MovingSum {
        interval: *INTERVALS.last().unwrap(),
        logs: VecDeque::new(),
    };
    let sell_moving_sum: MovingSum = MovingSum {
        interval: *INTERVALS.last().unwrap(),
        logs: VecDeque::new(),
    };
    (buy_moving_sum, sell_moving_sum)
}

// Fill moving sums with historical transactions
async fn add_past_transactions_to_moving_sums(
    buy_moving_sum: &mut MovingSum,
    sell_moving_sum: &mut MovingSum,
    web3: &web3::Web3<web3::transports::WebSocket>,
) -> web3::contract::Result<()> {
    println!("2. Querying past events...");

    let token_eth_pair_address: Address = TOKEN_ETH_PAIR_ADDRESS.parse().unwrap();
    let mut curr_block = web3.eth().block_number().await?;
    let mut from_block = curr_block - U64::from(*INTERVALS.last().unwrap());

    // Queries past transactions in batches of QUERY_BLOCK_INTERVAL
    while from_block < curr_block {
        println!(".");
        let mut to_block = from_block + NUM_BLOCKS_PER_QUERY;
        // To make sure to_block doesn't ever exceed the latest block number. Is this needed though? Hmm..
        if to_block > curr_block {
            to_block = curr_block;
        }
        let filter = FilterBuilder::default()
            .address(vec![token_eth_pair_address])
            .from_block(BlockNumber::from(from_block))
            .to_block(BlockNumber::from(to_block))
            .topics(
                Some(vec![EVENT_SIGNATURE_HASH.parse().unwrap()]),
                None,
                None,
                None,
            )
            .build();
        let events = web3.eth().logs(filter).await?;
        for log in events {    
            push_log_to_ms(buy_moving_sum, sell_moving_sum, log);
        }
        from_block += U64::from(NUM_BLOCKS_PER_QUERY);
        curr_block = web3.eth().block_number().await?;
    }
    Ok(())
}


async fn subscribe(
    buy_moving_sum: &mut MovingSum,
    sell_moving_sum: &mut MovingSum,
    web3: &web3::Web3<web3::transports::WebSocket>
) -> web3::contract::Result<()> {
    println!("3. Listening for new events...");

    let token_eth_pair_address: Address = TOKEN_ETH_PAIR_ADDRESS.parse().unwrap();
    let filter = FilterBuilder::default()
        .address(vec![token_eth_pair_address])
        .topics(
            Some(vec![EVENT_SIGNATURE_HASH.parse().unwrap()]),
            None,
            None,
            None,
        )
        .build();
    let sub = web3.eth_subscribe().subscribe_logs(filter).await?;
    sub.for_each(|log| {
        let log = log.unwrap();
        push_log_to_ms(buy_moving_sum, sell_moving_sum, log);
        println!("{:?}", sell_moving_sum);
        future::ready(())
    })
    .await;
    Ok(())
}

fn push_log_to_ms(buy_moving_sum: &mut MovingSum, sell_moving_sum: &mut MovingSum, log: Log) {
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
        add(sell_moving_sum, latest_tx);
    } else {
        latest_tx.qty = uni_bought;
        add(buy_moving_sum, latest_tx);
    }

}

fn add(moving_sum: &mut MovingSum, minimal_tx: MinimalTx) {
    moving_sum.logs.push_back(minimal_tx);

    // Remove transactions that fall outside of latest block number - interval
    loop {
        let first_block = moving_sum.logs.front().unwrap().block;
        let last_block = moving_sum.logs.back().unwrap().block;
        let block_diff = last_block - first_block;
        if block_diff > U64::from(moving_sum.interval) {
            moving_sum.logs.pop_front();
        } else {
            break;
        }
    }
}
