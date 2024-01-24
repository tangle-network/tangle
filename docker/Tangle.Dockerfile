# Copyright 2024 Webb Technologies Inc.
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
# http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.
#
FROM ubuntu:20.04 AS chef
WORKDIR /tangle

# Set the Binary name that we are trying to build.
ARG BINARY

COPY . .

FROM chef AS builder
# Install Required Packages
RUN apt-get update && apt-get install -y git \
  cmake clang curl libssl-dev llvm libudev-dev \
  libgmp3-dev protobuf-compiler ca-certificates \
  && rm -rf /var/lib/apt/lists/* && update-ca-certificates

COPY rust-toolchain.toml .
# Build dependencies - this is the caching Docker layer!
ARG BINARY
ARG FEATURES=default
COPY . .
# Build application
RUN cargo build -Z sparse-registry --release --features ${FEATURES} -p ${BINARY}

# This is the 2nd stage: a very small image where we copy the tangle binary."
FROM ubuntu:20.04
LABEL maintainer="Webb Developers <dev@webb.tools>"
LABEL description="Tangle Network Node"
ARG BINARY
ENV BINARY=${BINARY}

COPY --from=builder /tangle/target/release/${BINARY} /usr/local/bin

# check if executable works in this container
RUN /usr/local/bin/tangle --version

EXPOSE 30333 9933 9944
VOLUME ["/tangle"]
ENTRYPOINT ["/usr/local/bin/tangle "]
