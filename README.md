# Build
cargo build

# Run
```
RUST_BACKTRACE=1 RUST_LOG=info ./target/debug/perf --name-0 other --grpc-url-0 http://<endpoint0> --x-token-0 <x-token> --name-1 me --grpc-url-1 <endpoint1>
```
