#!/usr/bin/env bash

set -e

echo "*** Start Tangle Standalone Node ***"
./target/release/tangle-standalone-node --dev --alice --node-key 0000000000000000000000000000000000000000000000000000000000000001 &
./target/release/tangle-standalone-node --dev --bob --port 33334 --tmp   --bootnodes /ip4/127.0.0.1/tcp/30333/p2p/12D3KooWEyoppNCUx8Yx66oV9fJnriXwCcXwDDUA2kj6vnc6iDEp
