# Node for Standalone Eggnet.
#
# Requires to run from repository root and to copy the binary in the build folder (part of the release workflow)

FROM rust:buster as builder
WORKDIR /app

RUN rustup default nightly-2022-06-22 && \
	rustup target add wasm32-unknown-unknown --toolchain nightly-2022-06-22

# Install deps
RUN apt-get update && apt-get install -y clang curl libssl-dev llvm libudev-dev libgmp3-dev protobuf-compiler && rm -rf /var/lib/apt/lists/*
RUN apt-get install -y ca-certificates && update-ca-certificates

COPY . .
# Build DKG Standalone Node
RUN cargo build --release -p egg-standalone-node

FROM debian:buster-slim
LABEL maintainer="Webb Developers <dev@webb.tools>"
LABEL description="Binary for Standalone Eggnet Node"

RUN useradd -m -u 1000 -U -s /bin/sh -d /eggnet admin && \
	mkdir -p /eggnet/.local/share && \
	mkdir /data && \
	chown -R admin:admin /data && \
	chown -R admin:admin /eggnet && \
	ln -s /data /eggnet/.local/share/standalone && \
	rm -rf /usr/bin /usr/sbin

COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt 
COPY --from=builder --chown=admin /app/target/release/egg-standalone-node /eggnet

USER admin

RUN chmod uog+x /eggnet/egg-standalone-node*

# 30333 for parachain p2p
# 9933 for RPC call
# 9944 for Websocket
# 9615 for Prometheus (metrics)
EXPOSE 30333 30334 9933 9944 9615

VOLUME ["/data"]

ENTRYPOINT ["/eggnet/egg-standalone-node"]