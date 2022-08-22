# How to generate testnet files

## Relaychain spec

1. Build the latest relaychain 
    ```
    # Clone the Polkadot Repository, with correct version
    git clone --depth 1 --branch release-v0.9.27
    cd polkadot
    cargo b -r
    ```

2. Generate the relaychain chainspec

    ```
    ./target/release/polkadot build-spec --chain rococo-local --disable-default-bootnode --raw > rococo-local-cfde.json
    ```

3. Replace the created file with the file in /resources/testnet/rococo-local-cfde.json


## Parachain spec

1. Build the latest parachain (we are not passing the argument `--chain` since the default is `local-testnet`)

    ```
    cargo b -p tangle-collator -r
    ```

2. Generate raw chainspec and artifacts (we are not passing the argument `--chain` since the default is `local-testnet`). Remember to generate this on the same machine you will be running the initial collator on.

    ```
    /usr/local/bin/tangle-collator build-spec --disable-default-bootnode --raw > template-local-raw.json
    /usr/local/bin/tangle-collator export-genesis-state > genesis-state-local
    /usr/local/bin/tangle-collator export-genesis-wasm > genesis-wasm-local
    ```