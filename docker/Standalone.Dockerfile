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
# Build Standalone Node.
RUN git submodule update --init && \
  cargo build --release --locked -p egg-standalone-node


# ===============

FROM phusion/baseimage:bionic-1.0.0

COPY --from=builder /network/target/release/egg-standalone-node /usr/local/bin

RUN useradd -m -u 1000 -U -s /bin/sh -d /network webb && \
  mkdir -p /data /network/.local/share/webb && \
  chown -R webb:webb /data && \
  ln -s /data /network/.local/share/webb && \
  # Sanity checks
  ldd /usr/local/bin/egg-standalone-node && \
  /usr/local/bin/egg-standalone-node --version

USER webb
EXPOSE 30333 9933 9944 9615 33334
VOLUME ["/data"]

ENTRYPOINT [ "/usr/local/bin/egg-standalone-node"]