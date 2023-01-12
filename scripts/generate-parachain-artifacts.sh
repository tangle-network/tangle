#!/usr/bin/env bash
set -e
pushd .

# The following line ensure we run from the project root
PROJECT_ROOT=$(git rev-parse --show-toplevel)
cd "$PROJECT_ROOT"

echo "****************** GENERATE RAW CHAINSPEC ******************"
./target/release/tangle-parachain build-spec --disable-default-bootnode --chain tangle --raw > ./chainspecs/tangle-parachain.json
./target/release/tangle-parachain export-genesis-state --chain ./chainspecs/tangle-parachain.json > ./chainspecs/tangle-genesis-state
./target/release/tangle-parachain export-genesis-wasm --chain ./chainspecs/tangle-parachain.json > ./chainspecs/tangle-genesis-wasm

popd