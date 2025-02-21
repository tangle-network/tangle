#!/bin/bash

# Script to generate weights for Tangle network pallets
set -e

# Hardcoded benchmark parameters
steps=10
repeat=10

# List of pallets to benchmark
pallets=(
  pallet_multi_asset_delegation
  pallet_tangle_lst
  pallet_services
  pallet_rewards
)

# Generate weights for testnet runtime
echo "[testnet] Generating weights with steps: 10, repeat: 10"
for pallet in "${pallets[@]}"; do
  echo "[testnet] Benchmarking $pallet"
  
  ./target/release/tangle benchmark pallet \
    --chain=dev \
    --execution=wasm \
    --wasm-execution=compiled \
    --pallet="$pallet" \
    --extrinsic='*' \
    --steps="$steps" \
    --repeat="$repeat" \
    --template=./.maintain/frame-weights-template.hbs \
    --output="./runtime/testnet/src/weights/${pallet/pallet_/}.rs"
done

# Generate weights for mainnet runtime
echo "[mainnet] Generating weights with steps: 10, repeat: 10"
for pallet in "${pallets[@]}"; do
  echo "[mainnet] Benchmarking $pallet"
  
  ./target/release/tangle benchmark pallet \
    --chain=dev \
    --execution=wasm \
    --wasm-execution=compiled \
    --pallet="$pallet" \
    --extrinsic='*' \
    --steps="$steps" \
    --repeat="$repeat" \
    --template=./.maintain/frame-weights-template.hbs \
    --output="./runtime/mainnet/src/weights/${pallet/pallet_/}.rs"
done

echo "Weight generation complete!"
