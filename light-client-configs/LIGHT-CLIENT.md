## Running Tangle Node with Webb Light Client Relayer

Following these steps will allow you to run Tangle Standalone Node with Webb Light Client Relayer included.


1. Compile the tangle standalone node with light client feature
```sh
cargo build --release -p tangle-standalone --features light-client
```
2. Set you Infura API Key

```bash
export ETH1_INFURA_API_KEY="your_infura_key"
```
3. Create your Relayer configuration directory

```sh
mkdir -p light-client-configs
touch light-client-configs/block-relay-config.toml
touch light-client-configs/init-pallet-config.toml
```

> You can check sample configuration files here [Light Client Relayer configuration](../light-client-configs/block-relay-config.toml)


4. Start Tangle Node with the light client relayer config.
Make sure to provide correct path for configuration files

```sh
./target/release/tangle-standalone --tmp --chain local --validator --alice \
--rpc-cors all --rpc-methods=unsafe --rpc-port 9944 \
--light-client-init-pallet-config-path=./light-client-configs/init-pallet-config.toml \
--light-client-relay-config-path=./light-client-configs/block-relay-config.toml \
```

Now, you can see light client submitting beacon headers to tangle.
