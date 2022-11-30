#!/bin/sh

# The following line ensure we run from the project root
PROJECT_ROOT=$(git rev-parse --show-toplevel)
cd "$PROJECT_ROOT"

echo "****************** GENERATE RAW CHAINSPEC ******************"
./target/release/tangle-parachain build-spec --disable-default-bootnode --chain tangle-dev --raw > ./chainspecs/tangle-parachain-chainspec.json
./target/release/tangle-parachain export-genesis-state --chain ./chainspecs/tangle-parachain-chainspec.json > ./chainspecs/tangle-parachain-genesis-state
./target/release/tangle-parachain export-genesis-wasm --chain ./chainspecs/tangle-parachain-chainspec.json > ./chainspecs/tangle-parachain-genesis-wasm