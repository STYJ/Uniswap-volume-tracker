use web3::types::{H256, U256, U64};

pub const TOKEN_NAME: &str = "STAKE";
pub const TOKEN_DECIMALS: i8 = 18;
pub const TOKEN_ADDRESS: &str = "0ae055097c6d159879521c384f1d2123d1f195e6";
pub const _UNI_ROUTER_ADDRESS: &str = "7a250d5630B4cF539739dF2C5dAcb4c659F2488D";
pub const TOKEN_ETH_PAIR_ADDRESS: &str = "3b3d4eefdc603b232907a7f3d0ed1eea5c62b5f7";
pub const EVENT_SIGNATURE_HASH: &str = "d78ad95fa46c994b6551d0da85fc275fe613ce37657fb8d5e3d130840159d822";

pub const NUM_INTERVALS: usize = 4;
pub const INTERVALS: [u64; NUM_INTERVALS] = [10, 100, 1000, 10000];  // ~2.1 mins, ~21 mins, 3.6 hrs, 36 hours
pub const NUM_BLOCKS_PER_QUERY: i32 = 1000;

#[derive(Debug, Copy, Clone)]
pub struct MinimalTx {
    pub hash: H256,
    pub block: U64,
    pub qty: U256,
}