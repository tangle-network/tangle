#!/bin/bash

# Script to generate weights for Tangle network pallets
set -e

# Hardcoded benchmark parameters
steps=10
repeat=2

# List of pallets and their corresponding folder names
pallets=(pallet_airdrop_claims pallet_credits pallet_multi_asset_delegation pallet_rewards pallet_tangle_lst pallet_services pallet_tangle_lst)
folders=(claims credits multi-asset-delegation rewards services tangle-lst)

# Generate weights for testnet runtime
echo "[testnet] Generating weights with steps: $steps, repeat: $repeat"
for i in "${!pallets[@]}"; do
  pallet=${pallets[$i]}
  echo "[testnet] Benchmarking $pallet"
  
  ./target/release/tangle benchmark pallet \
    --chain=dev \
    --wasm-execution=compiled \
    --pallet="$pallet" \
    --extrinsic='*' \
    --steps="$steps" \
    --repeat="$repeat" \
    --template=./.maintain/frame-weights-template.hbs \
    --output="./pallets/${folders[$i]}/src/weights.rs"
done

echo "Weight generation complete!"
