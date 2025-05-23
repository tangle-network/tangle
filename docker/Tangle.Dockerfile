# Copyright 2024 Tangle Foundation.
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

# Build stage
FROM ubuntu:22.04 AS builder

LABEL maintainer="Webb Developers <dev@webb.tools>"
LABEL description="Tangle Network Builder"

# Install dependencies required for building
RUN apt-get update && apt-get install -y --no-install-recommends \
    curl ca-certificates git build-essential \
    clang cmake pkg-config libssl-dev libc6 zlib1g-dev libtinfo-dev \
    && rm -rf /var/lib/apt/lists/*

# Install Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

# Copy the source code
WORKDIR /build
COPY . /build

# Build the Tangle binary
RUN cargo build --release

# Verify the binary works in this environment
RUN /build/target/release/tangle --version

# Run stage - using the same Ubuntu 22.04 to ensure binary compatibility
FROM ubuntu:22.04

LABEL maintainer="Webb Developers <dev@webb.tools>"
LABEL description="Tangle Network Node"

# Install minimal runtime dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates libc6 zlib1g-dev libtinfo-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy the binary from builder stage
COPY --from=builder /build/target/release/tangle /usr/local/bin/

# Create user and set up directories
RUN useradd -m -u 5000 -U -s /bin/sh -d /tangle tangle && \
	mkdir -p /data /tangle/.local/share && \
	chown -R tangle:tangle /data && \
	ln -s /data /tangle/.local/share/tangle && \
	# unclutter and minimize the attack surface
	rm -rf /usr/bin /usr/sbin && \
	# Verify the binary works in the final container
	/usr/local/bin/tangle --version

USER tangle

EXPOSE 30333 9933 9944 9615
VOLUME ["/data"]
ENTRYPOINT ["/usr/local/bin/tangle"]
