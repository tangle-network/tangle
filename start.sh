rm -rf /tmp/parachain

cargo b -p egg-collator -r 

./target/release/egg-collator build-spec --disable-default-bootnode --raw --chain egg-rococo > webb_rococo.json

./target/release/egg-collator build-spec --disable-default-bootnode --raw --chain webb_rococo.json > webb_rococo_raw.json

./target/release/egg-collator export-genesis-wasm --chain webb_rococo_raw.json > webb-rococo-4006-wasm

./target/release/egg-collator export-genesis-state --chain webb_rococo_raw.json > webb-rococo-4006-genesis

./target/release/egg-collator \
--alice \
--collator \
--force-authoring \
--chain webb_rococo_raw.json \
--base-path /tmp/parachain/alice \
--port 40333 \
--ws-port 9948 \
--rpc-port 9979 --rpc-cors all --discover-local \
--rpc-external --rpc-methods=unsafe \
-- \
--execution native \
--chain resources/rococo.json