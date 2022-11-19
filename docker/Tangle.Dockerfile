# Copyright 2022 Webb Technologies Inc.
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
FROM lukemathwalker/cargo-chef:latest-rust-1 AS chef
WORKDIR /tangle

# Set the Binary name that we are trying to build.
ARG BINARY

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
# Install Required Packages
RUN apt-get update && apt-get install -y git \
  cmake clang curl libssl-dev llvm libudev-dev \
  libgmp3-dev protobuf-compiler ca-certificates \
  && rm -rf /var/lib/apt/lists/* && update-ca-certificates

COPY --from=planner /tangle/recipe.json recipe.json
COPY rust-toolchain.toml .
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
# Build application
RUN cargo build --release -p ${BINARY}

# This is the 2nd stage: a very small image where we copy the tangle binary."
FROM debian:buster-slim
LABEL maintainer="Webb Developers <dev@webb.tools>"
LABEL description="Binary for ${BINARY}"
ARG BINARY

COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt 
COPY --from=builder /tangle/target/release/${BINARY} /usr/local/bin

RUN useradd -m -u 1000 -U -s /bin/sh -d /tangle tangle && \
  mkdir -p /data /tangle/.local/share/tangle && \
  chown -R tangle:tangle /data && \
  ln -s /data /tangle/.local/share/tangle && \
  # Sanity checks
  ldd /usr/local/bin/$BINARY && \
  /usr/local/bin/$BINARY --version

USER tangle
EXPOSE 30333 9933 9944 9615
VOLUME ["/data"]
