use std::collections::VecDeque;

use crate::environment;

mod constants;
use constants::*;

use web3::contract::Contract;
use web3::futures::{future, StreamExt};
use web3::types::{Address, BlockNumber, Bytes, FilterBuilder, Log, U256, U64};

extern crate hex;

// TODO: Compare sum between intervals
// TODO: Notify on telegram if any intervals are breached

pub async fn poll() -> web3::contract::Result<()> {
    let url = environment::get_value("INFURA");
    let transport = web3::transports::WebSocket::new(&url).await?;
    let web3 = web3::Web3::new(transport);
    let token_eth_pair_address: Address = TOKEN_ETH_PAIR_ADDRESS.parse().unwrap();

    let _token_eth_pair_instance = Contract::from_json(
        web3.eth(),
        token_eth_pair_address,
        include_bytes!("../build/IUniswapV2Pair.abi"),
    )?;

    println!("Commencing logger");
    println!("-------------------");
    println!("Token name    : {}", TOKEN_NAME);
    println!("Token address : 0x{}", TOKEN_ADDRESS);
    println!("-------------------");

    let mut buy_logs: VecDeque<MinimalTx> = VecDeque::new();
    let mut sell_logs: VecDeque<MinimalTx> = VecDeque::new();

    add_past_transactions(&mut buy_logs, &mut sell_logs, &web3).await?;
    subscribe(&mut buy_logs, &mut sell_logs, &web3).await?;
    get_unique_interval_combinations();
    Ok(())
}

// 
async fn add_past_transactions(
    buy_logs: &mut VecDeque<MinimalTx>,
    sell_logs: &mut VecDeque<MinimalTx>,
    web3: &web3::Web3<web3::transports::WebSocket>,
) -> web3::contract::Result<()> {
    println!("1. Querying past events...");
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
            push_log(buy_logs, sell_logs, log);
        }
        from_block += U64::from(NUM_BLOCKS_PER_QUERY);
        curr_block = web3.eth().block_number().await?;
    }
    Ok(())
}

async fn subscribe(
    buy_logs: &mut VecDeque<MinimalTx>,
    sell_logs: &mut VecDeque<MinimalTx>,
    web3: &web3::Web3<web3::transports::WebSocket>,
) -> web3::contract::Result<()> {
    println!("2. Listening for new events...");
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
        push_log(buy_logs, sell_logs, log);
        future::ready(())
    })
    .await;
    Ok(())
}

fn push_log(buy_logs: &mut VecDeque<MinimalTx>, sell_logs: &mut VecDeque<MinimalTx>, log: Log) {
    let Bytes(data) = log.data;
    let uni_sold: U256 = data[..32].into();
    let uni_bought: U256 = data[64..96].into();
    let mut latest_tx = MinimalTx {
        hash: log.transaction_hash.unwrap(),
        block: log.block_number.unwrap(),
        qty: U256::from(0),
    };

    if uni_sold > U256::from(0) {
        latest_tx.qty = uni_sold;
        add_and_pop(sell_logs, latest_tx);
    } else {
        latest_tx.qty = uni_bought;
        add_and_pop(buy_logs, latest_tx);
    }
}

fn add_and_pop(logs: &mut VecDeque<MinimalTx>, minimal_tx: MinimalTx) {
    logs.push_back(minimal_tx);
    // Remove transactions that fall outside of latest block number - largest interval
    loop {
        let first_block = logs.front().unwrap().block;
        let last_block = logs.back().unwrap().block;
        let block_diff = last_block - first_block;
        if block_diff > U64::from(*INTERVALS.last().unwrap()) {
            logs.pop_front();
        } else {
            break;
        }
    }
}

// fn get_sum_for_each_interval(logs: &VecDeque<MinimalTx>) -> [U256; 3] {
//     let mut sums: [U256; 3] = [U256::from(0); 3];
//     // i is for interating logs
//     // j is for iterating intervals
//     let mut i = logs.len() - 2;
//     let mut j = 0;
//     let last = logs.back().unwrap();
//     let last_block = last.block;
//     let mut sum = last.qty;
//     while i >= 0 && j < INTERVALS.len() {
//         let curr_block = logs[i].block;
//         if last_block - curr_block < U64::from(*INTERVALS.last().unwrap()) {
            
//         }


//         i -= 1;
//     }
//     sums
// }

fn get_unique_interval_combinations() -> VecDeque<(u64, u64)> {
    let mut i: u64 = 0;
    let mut j: u64 = 1;
    let mut combinations: VecDeque<(u64, u64)> = VecDeque::new();
    while i < NUM_INTERVALS && j < NUM_INTERVALS && i < j {
        combinations.push_back((i, j));
        j += 1;
        if j == NUM_INTERVALS {
            i += 1;
            j = i + 1;
        }
    }
    combinations
}