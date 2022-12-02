#!/bin/sh

#!/usr/bin/env bash
set -e

CLEAN=${CLEAN:-false}
# Parse arguments for the script

while [[ $# -gt 0 ]]; do
    key="$1"
    case $key in
        -c|--clean)
            CLEAN=true
            shift # past argument
            ;;
        *)    # unknown option
            shift # past argument
            ;;
    esac
done

pushd .

# Check if we should clean the tmp directory
if [ "$CLEAN" = true ]; then
  echo "Cleaning tmp directory"
  rm -rf ./tmp
fi

# The following line ensure we run from the project root
PROJECT_ROOT=$(git rev-parse --show-toplevel)
cd "$PROJECT_ROOT"

echo "** Generating arana local chainspec"
./target/release/tangle-standalone build-spec --chain arana-local > ./chainspecs/arana-local.json

echo "** Inserting keys **"
./scripts/insert_keys.sh

echo "*** Start Webb DKG Standalone | Arana Local Config ***"
# Node 1
./target/release/tangle-standalone --base-path=./tmp/standalone1 -lerror --chain ./chainspecs/arana-local.json --validator \
  --rpc-cors all --unsafe-rpc-external --unsafe-ws-external \
  --port 30304 \
  --ws-port 9944 &
# Node 2
./target/release/tangle-standalone --base-path=./tmp/standalone2 -lerror --chain ./chainspecs/arana-local.json --validator \
  --rpc-cors all --unsafe-rpc-external --unsafe-ws-external \
  --port 30305 \
  --ws-port 9945 &
# Node 3
./target/release/tangle-standalone --base-path=./tmp/standalone3 -lerror --chain ./chainspecs/arana-local.json --validator \
  --rpc-cors all --unsafe-rpc-external --unsafe-ws-external \
  --port 30306 \
  --ws-port 9946 &
# Node 4
./target/release/tangle-standalone --base-path=./tmp/standalone4 -lerror --chain ./chainspecs/arana-local.json --validator \
  --rpc-cors all --unsafe-rpc-external --unsafe-ws-external \
  --port 30307 \
  --ws-port 9947 &
# Node 5
./target/release/tangle-standalone --base-path=./tmp/standalone5 -linfo --validator --chain ./chainspecs/arana-local.json \
    --rpc-cors all --unsafe-rpc-external --unsafe-ws-external \
    --ws-port 9948 \
    --port 30308 \
    -ldkg=debug \
    -ldkg_gadget::worker=debug \
    -lruntime::dkg_metadata=debug \
    -ldkg_metadata=debug \
    -lruntime::dkg_proposal_handler=debug \
    -lruntime::offchain=debug \
    -ldkg_proposal_handler=debug \
    -lruntime::im-online=debug
popd
