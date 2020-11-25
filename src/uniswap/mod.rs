use std::collections::VecDeque;

use crate::environment;

mod constants;
use constants::*;

use web3::contract::Contract;
use web3::futures::{future, StreamExt};
use web3::types::{Address, BlockNumber, Bytes, FilterBuilder, Log, U256, U64};

extern crate hex;

// TODO List
// 1. Notify on telegram if any intervals are breached

// TODO Enhancement List
// a. Calculate sum while tracking past transactions and subscribing
// b. `get_sums_for_each_interval` should be running separately outside of subscribe otherwise it will only trigger every time subscribe receives a new event that passes the filter.
// c. Upgrade `get_sums_for_each_interval` to use latest blockchain block instead of latest transaction recorded in logs

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
    Ok(())
}

// Queries past buy / sell transactions
async fn add_past_transactions(
    buy_logs: &mut VecDeque<MinimalTx>,
    sell_logs: &mut VecDeque<MinimalTx>,
    web3: &web3::Web3<web3::transports::WebSocket>,
) -> web3::contract::Result<()> {
    println!("1. Querying past events...");
    let token_eth_pair_address: Address = TOKEN_ETH_PAIR_ADDRESS.parse().unwrap();
    let mut curr_block = web3.eth().block_number().await?;
    let mut from_block = curr_block - U64::from(*INTERVALS.last().unwrap());

    // Runs web3 queries in batches of QUERY_BLOCK_INTERVAL (100) transactions
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
            parse_and_add(buy_logs, sell_logs, log);
        }
        from_block += U64::from(NUM_BLOCKS_PER_QUERY);
        curr_block = web3.eth().block_number().await?;
    }
    Ok(())
}

// Listens for new blocks
async fn subscribe(
    buy_logs: &mut VecDeque<MinimalTx>,
    sell_logs: &mut VecDeque<MinimalTx>,
    web3: &web3::Web3<web3::transports::WebSocket>,
) -> web3::contract::Result<()> {
    println!("2. Listening for new events...");
    let pairs = get_interval_index_pairs();
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
        parse_and_add(buy_logs, sell_logs, log);
        // TODO task a
        let buy_sums = get_sums_for_each_interval(buy_logs);
        let sell_sums = get_sums_for_each_interval(sell_logs);

        // println! {"{:?}", buy_logs};
        // println! {"{:?}", sell_logs};
        compare_same_type("buy", &pairs, &buy_sums);
        compare_same_type("sell", &pairs, &sell_sums);
        compare_diff_type(&buy_sums, &sell_sums);
        future::ready(())
    })
    .await;
    Ok(())
}

// Parse log's data and adds it into their respective logs
fn parse_and_add(buy_logs: &mut VecDeque<MinimalTx>, sell_logs: &mut VecDeque<MinimalTx>, log: Log) {
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

// Adds the log to the front of logs and pops those that are no longer relevant
fn add_and_pop(logs: &mut VecDeque<MinimalTx>, minimal_tx: MinimalTx) {
    // All new transactions are added to the front of the queue
    logs.push_front(minimal_tx);
    // Remove transactions that fall outside of latest block number - largest interval
    loop {
        let latest = logs.front().unwrap().block;
        let earliest = logs.back().unwrap().block;
        let block_diff = latest - earliest;
        if block_diff > U64::from(*INTERVALS.last().unwrap()) {
            logs.pop_back();
        } else {
            break;
        }
    }
}

// TODO task b
// Calculates the total volume for each interval
fn get_sums_for_each_interval(logs: &VecDeque<MinimalTx>) -> [U256; NUM_INTERVALS] {
    let mut sums: [U256; NUM_INTERVALS] = [U256::from(0); NUM_INTERVALS];
    // i is for interating logs
    // j is for iterating intervals
    let mut i = 1;
    let mut j = 0;
    let latest = logs[0];
    // TODO task c
    // Note that this "latest block" is referring to the last block with a buy / sell transaction
    // instead of the latest blockchain block!
    let latest_block = latest.block;
    let mut sum = latest.qty;
    println!("{}, {} = {:?}", i, j, sum);
    while i < logs.len() && j < INTERVALS.len() {
        let curr = logs[i];
        let curr_block = curr.block;
        let block_diff = latest_block - curr_block;
        if block_diff <= U64::from(INTERVALS[j]) {
            sum += curr.qty;
            i += 1;
            println!("{}, {} = {:?}", i, j, sum);
        } else {
            sums[j] = sum;
            j += 1;
        }
    }
    sums[j] = sum;
    sums
}

// Generate all possible pairs (2 elements) for the intervals
fn get_interval_index_pairs() -> VecDeque<(usize, usize)> {
    let mut i: usize = 0;
    let mut j: usize = 1;
    let mut combinations: VecDeque<(usize, usize)> = VecDeque::new();
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

// Compare between buy intervals e.g. 10 block buy vs 100 block buy
fn compare_same_type(
    verb: &str,
    pairs: &VecDeque<(usize, usize)>,
    sums: &[web3::types::U256; NUM_INTERVALS],
) {
    // Compare buy pairs
    for (a, b) in pairs.iter() {
        // Note that I'm only printing if LHS > RHS because LHS is always < RHS base on how I generate my pairs
        // LHS is always the shorter time frame e.g. 10 blocks
        // RHS is always the longer time frame e.g. 100 blocks
        // If it doesn't print, you can assume the opposite is true i.e. LHS < RHS
        let a_blocks = INTERVALS[*a];
        let b_blocks = INTERVALS[*b];
        let a_vol = sums[*a];
        let b_vol = sums[*b] / U256::from(10).pow(U256::from(b - a));
        if a_vol > b_vol {
            println!(
                "{0} block {1} ({2} {3}) > {4} block buy (averaged to {0} blocks, {5} {3})",
                a_blocks,
                verb,
                a_vol / U256::from(10).pow(U256::from(TOKEN_DECIMALS)),
                TOKEN_NAME,
                b_blocks,
                b_vol / U256::from(10).pow(U256::from(TOKEN_DECIMALS))
            );
        }
    }
}

// Compare between buy and sell intervals e.g. 10 block buy vs 10 block sell
fn compare_diff_type(
    buy_sums: &[web3::types::U256; NUM_INTERVALS],
    sell_sums: &[web3::types::U256; NUM_INTERVALS],
) {
    for i in 0..NUM_INTERVALS {
        // Note that I'm only printing if buy is > sell because I'm only interested in that lol.
        // If it doesn't print, you can assume the opposite is true i.e. sell > buy.
        if buy_sums[i] > sell_sums[i] {
            println!(
                "{0} block buy ({1} {2}) > {0} block sell ({3} {2})",
                INTERVALS[i],
                buy_sums[i] / U256::from(10).pow(U256::from(TOKEN_DECIMALS)),
                TOKEN_NAME,
                sell_sums[i] / U256::from(10).pow(U256::from(TOKEN_DECIMALS))
            );
        }
    }
}
