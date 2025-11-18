#!/bin/bash

# Script to generate weights for Tangle network pallets
set -e

# Hardcoded benchmark parameters
steps=10
repeat=2

# List of pallets and their corresponding folder names
pallets=(pallet_airdrop_claims pallet_credits pallet_multi_asset_delegation pallet_rewards pallet_services)
folders=(claims credits multi-asset-delegation rewards services)

chain-spec-builder create --runtime target/release/wbuild/tangle-testnet-runtime/tangle_testnet_runtime.wasm default

# Generate weights for testnet runtime
echo "[testnet] Generating weights with steps: $steps, repeat: $repeat"
for i in "${!pallets[@]}"; do
  pallet=${pallets[$i]}
  echo "[testnet] Benchmarking $pallet"
  
  frame-omni-bencher v1 benchmark pallet \
    --chain=chain_spec.json \
    --pallet="$pallet" \
    --extrinsic='*' \
    --steps="$steps" \
    --repeat="$repeat" \
    --template=./.maintain/frame-weights-template.hbs \
    --output="./pallets/${folders[$i]}/src/weights.rs"
done

echo "Weight generation complete!"

echo "Cleaning up ..."

rm -rf chain_spec.json

echo "Done!"