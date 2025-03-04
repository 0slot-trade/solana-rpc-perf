# What's RPC Node Speed Test
RPC Node Speed Test is a command-line tool for testing and comparing RPC node speeds to facilitate selecting the optimal node.

# Installation Guide

## Prerequisites

- Developed in Rust - search for Rust installation guide.
- Git installed on your computer (optional, but recommended for cloning the repository).

## Installation Steps

1. **Obtain the source code:**
   - **Option 1: Clone the repository using Git**
     - Open a terminal or command prompt.
     - Run the following command: `git clone https://github.com/0slot-trade/solana-rpc-perf.git`
     - This will create a directory named `solana-rpc-perf` containing the source code.
   - **Option 2: Download the repository as a ZIP file**
     - Go to [https://github.com/0slot-trade/solana-rpc-perf](https://github.com/0slot-trade/solana-rpc-perf).
     - Click on the "Code" button.
     - Select "Download ZIP."
     - Extract the downloaded ZIP file to a directory of your choice.

## Build
On the shell, cd to the source directory, and then run:
```
cargo build
```

## Run
```
RUST_BACKTRACE=1 RUST_LOG=info ./target/debug/perf --name-0 other --grpc-url-0 http://<endpoint0> --x-token-0 <x-token> --name-1 me --grpc-url-1 <endpoint1>
```
## gRPC Program Invocation
```
# Usage: perf --name-0 <NAME_0> --grpc-url-0 <GRPC_URL_0> --name-1 <NAME_1> --grpc-url-1 <GRPC_URL_1>
./target/debug/perf --name-0 lon --grpc-url-0 http://10.0.0.10:10000 \
--name-1 fr --grpc-url-1 http://10.0.0.11:10000
```

## RPC Program Invocation
```
# Usage: perf_rpc --name-0 <NAME_0> --websocket-url-0 <WEBSOCKET_URL_0> \
--name-1 <NAME_1> --websocket-url-1 <WEBSOCKET_URL_1>
./target/debug/perf_rpc --name-0 tianyi_fr --websocket-url-0 ws://10.0.0.10:10000 \
--name-1 quiknode --websocket-url-1 wss://methodical-cool-replica.solana-mainnet.quiknode.pro/xxxxxxxxxxxxx
```
