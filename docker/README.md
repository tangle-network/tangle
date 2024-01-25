# Tangle Docker

## Installation Instructions

When connecting to testnet, it will take a few hours/days to completely sync chain. Make sure that your system meets the requirements.

## Run via CLI :

Make sure to set `<RELEASE_VERSION>` to your desired version.

You can use the following command to pull the latest image and run from your CLI, remember to set `YOUR-NODE-NAME`

```bash
docker run --network="host" -v "/var/lib/data" \
ghcr.io/webb-tools/tangle/tangle:<RELEASE_VERSION> \
--chain tangle-testnet \
--name="YOUR-NODE-NAME" \
--trie-cache-size 0
--telemetry-url "wss://telemetry.polkadot.io/submit/ 0"
```

## Run via Docker Compose :

The docker-compose file will spin up a container running tangle standalone node, but you have to set the following environment variables.
Remember to customize your the values depending on your environment and then copy paste this to CLI.

```bash
export RELEASE_VERSION=<RELEASE_VERSION>
```

After that run :

```bash
docker compose up -d
```