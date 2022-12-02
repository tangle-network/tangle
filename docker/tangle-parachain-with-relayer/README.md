# Tangle Relayer + Parachain Docker

## Installation Instructions

A Tangle parachain node and relayer can be spun up quickly using Docker. At the time of writing, the Docker version used was 19.03.6. When connecting to testnet, it will take a few hours/days to completely sync chain. Make sure that your system meets the requirements.

Create a local directory to store the chain data:

```bash
mkdir /var/lib/tangle/
```

## Run via Docker Compose :

The docker-compose file will spin up a container running tangle standalone node and another running relayer, but you have to set the following environment variables.
Remember to customize your the values depending on your environment and then copy paste this to CLI.

```bash
export TANGLE_RELEASE_VERSION=main
export RELAYER_RELEASE_VERSION=0.5.0-rc1
export BASE_PATH=/tmp/data/
export CHAINSPEC_PATH=/tmp/chainspec
export WEBB_PORT=9955
```

After that run :

```bash
docker compose up -d
```