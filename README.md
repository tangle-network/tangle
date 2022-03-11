<h1 align="center">Webb Protocol Egg Network</h1>

<p align="center">
    <strong>🕸️  Webb Protocol Egg Network  🧑‍✈️</strong>
    <br />
    <sub> ⚠️ Beta Software ⚠️ </sub>
</p>

<br />

## Overview
The Egg Network is the first parachain specific node featuring Webb's DKG and privacy pallet protocols. It is meant to run with a relay chain.

## Local Egg Testnet Setup

These steps were taken to generate the Rococo setup for the Egg testnet.

### Chainspecs

The following chainspecs are provided for your convenience in `/resources`:

| Chainspecs | Use |
|---|---|
| template-local-plain.json | Used for local testnet development with paraId 2000 |
| rococo-plain.json | Used for Rococo testnet with paraId 2076 |

Keep in mind each of the above mentioned specs are in plain json form and can be arbitrarily updated. 

## Relay Chain

To operate a parathread or parachain, you _must_ connect to a relay chain. Typically you would test
on a local Rococo development network, then move to the testnet, and finally launch on the mainnet.
**Keep in mind you need to configure the specific relay chain you will connect to in your collator
`chain_spec.rs`**. In the following examples, we will use `rococo-local` as the relay network.

**Note:** You may also use `polkadot-launch` in `./polkadot-launch`. Instructions to get started are included there in a `README.md`

### Build Relay Chain

Clone and build [Polkadot](https://github.com/paritytech/polkadot) (be aware of the version tag we used):

```bash
# Clone the Polkadot Repository
git clone https://github.com/paritytech/polkadot.git

# Switch into the Polkadot directory
cd polkadot

# Checkout the proper commit
git checkout v0.9.17

# Build the relay chain Node
cargo build --release

# Check if the help page prints to ensure the node is built correctly
./target/release/polkadot --help
```

### Generate the Relay Chain Chainspec

First, we create the chain specification file (chainspec). Note the chainspec file _must_ be generated on a
_single node_ and then shared among all nodes!

👉 Learn more about chain specification [here](https://substrate.dev/docs/en/knowledgebase/integrate/chain-spec).

```bash
./target/release/polkadot build-spec \
--chain rococo-local \
--raw \
--disable-default-bootnode \
> rococo_local.json
```

### Start Relay Chain

We need _n + 1_ full _validator_ nodes running on a relay chain to accept _n_ parachain / parathread
connections. Here we will start two relay chain nodes so we can have one parachain node connecting in
later.

From the Polkadot working directory:

```bash
# Start Relay `Alice` node
./target/release/polkadot \
--chain ./rococo_local.json \
-d /tmp/relay/alice \
--validator \
--alice \
--port 50555
--ws-port 9944
```

Open a new terminal, same directory:

**Note:** You will have to specify `--bootnodes /ip4/<Alice IP>/tcp/30333/p2p/<Alice Peer ID>` is necessary when operating over the network.  

```bash
# Start Relay `Bob` node
./target/release/polkadot \
--chain ./rococo_local.json \
-d /tmp/relay/bob \
--validator \
--bob \
--port 50556
--ws-port 9945
```

Open a new terminal, same directory:

```bash
# Start Relay `Charlie` node
./target/release/polkadot \
--chain ./rococo_local.json \
-d /tmp/relay/charlie \
--validator \
--charlie \
--port 50557
--ws-port 9946
```

Add more nodes as needed, with non-conflicting ports, DB directories, and validator keys
(`--dave`, etc.).

### Reserve a ParaID

To connect to a relay chain, you must first \_reserve a `ParaId` for your parathread that will
become a parachain. To do this, you will need sufficient amount of currency on the network account
to reserve the ID.

In this example, we will use **`Charlie` development account** where we have funds available.
Once you submit this extrinsic successfully, you can start your collators.

The easiest way to reserve your `ParaId` is via
[Polkadot Apps UI](https://polkadot.js.org/apps/?rpc=ws%3A%2F%2F127.0.0.1%3A9944#/parachains/parathreads)
under the `Parachains` -> `Parathreads` tab and use the `+ ParaID` button.

## Parachain

### Select the Correct Relay Chain

To operate your parachain, you need to specify the correct relay chain you will connect to in your
collator `chain_spec.rs`. Specifically you pass the command for the network you need in the
`Extensions` of your `ChainSpec::from_genesis()` [in the code](node/src/chain_spec.rs#78-81).

```rust
Extensions {
	relay_chain: "rococo-local".into(), // You MUST set this to the correct network!
	para_id: id.into(),
},
```

In the following examples, we will use the `rococo-local` relay network we setup in the last section.

### Export the DKG Genesis and Runtime

We first generate the **genesis state** and **genesis wasm** needed for the parachain registration.

> **NOTE**: we have set the `para_ID` to be **2000** here for local testnets. This _must_ be unique for all parathreads/chains
> on the relay chain you register with. You _must_ reserve this first on the relay chain for the
> testnet or mainnet.

```bash
# Build the dkg node (from it's top level dir)
cargo build --release -p egg-collator

# The following instructions outline how to build chain spec,
# wasm and genesis state for local setup. These files will be used
# during start up.

# Build local chainspec
# You may also use the chainspec provided in ./resources  
./target/release/egg-collator build-spec \
--disable-default-bootnode > ./resources/template-local-plain.json

# Build the raw chainspec file
./target/release/egg-collator build-spec \
--chain=./resources/template-local-plain.json \
--raw --disable-default-bootnode > ./resources/template-local-raw.json

# Export genesis state to `./resources`, using 2000 as the ParaId
./target/release/egg-collator export-genesis-state --chain=./resources/template-local-raw.json > ./resources/para-2000-genesis

# Export the genesis wasm
./target/release/egg-collator export-genesis-wasm > ./resources/para-2000-wasm
```

For building chainspec for Rococo Egg Testnet you need to pass the chain argument during the intial build spec.

```
# Note: This uses paraId 2076 and target Rococo relay chain
./target/release/egg-collator build-spec --disable-default-bootnode --chain=egg-rococo > ./resources/rococo-plain.json
```

### Start a Egg Collator Node

From the dkg-substrate working directory:

```bash
# NOTE: this command assumes the chain spec is in a directory named `polkadot`
# that is at the same level of the template working directory. Change as needed.
#
# It also assumes a ParaId of 2000. Change as needed.
./target/release/egg-collator \
--base-path /tmp/parachain/alice \
--collator \
--alice \
--force-authoring \
--ws-port 9948 \
--chain ./resources/template-local-raw.json \
-- \
--execution wasm \
--chain ../polkadot/rococo_local.json
```

### Parachain Registration

Now that you have two relay chain nodes, and a parachain node accompanied with a relay chain light
client running, the next step is to register the parachain in the relay chain with the following
steps (for detail, refer to the [Substrate Cumulus Worship](https://substrate.dev/cumulus-workshop/#/en/3-parachains/2-register)):

-   Goto [Polkadot Apps UI](https://polkadot.js.org/apps/#/explorer), connecting to your relay chain.

-   Execute a sudo extrinsic on the relay chain by going to `Developer` -> `sudo` page.

-   Pick `paraSudoWrapper` -> `sudoScheduleParaInitialize(id, genesis)` as the extrinsic type,
    shown below.

-   Set the `id: ParaId` to 2,000 (or whatever ParaId you used above), and set the `parachain: Bool`
    option to **Yes**.

-   For the `genesisHead`, drag the genesis state file exported above, `para-2000-genesis`, in.

-   For the `validationCode`, drag the genesis wasm file exported above, `para-2000-wasm`, in.

### Restart the Collator

The DKG node may need to be restarted to get it functioning as expected. After a
[new epoch](https://wiki.polkadot.network/docs/en/glossary#epoch) starts on the relay chain,
your parachain will come online. Once this happens, you should see the collator start
reporting _parachain_ blocks:

**Note the delay here!** It may take some time for your relay chain to enter a new epoch.
