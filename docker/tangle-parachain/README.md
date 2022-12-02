# Tangle Parachain Docker

## Installation Instructions

A Tangle parachain node can be spun up quickly using Docker. At the time of writing, the Docker version used was 19.03.6. When connecting to testnet, it will take a few hours/days to completely sync the embedded relay chain. Make sure that your system meets the requirements.

Create a local directory to store the chain data:

```bash
mkdir /var/lib/tangle/
```

Download the latest chainspec for parachain testnet

```bash
https://github.com/webb-tools/tangle/blob/main/chainspecs/tangle-parachain.json
```

## Run via CLI :

You can use the following command to pull the latest image and run from your CLI, remember to set `YOUR-NODE-NAME` in two different places

```bash
docker run --platform linux/amd64 --network="host" -v "/var/lib/data" --entrypoint ./tangle-parachain \
ghcr.io/webb-tools/tangle/tangle-parachain:main \
--base-path=/data \
--chain dev \
--name="YOUR-NODE-NAME" \
--execution wasm \
--wasm-execution compiled \
--trie-cache-size 0 \
-- \
--execution wasm \
--name="YOUR-NODE-NAME (Embedded Relay)"
```

## Run via Docker Compose :

The docker-compose file will spin up a container running tangle standalone node, but you have to set the following environment variables.
Remember to customize your the values depending on your environment and then copy paste this to CLI.

```bash
RELEASE_VERSION=main
CHAINSPEC_PATH=/tmp/chainspec/
```

After that run :

```bash
docker compose up -d
```