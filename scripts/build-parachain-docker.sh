#!/usr/bin/env bash

set -e

# The following line ensure we run from the project root
PROJECT_ROOT=`git rev-parse --show-toplevel`
cd $PROJECT_ROOT

VERSION=`grep "^version" ./node/Cargo.toml | egrep -o "([0-9\.]+)"`
NODE_NAME=ghcr.io/webb-tools/dkg-node
BUILD_ARGS="cargo build --release -p dkg-node"

docker build -f ./docker/Parachain.Dockerfile . -t $NODE_NAME:$VERSION --build-arg GIT_COMMIT=${VERSION} --build-arg BUILD_ARGS="$BUILD_ARGS"
docker push $NODE_NAME:$VERSION