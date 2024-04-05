#!/bin/bash
set -e
# ensure we kill all child processes when we exit
trap "trap - SIGTERM && kill -- -$$" SIGINT SIGTERM EXIT

#define default ports
ports=(30333 30305 30308 30311 30313)

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

mkdir -p ./tmp

# The following line ensure we run from the project root
PROJECT_ROOT=$(git rev-parse --show-toplevel)
cd "$PROJECT_ROOT"

echo "*** Start Tangle Testnet ***"
# Alice 
./target/release/tangle -d ./tmp/alice --dev --validator -lerror --alice \
  --rpc-cors all --rpc-methods=unsafe --rpc-external \
  --port ${ports[0]} \
  --rpc-port 9944 \
  --rpc-max-request-size 3000 \
  --rpc-max-response-size 3000 \
  --ethapi trace,debug \
  --auto-insert-keys \
  --node-key 0000000000000000000000000000000000000000000000000000000000000001 &
# Sleep for a while to allow the node to start
sleep 3
# Bob
./target/release/tangle -d ./tmp/bob --dev --validator -lerror --bob \
  --rpc-cors all --rpc-methods=unsafe --rpc-external \
  --port ${ports[1]} \
  --rpc-port 9945 \
  --ethapi trace,debug \
  --auto-insert-keys \
  --bootnodes /ip4/127.0.0.1/tcp/30333/p2p/12D3KooWEyoppNCUx8Yx66oV9fJnriXwCcXwDDUA2kj6vnc6iDEp &
# Charlie
./target/release/tangle -d ./tmp/charlie --dev --validator -lerror --charlie \
  --rpc-cors all --rpc-methods=unsafe --rpc-external \
  --port ${ports[2]} \
  --rpc-port 9946 \
  --ethapi trace,debug \
  --auto-insert-keys \
  --bootnodes /ip4/127.0.0.1/tcp/30333/p2p/12D3KooWEyoppNCUx8Yx66oV9fJnriXwCcXwDDUA2kj6vnc6iDEp &
# Dave
./target/release/tangle -d ./tmp/dave --dev --validator -lerror --dave \
  --rpc-cors all --rpc-methods=unsafe --rpc-external \
  --port ${ports[3]} \
  --rpc-port 9947 \
  --ethapi trace,debug \
  --auto-insert-keys \
  --bootnodes /ip4/127.0.0.1/tcp/30333/p2p/12D3KooWEyoppNCUx8Yx66oV9fJnriXwCcXwDDUA2kj6vnc6iDEp &
# Eve
./target/release/tangle -d ./tmp/eve --dev --validator -linfo --eve \
    --rpc-cors all --rpc-methods=unsafe --rpc-external \
    --port ${ports[4]} \
    --rpc-port 9948 \
    --ethapi trace,debug \
    --auto-insert-keys \
    -levm=debug \
    -lgadget=trace \
    --bootnodes /ip4/127.0.0.1/tcp/30333/p2p/12D3KooWEyoppNCUx8Yx66oV9fJnriXwCcXwDDUA2kj6vnc6iDEp
popd
