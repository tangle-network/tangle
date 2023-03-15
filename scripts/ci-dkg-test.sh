#!/usr/bin/env bash
set -e

# launch the standalone network
echo "** Starting standalone network **"
./scripts/run-standalone-local.sh --clean

# wait for sometime for the network to be ready
sleep 10

echo "** Starting test suite **"
cd dkg-liveness-test
npm install
node index.js