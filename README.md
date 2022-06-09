<h1 align="center">Webb Protocol Egg Network 🕸️ </h1>
<div align="center">
<a href="https://www.webb.tools/">
    <img alt="Webb Logo" src="./assets/webb-icon.svg" width="15%" height="30%" />
  </a>
  </div>
<p align="center">
    <strong>🚀 Threshold ECDSA Distributed Key Generation Protocol 🔑 </strong>
    <br />
    <sub> ⚠️ Beta Software ⚠️ </sub>
</p>

<div align="center" >

[![License Apache 2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg?style=flat-square)](https://opensource.org/licenses/Apache-2.0)
[![Twitter](https://img.shields.io/twitter/follow/webbprotocol.svg?style=flat-square&label=Twitter&color=1DA1F2)](https://twitter.com/webbprotocol)
[![Telegram](https://img.shields.io/badge/Telegram-gray?logo=telegram)](https://t.me/webbprotocol)
[![Discord](https://img.shields.io/discord/833784453251596298.svg?style=flat-square&label=Discord&logo=discord)](https://discord.gg/cv8EfJu3Tn)

</div>

<!-- TABLE OF CONTENTS -->
<h2 id="table-of-contents"> 📖 Table of Contents</h2>

<details open="open">
  <summary>Table of Contents</summary>
  <ul>
    <li><a href="#start"> Getting Started</a></li>
    <ul>
        <li><a href="#prerequisites">Prerequisites</a></li>
        <li><a href="#install">Installation</a></li>
        <ul>
          <li><a href="#trouble">Troubleshooting Apple Silicon</a>
          </li>
        </ul>
    </ul>
    <li><a href="#usage">Usage</a></li>
    <ul>
        <li><a href="#chainspec">Chainspecs</a></li>
        <li><a href="#launch">Run local testnet with polkadot-launch</a></li>
        <li><a href="#standalone">Standalone Testnet</a></li>
    </ul>
    <li><a href="#manual">Manual Local Parachain Setup</a></li>
        <ul>
        <li><a href="#relay">Relay Chain</a></li>
        <li><a href="#parachain">Parachain</a></li>
    </ul>
    <li><a href="#contribute">Contributing</a></li>
    <li><a href="#license">License</a></li>
  </ul>  
</details>

<h1 id="start"> Getting Started  🎉 </h1>

The Egg Network contains runtimes for both standalone and parachain nodes featuring Webb's DKG and privacy pallet protocols. 

## Prerequisites

This guide uses <https://rustup.rs> installer and the `rustup` tool to manage the Rust toolchain.

First install and configure `rustup`:

```bash
# Install
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
# Configure
source ~/.cargo/env
```

Configure the Rust toolchain to default to the latest stable version, add nightly and the nightly wasm target:

```bash
rustup default nightly
rustup update
rustup update nightly
rustup target add wasm32-unknown-unknown --toolchain nightly
```

Great! Now your Rust environment is ready! 🚀🚀

**Note:** You may need additional dependencies, checkout [substrate.io](https://docs.substrate.io/v3/getting-started/installation) for more information.

## Installation 💻

Once the development environment is set up, build the DKG. This command will build the [Wasm Runtime](https://docs.substrate.io/v3/advanced/executor/#wasm-execution) and [native](https://docs.substrate.io/v3/advanced/executor/#native-execution) code:

```bash
cargo build --release
```

> NOTE: You _must_ use the release builds! The optimizations here are required
> as in debug mode, it is expected that nodes are not able to run fast enough to produce blocks.

You will now have two runtimes built in `target/release/` dir:

1. `egg-collator`: Parachain node.
2. `egg-standalone-node`: Standalone node, used in the current standalone Egg network.
### Troubleshooting for Apple Silicon users

Install Homebrew if you have not already. You can check if you have it installed with the following command:

```bash
brew help
```

If you do not have it installed open the Terminal application and execute the following commands:

```bash
# Install Homebrew if necessary https://brew.sh/
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/master/install.sh)"

# Make sure Homebrew is up-to-date, install openssl
brew update
brew install openssl
```

❗ **Note:** Native ARM Homebrew installations are only going to be supported at `/opt/homebrew`. After Homebrew installs, make sure to add `/opt/homebrew/bin` to your PATH.

```bash
echo 'export PATH=/opt/homebrew/bin:$PATH' >> ~/.bash_profile
```

In order to build **dkg-substrate** in `--release` mode using `aarch64-apple-darwin` Rust toolchain you need to set the following environment variables:

```bash
echo 'export RUSTFLAGS="-L /opt/homebrew/lib"' >> ~/.bash_profile
```

Ensure `gmp` dependency is installed correctly.

```
brew install gmp
```

<h1 id="usage"> Usage </h1>

<h3 id="chainspec"> Chainspecs </h3>

The following chainspecs are provided for your convenience in `/resources`:

| Chainspecs | Use | Target
|---|---|---|
| template-local-plain.json | Used for local testnet development with paraId 2000 | `--chain=template-rococo`
| rococo-plain.json | Used for Rococo testnet with paraId 2003 | `--chain=egg-rococo`
| arana-standalone-plain.json | Used for standalone egg network | `--chain=eggnet`

Keep in mind each of the above mentioned specs are in plain json form and can be arbitrarily updated. The raw spec versions are included in `resources/` for your convenience. To learn more about chainspecs checkout the [docs](https://docs.substrate.io/v3/runtime/chain-specs/) 🎓.

<h2 style="border-bottom:none"> Quick Start ⚡ </h2>

<h3 id="launch"> Run local testnet with <a href="https://github.com/paritytech/polkadot-launch">polkadot-launch</a> ☄️</h3>

The fastest way to set up the DKG to run as a parachain is to make use of [polkadot-launch](https://github.com/paritytech/polkadot-launch). Follow the below steps to get up and running! 🏃

**Install polkadot-launch:**

```
npm install -g polkadot-launch
```

**Update configuration script:**

1. Run: `cd scripts/polkadot-launch`
2. Update the `bin` field for `relaychain` and `parachains` to point to appropriate paths. **Note:** You will need to have a built Polkadot binary. For Polkadot installation instructions follow the steps outlined [here](https://github.com/paritytech/polkadot).
3. Update ports and debug logs as you see fit.

**Launch Polkadot relay chain and DKG parachain:**

```bash
polkadot-launch dkg-launch.json
```

If everything went well you should see `POLKADOT LAUNCHED SUCCESSFULLY 🚀`. To follow the DKG parachain logs:

```bash
tail -f 9988.log
```

<h3 id="standalone"> Standalone Local Testnet </h3>

Currently the easiest way to run the DKG is to use a 3-node local testnet using `egg-standalone-node`. We will call those nodes `Alice`, `Bob` and `Charlie`. Each node will use the built-in development account with the same name, i.e. node `Alice` will use the `Alice` development account and so on. Each of the three accounts has been configured as an initial authority at genesis. So, we are using three validators for our testnet.

`Alice` is our bootnode and is started like so:

```
RUST_LOG=dkg=trace ./target/release/egg-standalone-node  --base-path /tmp/standalone/alice --alice
```

`Bob` is started like so:

```
RUST_LOG=dkg=trace ./target/release/egg-standalone-node  --base-path /tmp/standalone/bob --bob
```

`Charlie` is started like so:

```
RUST_LOG=dkg=trace ./target/release/egg-standalone-node --base-path /tmp/standalone/charlie --charlie
```

Great you are now running a 3-node standalone test network!

<h2 id="manual"> Manual Local Parachain Setup </h2>

The below instructions outline the steps required to setup a local test network with a 2-validator relay chain, registered DKG parachain, and 3-collator nodes.
 
<h3 id="relay"> Relay Chain </h3>

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
git checkout v0.9.23

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

Add more nodes as needed, with non-conflicting ports, DB directories, and validator keys
(`--charlie`, etc.).

### Reserve a ParaID

To connect to a relay chain, you must first \_reserve a `ParaId` for your parathread that will
become a parachain. To do this, you will need sufficient amount of currency on the network account
to reserve the ID.

In this example, we will use **`Charlie` development account** where we have funds available.
Once you submit this extrinsic successfully, you can start your collators.

The easiest way to reserve your `ParaId` is via
[Polkadot Apps UI](https://polkadot.js.org/apps/?rpc=ws%3A%2F%2F127.0.0.1%3A9944#/parachains/parathreads)
under the `Parachains` -> `Parathreads` tab and use the `+ ParaID` button.

<h2 id="parachain"> Parachain </h2>

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

<h2 id="contribute"> Contributing </h2>

Interested in contributing to the Webb Relayer Network? Thank you so much for your interest! We are always appreciative for contributions from the open-source community!

If you have a contribution in mind, please check out our [Contribution Guide](./.github/CONTRIBUTING.md) for information on how to do so. We are excited for your first contribution!

<h2 id="license"> License </h2>

Licensed under <a href="LICENSE">Apache 2.0 license</a>.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this crate by you, as defined in the Apache 2.0 license, shall be licensed as above, without any additional terms or conditions.