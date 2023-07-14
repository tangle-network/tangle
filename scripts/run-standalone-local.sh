#!/bin/bash
set -e
# ensure we kill all child processes when we exit
trap "trap - SIGTERM && kill -- -$$" SIGINT SIGTERM EXIT

#define default ports
ports=(30333 30305 30308)

#check to see process is not orphaned or already running
for port in ${ports[@]}; do
    if [[ $(lsof -i -P -n | grep LISTEN | grep :$port) ]]; then
      echo "Port $port has a running process. Exiting"
      exit -1
    fi
done

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

mkdir ./tmp

# The following line ensure we run from the project root
PROJECT_ROOT=$(git rev-parse --show-toplevel)
cd "$PROJECT_ROOT"

echo "*** Start Webb DKG Node ***"
# Alice 
./target/release/tangle-standalone --tmp --chain local --validator -lerror --alice \
  --rpc-cors all --rpc-methods=unsafe \
  --port ${ports[0]} \
  --node-key 0000000000000000000000000000000000000000000000000000000000000001 &
# Bob
./target/release/tangle-standalone --tmp --chain local --validator -lerror --bob \
  --rpc-cors all --rpc-methods=unsafe \
  --port ${ports[1]} \
   --bootnodes /ip4/127.0.0.1/tcp/30333/p2p/12D3KooWEyoppNCUx8Yx66oV9fJnriXwCcXwDDUA2kj6vnc6iDEp &
# Charlie
./target/release/tangle-standalone --tmp --chain local --validator -lerror --charlie \
  --rpc-cors all --rpc-methods=unsafe \
  --port ${ports[1]} \
   --bootnodes /ip4/127.0.0.1/tcp/30333/p2p/12D3KooWEyoppNCUx8Yx66oV9fJnriXwCcXwDDUA2kj6vnc6iDEp &
# Dave
./target/release/tangle-standalone --tmp --chain local --validator -lerror --dave \
  --rpc-cors all --rpc-methods=unsafe \
  --port ${ports[1]} \
   --bootnodes /ip4/127.0.0.1/tcp/30333/p2p/12D3KooWEyoppNCUx8Yx66oV9fJnriXwCcXwDDUA2kj6vnc6iDEp &
# Eve
./target/release/tangle-standalone --tmp --chain local --validator -linfo --eve \
    --rpc-cors all \
    --port ${ports[2]} \
    -ldkg=debug \
    -ldkg_gadget::worker=debug \
    -lruntime::dkg_metadata=debug \
    -ldkg_metadata=debug \
    -lruntime::dkg_proposal_handler=debug \
    -lruntime::offchain=debug --bootnodes /ip4/127.0.0.1/tcp/30333/p2p/12D3KooWEyoppNCUx8Yx66oV9fJnriXwCcXwDDUA2kj6vnc6iDEp \
    -ldkg_proposal_handler=debug --unsafe-rpc-external --rpc-methods=unsafe
popd
