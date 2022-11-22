#!/usr/bin/env bash
set -e

pushd .

# The following line ensure we run from the project root
PROJECT_ROOT=$(git rev-parse --show-toplevel)
cd "$PROJECT_ROOT"

# Find the current version from Cargo.toml
VERSION=$(grep "^version" ./standalone/node/Cargo.toml | grep -E -o "([0-9\.]+)")
GITUSER=webb-tools
IMAGE_NAME=tangle-collator

# Build the image
echo "Building ${GITUSER}/${IMAGE_NAME}:latest docker image, hang on!"
time docker build -f ./docker/Tangle.Dockerfile --build-arg BINARY=$IMAGE_NAME -t ${GITUSER}/${IMAGE_NAME}:latest .
docker tag ${GITUSER}/${IMAGE_NAME}:latest ${GITUSER}/${IMAGE_NAME}:v"${VERSION}"

# Show the list of available images for this repo
echo "Image is ready"
docker images | grep ${IMAGE_NAME}

popd
