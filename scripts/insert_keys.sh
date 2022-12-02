#!/bin/sh

# The following line ensure we run from the project root
PROJECT_ROOT=$(git rev-parse --show-toplevel)
cd "$PROJECT_ROOT"

echo "****************** NODE-1 KEY INSERTION ******************"

./target/release/tangle-standalone key insert --base-path ./tmp/standalone1 \
--chain ./chainspecs/arana-local.json \
--scheme Sr25519 \
--suri "gown surprise mirror hotel cash alarm raccoon you frog rose midnight enter//webb//0" \
--key-type acco

./target/release/tangle-standalone key insert --base-path ./tmp/standalone1 \
--chain ./chainspecs/arana-local.json \
--scheme Sr25519 \
--suri "gown surprise mirror hotel cash alarm raccoon you frog rose midnight enter//webb//0//stash" \
--key-type acco

./target/release/tangle-standalone key insert --base-path ./tmp/standalone1 \
--chain ./chainspecs/arana-local.json \
--scheme Sr25519 \
--suri "gown surprise mirror hotel cash alarm raccoon you frog rose midnight enter//webb//0//aura" \
--key-type aura

./target/release/tangle-standalone key insert --base-path ./tmp/standalone1 \
--chain ./chainspecs/arana-local.json \
--scheme Sr25519 \
--suri "gown surprise mirror hotel cash alarm raccoon you frog rose midnight enter//webb//0//aura" \
--key-type imon

./target/release/tangle-standalone key insert --base-path ./tmp/standalone1 \
--chain ./chainspecs/arana-local.json \
--scheme Ed25519 \
--suri "gown surprise mirror hotel cash alarm raccoon you frog rose midnight enter//webb//0//grandpa" \
--key-type gran

./target/release/tangle-standalone key insert --base-path ./tmp/standalone1 \
--chain ./chainspecs/arana-local.json \
--scheme Ecdsa \
--suri "gown surprise mirror hotel cash alarm raccoon you frog rose midnight enter//webb//0//dkg" \
--key-type wdkg

echo "node-1 keys inserted into path: ./tmp/standalone1 \n"

echo "\n ****************** NODE-2 KEY INSERTION ******************"

./target/release/tangle-standalone key insert --base-path ./tmp/standalone2 \
--chain ./chainspecs/arana-local.json \
--scheme Sr25519 \
--suri "gown surprise mirror hotel cash alarm raccoon you frog rose midnight enter//webb//1" \
--key-type acco

./target/release/tangle-standalone key insert --base-path ./tmp/standalone2 \
--chain ./chainspecs/arana-local.json \
--scheme Sr25519 \
--suri "gown surprise mirror hotel cash alarm raccoon you frog rose midnight enter//webb//1//stash" \
--key-type acco

./target/release/tangle-standalone key insert --base-path ./tmp/standalone2 \
--chain ./chainspecs/arana-local.json \
--scheme Sr25519 \
--suri "gown surprise mirror hotel cash alarm raccoon you frog rose midnight enter//webb//1//aura" \
--key-type aura

./target/release/tangle-standalone key insert --base-path ./tmp/standalone2 \
--chain ./chainspecs/arana-local.json \
--scheme Sr25519 \
--suri "gown surprise mirror hotel cash alarm raccoon you frog rose midnight enter//webb//1//aura" \
--key-type imon

./target/release/tangle-standalone key insert --base-path ./tmp/standalone2 \
--chain ./chainspecs/arana-local.json \
--scheme Ed25519 \
--suri "gown surprise mirror hotel cash alarm raccoon you frog rose midnight enter//webb//1//grandpa" \
--key-type gran

./target/release/tangle-standalone key insert --base-path ./tmp/standalone2 \
--chain ./chainspecs/arana-local.json \
--scheme Ecdsa \
--suri "gown surprise mirror hotel cash alarm raccoon you frog rose midnight enter//webb//1//dkg" \
--key-type wdkg

echo "node-2 keys inserted into path: ./tmp/standalone2 \n"


echo "\n ****************** NODE-3 KEY INSERTION ******************"

./target/release/tangle-standalone key insert --base-path ./tmp/standalone3 \
--chain ./chainspecs/arana-local.json \
--scheme Sr25519 \
--suri "gown surprise mirror hotel cash alarm raccoon you frog rose midnight enter//webb//2" \
--key-type acco

./target/release/tangle-standalone key insert --base-path ./tmp/standalone3 \
--chain ./chainspecs/arana-local.json \
--scheme Sr25519 \
--suri "gown surprise mirror hotel cash alarm raccoon you frog rose midnight enter//webb//2//stash" \
--key-type acco

./target/release/tangle-standalone key insert --base-path ./tmp/standalone3 \
--chain ./chainspecs/arana-local.json \
--scheme Sr25519 \
--suri "gown surprise mirror hotel cash alarm raccoon you frog rose midnight enter//webb//2//aura" \
--key-type aura

./target/release/tangle-standalone key insert --base-path ./tmp/standalone3 \
--chain ./chainspecs/arana-local.json \
--scheme Sr25519 \
--suri "gown surprise mirror hotel cash alarm raccoon you frog rose midnight enter//webb//2//aura" \
--key-type imon

./target/release/tangle-standalone key insert --base-path ./tmp/standalone3 \
--chain ./chainspecs/arana-local.json \
--scheme Ed25519 \
--suri "gown surprise mirror hotel cash alarm raccoon you frog rose midnight enter//webb//2//grandpa" \
--key-type gran

./target/release/tangle-standalone key insert --base-path ./tmp/standalone3 \
--chain ./chainspecs/arana-local.json \
--scheme Ecdsa \
--suri "gown surprise mirror hotel cash alarm raccoon you frog rose midnight enter//webb//2//dkg" \
--key-type wdkg


echo "\n ****************** NODE-4 KEY INSERTION ******************"

./target/release/tangle-standalone key insert --base-path ./tmp/standalone4 \
--chain ./chainspecs/arana-local.json \
--scheme Sr25519 \
--suri "gown surprise mirror hotel cash alarm raccoon you frog rose midnight enter//webb//3" \
--key-type acco

./target/release/tangle-standalone key insert --base-path ./tmp/standalone4 \
--chain ./chainspecs/arana-local.json \
--scheme Sr25519 \
--suri "gown surprise mirror hotel cash alarm raccoon you frog rose midnight enter//webb//3//stash" \
--key-type acco

./target/release/tangle-standalone key insert --base-path ./tmp/standalone4 \
--chain ./chainspecs/arana-local.json \
--scheme Sr25519 \
--suri "gown surprise mirror hotel cash alarm raccoon you frog rose midnight enter//webb//3//aura" \
--key-type aura

./target/release/tangle-standalone key insert --base-path ./tmp/standalone4 \
--chain ./chainspecs/arana-local.json \
--scheme Sr25519 \
--suri "gown surprise mirror hotel cash alarm raccoon you frog rose midnight enter//webb//3//aura" \
--key-type imon

./target/release/tangle-standalone key insert --base-path ./tmp/standalone4 \
--chain ./chainspecs/arana-local.json \
--scheme Ed25519 \
--suri "gown surprise mirror hotel cash alarm raccoon you frog rose midnight enter//webb//3//grandpa" \
--key-type gran

./target/release/tangle-standalone key insert --base-path ./tmp/standalone4 \
--chain ./chainspecs/arana-local.json \
--scheme Ecdsa \
--suri "gown surprise mirror hotel cash alarm raccoon you frog rose midnight enter//webb//3//dkg" \
--key-type wdkg

echo "\n ****************** NODE-5 KEY INSERTION ******************"

./target/release/tangle-standalone key insert --base-path ./tmp/standalone5 \
--chain ./chainspecs/arana-local.json \
--scheme Sr25519 \
--suri "gown surprise mirror hotel cash alarm raccoon you frog rose midnight enter//webb//4" \
--key-type acco

./target/release/tangle-standalone key insert --base-path ./tmp/standalone5 \
--chain ./chainspecs/arana-local.json \
--scheme Sr25519 \
--suri "gown surprise mirror hotel cash alarm raccoon you frog rose midnight enter//webb//4//stash" \
--key-type acco

./target/release/tangle-standalone key insert --base-path ./tmp/standalone5 \
--chain ./chainspecs/arana-local.json \
--scheme Sr25519 \
--suri "gown surprise mirror hotel cash alarm raccoon you frog rose midnight enter//webb//4//aura" \
--key-type aura

./target/release/tangle-standalone key insert --base-path ./tmp/standalone5 \
--chain ./chainspecs/arana-local.json \
--scheme Sr25519 \
--suri "gown surprise mirror hotel cash alarm raccoon you frog rose midnight enter//webb//4//aura" \
--key-type imon

./target/release/tangle-standalone key insert --base-path ./tmp/standalone5 \
--chain ./chainspecs/arana-local.json \
--scheme Ed25519 \
--suri "gown surprise mirror hotel cash alarm raccoon you frog rose midnight enter//webb//4//grandpa" \
--key-type gran

./target/release/tangle-standalone key insert --base-path ./tmp/standalone5 \
--chain ./chainspecs/arana-local.json \
--scheme Ecdsa \
--suri "gown surprise mirror hotel cash alarm raccoon you frog rose midnight enter//webb//4//dkg" \
--key-type wdkg
