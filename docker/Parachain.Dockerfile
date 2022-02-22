FROM rust:1 as builder
WORKDIR /parachain

# Install Required Packages
RUN apt-get update && apt-get install -y git clang curl libssl-dev llvm libudev-dev libgmp3-dev && rm -rf /var/lib/apt/lists/*

COPY . .
# Build Standalone Node
RUN cargo build --release

# This is the 2nd stage: a very small image where we copy the DKG binary."

FROM ubuntu:20.04

COPY --from=builder /parachain/target/release/egg-collator /usr/local/bin

RUN apt-get update && apt-get install -y clang libssl-dev llvm libudev-dev libgmp3-dev && rm -rf /var/lib/apt/lists/*

RUN useradd -m -u 1000 -U -s /bin/sh -d /parachain parachain && \
  mkdir -p /data /parachain/.local/share/parachain && \
  chown -R parachain:parachain /data && \
  ln -s /data /parachain/.local/share/parachain && \
  # Sanity checks
  ldd /usr/local/bin/egg-collator && \
  /usr/local/bin/egg-collator --version

USER parachain
EXPOSE 30333 9933 9944 9615
VOLUME ["/data"]
