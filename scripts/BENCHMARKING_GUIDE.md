## How to run benchmarking

This a simple guide that will outline the required steps to run the benchmarking scripts.

### Prerequisites

- Install the `chain-spec-builder` tool:

```sh
cargo install --git https://github.com/paritytech/polkadot-sdk --force --locked staging-chain-spec-builder
```

- Install the `frame-omni-bencher` tool:

```sh
cargo install --git https://github.com/paritytech/polkadot-sdk --force --locked frame-omni-bencher
```

### Generate weights

Build the testnet runtime:
```sh
cargo build --release --features testnet,runtime-benchmarks
```

To generate the weights for the pallets, you can use the `generate-weights.sh` script.

```sh
bash ./scripts/generate-weights.sh
```

### References

- https://docs.polkadot.com/develop/parachains/testing/benchmarking/
- https://docs.polkadot.com/develop/parachains/deployment/generate-chain-specs/