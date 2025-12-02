#!/bin/bash
# Automated runtime migration testing script
# Usage: ./test-migration.sh [mainnet|testnet]

set -euo pipefail

# Extra args for try-runtime CLI. Defaults disable spec-version
# and idempotency checks so we can iterate faster on snapshots.
TRY_RUNTIME_EXTRA_ARGS="${TRY_RUNTIME_EXTRA_ARGS:---disable-spec-version-check --disable-idempotency-checks}"

NETWORK=${1:-mainnet}
PORT=8000

if [ "$NETWORK" = "testnet" ]; then
    PORT=8001
fi

LOG_DIR=${LOG_DIR:-logs}
mkdir -p "$LOG_DIR"
mkdir -p snapshots

CHOPSTICKS_PID=""
TAIL_PIDS=()

cleanup() {
    if [ -n "$CHOPSTICKS_PID" ] && ps -p "$CHOPSTICKS_PID" > /dev/null 2>&1; then
        kill "$CHOPSTICKS_PID" >/dev/null 2>&1 || true
    fi
    for tail_pid in "${TAIL_PIDS[@]}"; do
        if ps -p "$tail_pid" > /dev/null 2>&1; then
            kill "$tail_pid" >/dev/null 2>&1 || true
        fi
    done
}
trap cleanup EXIT

timestamp() {
    date +%Y%m%d-%H%M%S
}

run_with_live_logs() {
    local log_file=$1
    shift
    echo "Logging to $log_file"
    : > "$log_file"
    stdbuf -oL -eL "$@" &> "$log_file" &
    local cmd_pid=$!
    tail -n 50 -f "$log_file" &
    local tail_pid=$!
    TAIL_PIDS+=("$tail_pid")
    wait "$cmd_pid"
    local status=$?
    kill "$tail_pid" >/dev/null 2>&1 || true
    return $status
}

echo "========================================="
echo "Tangle Runtime Migration Test"
echo "Network: $NETWORK"
echo "Logs directory: $LOG_DIR"
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

if ! command -v stdbuf &> /dev/null; then
    echo "ERROR: stdbuf not found (part of coreutils). Install before running."
    exit 1
fi

# Check runtime WASM exists
if [ "$NETWORK" = "mainnet" ]; then
    RUNTIME_WASM="../target/release/wbuild/tangle-runtime/tangle_runtime.wasm"
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

# Define snapshot file path
SNAPSHOT_FILE="snapshots/${NETWORK}.snap"

# Start Chopsticks fork with logging
echo "Starting Chopsticks fork on port $PORT..."
CHOP_LOG="$LOG_DIR/chopsticks-${NETWORK}-$(timestamp).log"
stdbuf -oL -eL npx @acala-network/chopsticks \
    --config=./configs/${NETWORK}.yml \
    --port=$PORT &> "$CHOP_LOG" &
CHOPSTICKS_PID=$!
tail -n 50 -f "$CHOP_LOG" &
TAIL_PIDS+=("$!")
echo "Chopsticks PID: $CHOPSTICKS_PID (log: $CHOP_LOG)"

# Wait for Chopsticks to be ready
echo "Waiting for Chopsticks to be ready..."
sleep 15
echo ""

echo "✓ Chopsticks fork ready"
echo ""

# Create or use existing snapshot
echo "========================================="
echo "Creating/Using snapshot..."
echo "========================================="
echo ""

if [ ! -f "$SNAPSHOT_FILE" ]; then
    echo "Snapshot not found. Creating snapshot from live node..."
    echo "Chopsticks URI: ws://localhost:$PORT"
    echo "Snapshot file: $SNAPSHOT_FILE"
    echo ""

    SNAP_LOG="$LOG_DIR/try-runtime-snapshot-${NETWORK}-$(timestamp).log"
    if ! run_with_live_logs "$SNAP_LOG" \
        env RUST_LOG=runtime=debug,try-runtime::cli=trace \
        try-runtime \
            --runtime existing \
            create-snapshot \
            --uri ws://localhost:$PORT \
            "$SNAPSHOT_FILE"; then
        echo "ERROR: Failed to create snapshot (see $SNAP_LOG)"
        exit 1
    fi

    echo "✓ Snapshot created successfully"
else
    echo "✓ Using existing snapshot: $SNAPSHOT_FILE"
fi
echo ""

# Run migration test
echo "========================================="
echo "Running on-runtime-upgrade test..."
echo "========================================="
echo ""

# Blocktime in milliseconds (6 seconds = 6000ms for Tangle)
# node/src/distributions/mainnet.rs:140
BLOCKTIME=6000

echo "Runtime WASM: $RUNTIME_WASM"
echo "Snapshot file: $SNAPSHOT_FILE"
echo "Blocktime: ${BLOCKTIME}ms"
echo ""

UPGRADE_LOG="$LOG_DIR/try-runtime-upgrade-${NETWORK}-$(timestamp).log"
if run_with_live_logs "$UPGRADE_LOG" \
    env RUST_LOG=runtime=debug,try-runtime::cli=trace \
    timeout 1800 \
    try-runtime \
        --runtime "$RUNTIME_WASM" \
        on-runtime-upgrade \
        --blocktime $BLOCKTIME \
        --checks pre-and-post \
        $TRY_RUNTIME_EXTRA_ARGS \
        snap \
        -p "$SNAPSHOT_FILE"; then
    TEST_RESULT=0
else
    TEST_RESULT=$?
fi

echo ""
echo "Cleaning up..."

if [ $TEST_RESULT -eq 0 ]; then
    echo ""
    echo "========================================="
    echo "✅ Migration test PASSED"
    echo "Logs:"
    echo "  Chopsticks: $CHOP_LOG"
    echo "  Upgrade:    $UPGRADE_LOG"
    echo "========================================="
    exit 0
else
    echo ""
    echo "========================================="
    echo "❌ Migration test FAILED (exit code $TEST_RESULT)"
    echo "Logs:"
    echo "  Chopsticks: $CHOP_LOG"
    echo "  Upgrade:    $UPGRADE_LOG"
    echo "========================================="
    exit $TEST_RESULT
fi
