#!/usr/bin/env bash
set +e

# launch the standalone network
echo "** Starting standalone network **"

echo "Cleaning tmp directory"
rm -rf ./tmp
rm -rf ./chainspecs/standalone-local.json

# The following line ensure we run from the project root
PROJECT_ROOT=$(git rev-parse --show-toplevel)
cd "$PROJECT_ROOT"

echo "** Generating Standalone local chainspec"
./target/release/tangle-standalone build-spec --chain standalone-local > ./chainspecs/standalone-local.json

echo "** Inserting keys **"
./scripts/insert_keys.sh

echo "*** Start Tangle Standalone | Standalone Local Config ***"
# Node 1
./target/release/tangle-standalone --base-path=./tmp/standalone1 -lerror --chain ./chainspecs/standalone-local.json --validator \
  --rpc-cors all --unsafe-rpc-external --unsafe-ws-external \
  --port 30304 \
  --ws-port 9944  > ~/1.log 2>&1 &
# Node 2
./target/release/tangle-standalone --base-path=./tmp/standalone2 -lerror --chain ./chainspecs/standalone-local.json --validator \
  --rpc-cors all --unsafe-rpc-external --unsafe-ws-external \
  --port 30305 \
  --ws-port 9945  > ~/2.log 2>&1 &
# Node 3
./target/release/tangle-standalone --base-path=./tmp/standalone3 -lerror --chain ./chainspecs/standalone-local.json --validator \
  --rpc-cors all --unsafe-rpc-external --unsafe-ws-external \
  --port 30306 \
  --ws-port 9946  > ~/3.log 2>&1 &
# Node 4
./target/release/tangle-standalone --base-path=./tmp/standalone4 -lerror --chain ./chainspecs/standalone-local.json --validator \
  --rpc-cors all --unsafe-rpc-external --unsafe-ws-external \
  --port 30307 \
  --ws-port 9947 > ~/4.log 2>&1 &
# Node 5
./target/release/tangle-standalone --base-path=./tmp/standalone5 -linfo --validator --chain ./chainspecs/standalone-local.json \
    --rpc-cors all --unsafe-rpc-external --unsafe-ws-external \
    --ws-port 9948 \
    --port 30308 > ~/5.log 2>&1 &

# wait for sometime for the network to be ready
echo "** Waiting for testnet to start producing blocks **"
sleep 120

echo "** Starting test suite **"

cd dkg-liveness-test
npm install

if ! node index.js ; then exit 1 ; fi

echo "** Liveness testing completed **"

# trap 'trap - SIGTERM && kill 0' SIGINT SIGTERM EXIT

exit 0
