FROM rust:buster as builder
WORKDIR /network

RUN rustup default nightly-2022-02-01 && \
	rustup target add wasm32-unknown-unknown --toolchain nightly-2022-02-01

# Install Required Packages
RUN apt-get update && apt-get install -y git clang curl libssl-dev llvm libudev-dev libgmp3-dev && rm -rf /var/lib/apt/lists/*

ARG GIT_COMMIT=
ENV GIT_COMMIT=$GIT_COMMIT
ARG BUILD_ARGS

COPY . .
# Build DKG Parachain Node
RUN cargo build --release --locked -p egg-collator

# =============

FROM phusion/baseimage:bionic-1.0.0

RUN useradd -m -u 1000 -U -s /bin/sh -d /dkg dkg

COPY --from=builder /network/target/release/egg-collator /usr/local/bin

# checks
RUN ldd /usr/local/bin/egg-collator && \
  /usr/local/bin/egg-collator --version

# Shrinking
RUN rm -rf /usr/lib/python* && \
	rm -rf /usr/bin /usr/sbin /usr/share/man

USER dkg
EXPOSE 30333 9933 9944 9615

RUN mkdir /dkg/data
RUN chown -R dkg:dkg /dkg/data

VOLUME ["/dkg/data"]

ENTRYPOINT [ "/usr/local/bin/egg-collator" ]