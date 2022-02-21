FROM rust:1
WORKDIR /webb

# Install Required Packages
RUN apt-get update && \
    apt-get install -y git pkg-config clang curl libssl-dev llvm libudev-dev libgmp3-dev && \
    rm -rf /var/lib/apt/lists/*

COPY . .

RUN rustup default nightly

RUN cargo install cargo-tarpaulin

CMD SKIP_WASM_BUILD=1 cargo +nightly tarpaulin --out Xml \
    -p pallet-dkg-metadata \
    -p pallet-dkg-proposal-handler \
    -p pallet-dkg-proposals \
    -p pallet-parachain-staking \
    -p dkg-primitives \
    -p dkg-runtime-primitives \
    --timeout 3600
