#!/bin/sh
set -e

CLEAN=${CLEAN:-false}

# define default ports
wstangleports=(9944 9945 9946 9947 9948)

# check to see process is not orphaned or already running
# for port in ${wstangleports[@]}; do
#     if [[ $(lsof -i -P -n | grep LISTEN | grep :$port) ]]; then
#       echo "Port $port has a running process. Exiting"
#       exit -1
#     fi
# done

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
  rm -rf ./chainspecs/standalone-local.json
fi

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
  --ws-port ${wstangleports[0]} &
# Node 2
./target/release/tangle-standalone --base-path=./tmp/standalone2 -lerror --chain ./chainspecs/standalone-local.json --validator \
  --rpc-cors all --unsafe-rpc-external --unsafe-ws-external \
  --port 30305 \
  --ws-port ${wstangleports[1]} &
# Node 3
./target/release/tangle-standalone --base-path=./tmp/standalone3 -lerror --chain ./chainspecs/standalone-local.json --validator \
  --rpc-cors all --unsafe-rpc-external --unsafe-ws-external \
  --port 30306 \
  --ws-port ${wstangleports[2]} &
# Node 4
./target/release/tangle-standalone --base-path=./tmp/standalone4 -lerror --chain ./chainspecs/standalone-local.json --validator \
  --rpc-cors all --unsafe-rpc-external --unsafe-ws-external \
  --port 30307 \
  --ws-port ${wstangleports[3]} &
# Node 5
./target/release/tangle-standalone --base-path=./tmp/standalone5 -linfo --validator --chain ./chainspecs/standalone-local.json \
    --rpc-cors all --unsafe-rpc-external --unsafe-ws-external \
    --ws-port ${wstangleports[4]} \
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