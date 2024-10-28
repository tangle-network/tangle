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
FROM ubuntu:22.04

COPY ../target/release/tangle /usr/local/bin/

LABEL maintainer="Webb Developers <dev@webb.tools>"
LABEL description="Tangle Network Node"

RUN apt-get update && apt-get install -y --no-install-recommends \
    curl \
    ca-certificates libc6 \
    && rm -rf /var/lib/apt/lists/*

RUN useradd -m -u 5000 -U -s /bin/sh -d /tangle tangle && \
	mkdir -p /data /tangle/.local/share && \
	chown -R tangle:tangle /data && \
	ln -s /data /tangle/.local/share/tangle && \
	# unclutter and minimize the attack surface
	rm -rf /usr/bin /usr/sbin && \
	# check if executable works in this container
	/usr/local/bin/tangle --version

USER tangle

EXPOSE 30333 9933 9944 9615
VOLUME ["/data"]
ENTRYPOINT ["/usr/local/bin/tangle"]
