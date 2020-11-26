# What is uniswap volume tracker about?

The uniswap volume tracker tracks the buy and sell volumes for a specific token pair across different block intervals. An interval is analogous to a moving average. For example, there are 4 default intervals: 10, 100, 1000, 10000 blocks. The algorithm sums the total buy volume and the total sell volume of each interval and compares them. Here are a few examples:

- *1000 block sell (40848 STAKE) > 10000 block sell (averaged to 1000 blocks, 19174 STAKE)* - this shows that there is a higher sell volume in the shorter time frame (1000 blocks) compared to the longer time frame (10000 blocks). Volume comparisons can be used for different analysis so make of that what you will.
- *10000 block buy (219045 STAKE) > 10000 block sell (191749 STAKE)* - this shows that there are more tokens bought than tokens sold in the 10000 block time frame. Multiple repeats of this pattern indicates a potential upwards trend.

# Prerequisites

You need to have [rust](https://www.rust-lang.org/tools/install) and [git](https://git-scm.com/book/en/v2/Getting-Started-Installing-Git) installed.

# Setup

1. Clone the repo and navigate to it.
2. Create a `.env` file in the root folder.
3. Create a key pair in your `.env` file for INFURA i.e.
```
INFURA=wss://mainnet.infura.io/ws/v3/xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
```
4. Run the command `cargo run` in your terminal. This will download the relevant dependencies, compile and run.

# How to track a different token?

1. Open the file `src/uniswap/constants.rs`
2. The 4 constants you want to modify are `TOKEN_NAME`, `TOKEN_DECIMALS`, `TOKEN_ADDRESS` and `TOKEN_ETH_PAIR_ADDRESS`.
3. For `TOKEN_ADDRESS` and `TOKEN_ETH_PAIR_ADDRESS`, remove the `0x` prefix.
4. Once modified, save the file and re-run `cargo run`.

# How to change the intervals?

1. Open the file `src/uniswap/constants.rs`
2. The 2 constants you want to modify are `NUM_INTERVALS` and `INTERVALS`.
3. Ensure that if your `NUM_INTERVAL` says 6, that you should have 6 elements in `INTERVALS`.
4. Once modified, save the file and re-run `cargo run`.

# Things to note

- You can use whatever node provider you want, just update the `.env` file. You may also need to update lines 19-20 of `src/uniswap/mod.rs`. For example:
```
# .env
ALCHEMY=wss://eth-mainnet.ws.alchemyapi.io/v2/xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx

# src/uniswap/mod.rs
// ...
pub async fn poll() -> web3::contract::Result<()> {
    let url = environment::get_value("ALCHEMY");
    let transport = web3::transports::WebSocket::new(&url).await?;
// ...
```
- If possible, avoid changing `NUM_BLOCKS_PER_QUERY` because sometimes if you try to query past events across too many blocks in 1 API call, it will fail. This is usually because of a size restriction on the response from the node provider.
- The `get_sums_for_each_interval` function in `src/uniswap/mod.rs` only runs when the algorithm detects a new valid transaction as opposed to running every block. ctrl (cmd if you're on mac) + shift + f and search for "TODO task a" for more information.
- The `get_sums_for_each_interval` function in `src/uniswap/mod.rs` considers the last block with a valid transaction as the `latest block` instead of using the latest blockchain block. ctrl (cmd if you're on mac) + shift + f and search for "TODO task b" for more information.