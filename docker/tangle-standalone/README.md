# Tangle Standalone Docker

## Installation Instructions

A Tangle standalone node can be spun up quickly using Docker. At the time of writing, the Docker version used was 19.03.6. When connecting to testnet, it will take a few hours/days to completely sync chain. Make sure that your system meets the requirements.

Create a local directory to store the chain data:

```bash
mkdir /var/lib/tangle/
```

Download the latest chainspec for standalone testnet

```bash
https://github.com/webb-tools/tangle/blob/main/chainspecs/arana-alpha.json
```

## Run via CLI :

You can use the following command to pull the latest image and run from your CLI, remember to set `YOUR-NODE-NAME`

```bash
docker run --platform linux/amd64 --network="host" -v "/var/lib/data" --entrypoint ./tangle \
ghcr.io/webb-tools/tangle/tangle:main \
--base-path=/var/lib/tangle/ \
--chain <PATH_TO_CHAINSPEC> \
--name="YOUR-NODE-NAME" \
--trie-cache-size 0
```

## Run via Docker Compose :

The docker-compose file will spin up a container running tangle standalone node, but you have to set the following environment variables.
Remember to customize your the values depending on your environment and then copy paste this to CLI.

```bash
export RELEASE_VERSION=main
export CHAINSPEC_PATH=/tmp/chainspec
```

After that run :

```bash
docker compose up -d
```