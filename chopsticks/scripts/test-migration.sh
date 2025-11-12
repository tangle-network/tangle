#!/bin/bash
# Automated runtime migration testing script
# Usage: ./test-migration.sh [mainnet|testnet]

set -e

NETWORK=${1:-mainnet}
PORT=8000

if [ "$NETWORK" = "testnet" ]; then
    PORT=8001
fi

echo "========================================="
echo "Tangle Runtime Migration Test"
echo "Network: $NETWORK"
echo "========================================="
echo ""

# Check dependencies
echo "Checking dependencies..."
if ! command -v try-runtime &> /dev/null; then
    echo "ERROR: try-runtime-cli not found. Install with:"
    echo "  cargo install --git https://github.com/paritytech/try-runtime-cli --locked"
    exit 1
fi

if ! command -v npx &> /dev/null; then
    echo "ERROR: npx not found. Install Node.js first."
    exit 1
fi

# Check runtime WASM exists
if [ "$NETWORK" = "mainnet" ]; then
    RUNTIME_WASM="../target/release/wbuild/tangle-mainnet-runtime/tangle_mainnet_runtime.wasm"
else
    RUNTIME_WASM="../target/release/wbuild/tangle-testnet-runtime/tangle_testnet_runtime.wasm"
fi

if [ ! -f "$RUNTIME_WASM" ]; then
    echo "ERROR: Runtime WASM not found at $RUNTIME_WASM"
    echo "Build with: cargo build --release --features try-runtime --package tangle-${NETWORK}-runtime"
    exit 1
fi

echo "✓ Dependencies OK"
echo "✓ Runtime WASM found"
echo ""

# Start Chopsticks fork
echo "Starting Chopsticks fork on port $PORT..."
npx @acala-network/chopsticks \
    --config=./configs/${NETWORK}.yml \
    --port=$PORT &

CHOPSTICKS_PID=$!
echo "Chopsticks PID: $CHOPSTICKS_PID"

# Wait for Chopsticks to be ready
echo "Waiting for Chopsticks to be ready..."
sleep 10

# Test connection
if ! nc -z localhost $PORT 2>/dev/null; then
    echo "ERROR: Chopsticks not responding on port $PORT"
    kill $CHOPSTICKS_PID 2>/dev/null || true
    exit 1
fi

echo "✓ Chopsticks fork ready"
echo ""

# Run migration test
echo "========================================="
echo "Running on-runtime-upgrade test..."
echo "========================================="
echo ""

RUST_LOG=runtime=debug,try-runtime::cli=trace \
try-runtime \
    --runtime $RUNTIME_WASM \
    on-runtime-upgrade \
    live \
    --uri ws://localhost:$PORT

TEST_RESULT=$?

# Cleanup
echo ""
echo "Cleaning up..."
kill $CHOPSTICKS_PID 2>/dev/null || true

if [ $TEST_RESULT -eq 0 ]; then
    echo ""
    echo "========================================="
    echo "✅ Migration test PASSED"
    echo "========================================="
    exit 0
else
    echo ""
    echo "========================================="
    echo "❌ Migration test FAILED"
    echo "========================================="
    exit 1
fi
