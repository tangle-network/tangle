#!/usr/bin/env bash

set -eou pipefail

# syn-protobuf.sh is a bash script to sync the protobuf
# files using the proto-compiler project. It will check
# out the protobuf files from the git versions specified in
# proto/src/prost/COSMOS_SDK_COMMIT and
# proto/src/prost/IBC_GO_COMMIT. If you want to sync
# the protobuf files to a newer version, modify the
# relevant files with the new commit IDs.

# This script should be run from the root directory of ibc-rs

# We can specify where to clone the git repositories
# for cosmos-sdk and ibc-go. By default they are cloned
# to /tmp/cosmos-sdk.git and /tmp/ibc-go.git.
# We can override this to existing directories
# that already have a clone of the repositories,
# so that there is no need to clone the entire
# repositories over and over again every time
# the script is called.

CACHE_PATH="${XDG_CACHE_HOME:-$HOME/.cache}"
COSMOS_SDK_GIT="${COSMOS_SDK_GIT:-$CACHE_PATH/cosmos/cosmos-sdk.git}"
IBC_GO_GIT="${IBC_GO_GIT:-$CACHE_PATH/ibc-go.git}"
GOGO_PROTO_GIT="${GOGO_PROTO_GIT:-$CACHE_PATH/gogoproto.git}"
COSMOS_PROTO_GIT="${COSMOS_PROTO_GIT:-$CACHE_PATH/cosmos-proto.git}"
GOOGLE_APIS_GIT="${GOOGLE_APIS_GIT:-$CACHE_PATH/googleapis.git}"
ICS23_GIT="${ICS23_GIT:-$CACHE_PATH/ics23.git}"


COSMOS_SDK_COMMIT="$(cat ibc/proto/src/COSMOS_SDK_COMMIT)"
IBC_GO_COMMIT="$(cat ibc/proto/src/IBC_GO_COMMIT)"

echo "COSMOS_SDK_COMMIT: $COSMOS_SDK_COMMIT"
echo "IBC_GO_COMMIT: $IBC_GO_COMMIT"

# Use either --sdk-commit flag for commit ID,
# or --sdk-tag for git tag. Because we can't modify
# proto-compiler to have smart detection on that.

if [[ "$COSMOS_SDK_COMMIT" =~ ^[a-zA-Z0-9]{40}$ ]]
then
	SDK_COMMIT_OPTION="--sdk-commit"
else
	SDK_COMMIT_OPTION="--sdk-tag"
fi

# If the git directories does not exist, clone them as
# bare git repositories so that no local modification
# can be done there.

if [[ ! -e "$COSMOS_SDK_GIT" ]]
then
	echo "Cloning cosmos-sdk source code to as bare git repository to $COSMOS_SDK_GIT"
	git clone --mirror https://github.com/cosmos/cosmos-sdk.git "$COSMOS_SDK_GIT"
else
	echo "Using existing cosmos-sdk bare git repository at $COSMOS_SDK_GIT"
fi


if [[ ! -e "$IBC_GO_GIT" ]]
then
	echo "Cloning ibc-go source code to as bare git repository to $IBC_GO_GIT"
	git clone --mirror https://github.com/ComposableFi/ibc-go.git "$IBC_GO_GIT"
else
	echo "Using existing ibc-go bare git repository at $IBC_GO_GIT"
fi

if [[ ! -e "$GOGO_PROTO_GIT" ]]
then
	echo "Cloning gogoproto source code to as bare git repository to $GOGO_PROTO_GIT"
	git clone --mirror https://github.com/cosmos/gogoproto.git "$GOGO_PROTO_GIT"
else
	echo "Using existing gogoproto bare git repository at $GOGO_PROTO_GIT"
fi

if [[ ! -e "$COSMOS_PROTO_GIT" ]]
then
	echo "Cloning cosmos_proto source code to as bare git repository to $COSMOS_PROTO_GIT"
	git clone --mirror https://github.com/cosmos/cosmos-proto.git "$COSMOS_PROTO_GIT"
else
	echo "Using existing cosmos_proto bare git repository at $COSMOS_PROTO_GIT"
fi

if [[ ! -e "$GOOGLE_APIS_GIT" ]]
then
	echo "Cloning googleapis source code to as bare git repository to $GOOGLE_APIS_GIT"
	git clone --mirror https://github.com/googleapis/googleapis.git "$GOOGLE_APIS_GIT"
else
	echo "Using existing googleapis bare git repository at $GOOGLE_APIS_GIT"
fi

if [[ ! -e "$ICS23_GIT" ]]
then
	echo "Cloning ics23 source code to as bare git repository to $ICS23_GIT"
	git clone --mirror https://github.com/cosmos/ics23.git "$ICS23_GIT"
else
	echo "Using existing ics23 bare git repository at $ICS23_GIT"
fi

# Update the repositories using git fetch. This is so that
# we keep local copies of the repositories up to sync first.
pushd "$COSMOS_SDK_GIT"
git fetch
popd

pushd "$IBC_GO_GIT"
git fetch
popd

pushd "$GOGO_PROTO_GIT"
git fetch
popd

pushd "$COSMOS_PROTO_GIT"
git fetch
popd

pushd "$GOOGLE_APIS_GIT"
git fetch
popd

pushd "$ICS23_GIT"
git fetch
popd

# Create a new temporary directory to check out the
# actual source files from the bare git repositories.
# This is so that we do not accidentally use an unclean
# local copy of the source files to generate the protobuf.
COSMOS_SDK_DIR=$(mktemp -d /tmp/cosmos-sdk-XXXXXXXX)

pushd "$COSMOS_SDK_DIR"
git clone "$COSMOS_SDK_GIT" .
git checkout "$COSMOS_SDK_COMMIT"

# We have to name the commit as a branch because
# proto-compiler uses the branch name as the commit
# output. Otherwise it will just output HEAD
git checkout -b "$COSMOS_SDK_COMMIT"
popd

IBC_GO_DIR=$(mktemp -d /tmp/ibc-go-XXXXXXXX)

pushd "$IBC_GO_DIR"
git clone "$IBC_GO_GIT" .
git checkout "$IBC_GO_COMMIT"
git checkout -b "$IBC_GO_COMMIT"
popd

GOGO_DIR=$(mktemp -d /tmp/gogo-XXXXXXXX)

pushd "$GOGO_DIR"
git clone "$GOGO_PROTO_GIT" .
popd

COSMOS_PROTO_DIR=$(mktemp -d /tmp/cosmos-proto-XXXXXXXX)

pushd "$COSMOS_PROTO_DIR"
git clone "$COSMOS_PROTO_GIT" .
popd

GOOGLE_DIR=$(mktemp -d /tmp/google-apis-XXXXXXXX)

pushd "$GOOGLE_DIR"
git clone "$GOOGLE_APIS_GIT" .
popd

ICS23_DIR=$(mktemp -d /tmp/ics23-XXXXXXXX)

pushd "$ICS23_DIR"
git clone "$ICS23_GIT" .
popd

# Remove the existing generated protobuf files
# so that the newly generated code does not
# contain removed files.

rm -rf ibc/proto/src/prost
mkdir -p ibc/proto/src/prost

cd ibc/proto-compiler

cargo +nightly build

# Run the proto-compiler twice,
# once for std version with --build-tonic set to true
# and once for no-std version with --build-tonic set to false

cargo +nightly run -- compile \
	--sdk "$COSMOS_SDK_DIR" --ibc "$IBC_GO_DIR" --gogo "$GOGO_DIR" --google "$GOOGLE_DIR" --cosmos "$COSMOS_PROTO_DIR" --ics23 "$ICS23_DIR" --out ../proto/src/prost

# Remove the temporary checkouts of the repositories

rm -rf "$COSMOS_SDK_DIR"
rm -rf "$IBC_GO_DIR"
