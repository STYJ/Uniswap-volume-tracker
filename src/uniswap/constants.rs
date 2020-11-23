use std::collections::VecDeque;
use web3::types::{H256, U256, U64};

pub const TOKEN_NAME: &str = "UNI";
pub const _TOKEN_DECIMALS: i32 = 18;
pub const TOKEN_ADDRESS: &str = "1f9840a85d5aF5bf1D1762F925BDADdC4201F984";
pub const UNI_ROUTER_ADDRESS: &str = "7a250d5630B4cF539739dF2C5dAcb4c659F2488D";
pub const TOKEN_ETH_PAIR_ADDRESS: &str = "d3d2E2692501A5c9Ca623199D38826e513033a17";
pub const EVENT_SIGNATURE_HASH: &str = "d78ad95fa46c994b6551d0da85fc275fe613ce37657fb8d5e3d130840159d822";

pub const INTERVALS: [u64; 3] = [10, 100, 1000];  // ~2.1 mins, ~21 mins, 3.6 hours
pub const NUM_BLOCKS_PER_QUERY: i8 = 100;

#[derive(Debug)]
pub struct MovingSum {
    pub interval: u64,
    pub logs: VecDeque<MinimalTx>,  // Not to be confused with `log::info!...`
}

#[derive(Debug)]
pub struct MinimalTx {
    pub hash: H256,
    pub block: U64,
    pub qty: U256,
}