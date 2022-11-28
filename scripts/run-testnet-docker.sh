#!/bin/bash
set -x
set -e

# READ ENV FILE
set -o allexport
source .env
set +o allexport

# PROJECT_NAME = <user_id>-<current_directory_name>
PROJECT_NAME=$USER-${PWD##*/}

# TEARDOWN AND DELETE
docker compose -p $PROJECT_NAME down -v

docker run -v $PWD/specs:/data/spec --rm $TANGLE build-spec --chain=$tangle_SOURCE_SPEC --raw > specs/raw-$tangle_SOURCE_SPEC.json

export tangle_RAW_SPEC_FILE=/data/spec/raw-$tangle_SOURCE_SPEC.json

# Get relay chain spec, genesis wasm+head
docker run -v $PWD/specs:/data/spec --rm $TANGLE export-genesis-state --chain=$tangle_RAW_SPEC_FILE > specs/tangle-genesis.hex
docker run -v $PWD/specs:/data/spec --rm $TANGLE export-genesis-wasm --chain=$tangle_RAW_SPEC_FILE > specs/tangle.wasm

# Active the line below if you are using a pre-compiled relay chain spec (peregrine {stg, prod})
# Else you need to build your own relay spec in the Polkadot repository (rococo-local for dev)
# docker run --rm --entrypoint cat $TANGLE /node/dev-specs/tangle-parachain/peregrine-stg-relay.json > specs/${RELAY_RAW_SPEC_FILE}

# Spin it up the network and script
docker compose -p $PROJECT_NAME up -d