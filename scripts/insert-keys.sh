#!/bin/bash

set -e

# Function to display usage instructions
show_usage() {
    echo "Usage: $0 [OPTIONS] <MNEMONIC>"
    echo "Insert keys generated from a mnemonic into the keystore for a Tangle node"
    echo ""
    echo "Options:"
    echo "  -h, --help             Show this help message and exit"
    echo "  -b, --base-path <PATH> Node base path (default: ./base-path)"
    echo "  -c, --chain <CHAIN>    Chain specification (default: tangle-testnet)"
    echo "  -n, --node-name <NAME> Node name for derivation (default: node)"
    echo "  -i, --node-index <NUM> Node index for derivation (default: 1)"
    echo ""
    echo "Example:"
    echo "  $0 'word1 word2 ... word12'"
    echo "  $0 --base-path /path/to/node --node-name validator --node-index 5 'word1 word2 ... word12'"
}

# Default values
BASE_PATH="./base-path"
CHAIN="tangle-testnet"
NODE_NAME="node"
NODE_INDEX="1"

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -h|--help)
            show_usage
            exit 0
            ;;
        -b|--base-path)
            BASE_PATH="$2"
            shift 2
            ;;
        -c|--chain)
            CHAIN="$2"
            shift 2
            ;;
        -n|--node-name)
            NODE_NAME="$2"
            shift 2
            ;;
        -i|--node-index)
            NODE_INDEX="$2"
            shift 2
            ;;
        -*)
            echo "Error: Unknown option: $1" >&2
            show_usage
            exit 1
            ;;
        *)
            # If not an option, assume it's the mnemonic
            MNEMONIC="$1"
            shift
            ;;
    esac
done

# Check if the mnemonic is provided
if [ -z "$MNEMONIC" ]; then
    echo "Error: Mnemonic is required" >&2
    show_usage
    exit 1
fi

# Check if tangle-standalone is available
if ! command -v tangle-standalone &> /dev/null; then
    echo "Error: tangle-standalone not found in PATH" >&2
    echo "Please make sure it's installed and available in your PATH" >&2
    exit 1
fi

# Check if base path exists, create if not
if [ ! -d "$BASE_PATH" ]; then
    mkdir -p "$BASE_PATH"
    echo "Created base path directory: $BASE_PATH"
fi

# Derivation path construction
DERIVATION_PATH="//$NODE_NAME//$NODE_INDEX"

# Function to insert a key with proper scheme
insert_key() {
    local scheme=$1
    local key_type=$2
    local suffix=$3
    local suri="${MNEMONIC}${DERIVATION_PATH}${suffix}"
    
    echo "Inserting $key_type key..."
    
    tangle-standalone key insert \
        --base-path "$BASE_PATH" \
        --chain "$CHAIN" \
        --scheme "$scheme" \
        --suri "$suri" \
        --key-type "$key_type"
        
    if [ $? -eq 0 ]; then
        echo "✅ Successfully inserted $key_type key"
    else
        echo "❌ Failed to insert $key_type key"
        exit 1
    fi
}

echo "===== Inserting Tangle Node Keys for $NODE_NAME-$NODE_INDEX ====="
echo "Using base path: $BASE_PATH"
echo "Chain: $CHAIN"

# Insert account keys
insert_key "Sr25519" "acco" ""

# Insert stash account keys
insert_key "Sr25519" "acco" "//stash"

# Insert BABE key (for block production) - Sr25519
insert_key "Sr25519" "babe" "//babe"

# Insert IMONLINE key (for heartbeat) - Sr25519
insert_key "Sr25519" "imon" "//imon"

# Insert GRANDPA key (for finality) - Ed25519
insert_key "Ed25519" "gran" "//grandpa"

# Insert ROLE key - Ecdsa
insert_key "Ecdsa" "role" "//role"

# Insert ROLE key - Sr25519
insert_key "Sr25519" "role" "//role"

echo "All keys have been inserted successfully into: $BASE_PATH"
echo "NOTE: Remember to restart your node for the keys to take effect."
