## Running Tangle Node with Webb Relayer

Following these steps will allow you to run Tangle Standalone Node with Webb Relayer included.

> Note that by default, running tangle standalone normally won't start the relayer, you may see
something like the Following: `Error: Not Starting Webb Relayer Gadget. No Config Directory Specified`
which is totally OK if you are not looking into running the webb relayer gadget.

1. Compile the tangle standalone node with relayer feature
```sh
cargo build --release -p tangle --features relayer
```
2. Create your .env file to store your secrets (during the development)

```bash
# internal Node SURI is the secret that is corresponds
# to your controller account of the running node.
# that key will be used to submit proposals to the dkg-proposals
# pallet.
INTERNAL_NODE_SURI=//Alice
```
3. Create your Relayer configuration directory

```sh
mkdir -p relayer-config && touch relayer-config/example.toml
```

> For all possible configuration, please refer to [Relayer configuration](https://github.com/webb-tools/relayer/tree/develop/config#readme)

4. Edit your example config file
```toml
port = 9955

# Controls what features are enabled in the relayer system
[features]
# if you are an authority, this always true.
governance-relay = true
data-query = true
private-tx-relay = true

[evm.goerli]
name = "goerli"
http-endpoint = "https://rpc.ankr.com/eth_goerli"
ws-endpoint = "wss://rpc.ankr.com/eth_goerli"
chain-id = 5
enabled = true
block-confirmations = 2
# The private key of the account that will be used to sign transactions
# If not set, we will use the Keystore to get the ECDSA private key.
# private-key = "$PRIVATE_KEY"

[[evm.goerli.contracts]]
contract = "VAnchor"
address = "0x38e7aa90c77f86747fab355eecaa0c2e4c3a463d"
deployed-at = 8703495
events-watcher = { enabled = true, polling-interval = 15000 }
proposal-signing-backend = { type = "DKGNode", chain-id = 0 }

[substrate.internal]
name = "internal"
chain-id = 0
http-endpoint = "http://localhost:9944"
ws-endpoint = "ws://localhost:9944"
suri = "$INTERNAL_NODE_SURI"
enabled = true

[[substrate.internal.pallets]]
pallet = "DKG"
events-watcher = { enabled = true, polling-interval = 3000, print-progress-interval = 30000 }

[[substrate.internal.pallets]]
pallet = "DKGProposalHandler"
events-watcher = { enabled = true, polling-interval = 3000, print-progress-interval = 30000 }
```
5. Start Tangle Node with the relayer config.

```sh
./target/release/tangle --tmp --chain local --validator --alice --rpc-cors all --rpc-methods=unsafe --rpc-port 9944 --relayer-config-dir ./relayer-config
```

Now, you should notice that the error about not starting the relayer gadget is gone, and to verify everything is working
you can now access: [Relayer Info Endpoint](http://localhost:9955/api/v1/info)
