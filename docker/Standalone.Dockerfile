FROM rust:1 as builder
WORKDIR /dkg

# Install Required Packages
RUN apt-get update && apt-get install -y git clang curl libssl-dev llvm libudev-dev libgmp3-dev && rm -rf /var/lib/apt/lists/*

COPY . .
# Build Standalone Node
RUN cargo build --release -p dkg-standalone-node

# This is the 2nd stage: a very small image where we copy the DKG binary."

FROM ubuntu:20.04

COPY --from=builder /dkg/target/release/dkg-standalone-node /usr/local/bin

RUN apt-get update && apt-get install -y clang libssl-dev llvm libudev-dev libgmp3-dev && rm -rf /var/lib/apt/lists/*

RUN useradd -m -u 1000 -U -s /bin/sh -d /dkg dkg && \
  mkdir -p /data /dkg/.local/share/dkg && \
  chown -R dkg:dkg /data && \
  ln -s /data /dkg/.local/share/dkg && \
  # Sanity checks
  ldd /usr/local/bin/dkg-standalone-node && \
  /usr/local/bin/dkg-standalone-node --version

USER dkg
EXPOSE 30333 9933 9944 9615
VOLUME ["/data"]
