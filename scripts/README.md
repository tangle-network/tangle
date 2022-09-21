## How to use the scripts

This a simple readme file that will outline the required steps to run the included scripts.


### Run a standalone Tangle Network

For running a standalone tangle network, Here are the steps that you need to follow:

1. Compile the standalone in the `release` mode:
```sh
cargo b -rp tangle-standalone-node
```
2. Execute the `run-standalone.sh` script:
```sh
./scripts/run-standalone --clean
```

Note that this will start a clean network state, if you want to continue running on an old state (using old database)
just omit the `--clean` flag.

### Run Tangle Parachain

For running the Tangle Network parachain, which usually involving a little bit of steps, Here is what you need to do:

1. Build Relay Chain (if you have not already)
```sh
# Clone the Polkadot Repository
git clone https://github.com/paritytech/polkadot.git

# Switch into the Polkadot directory
cd polkadot

# Checkout the proper commit
git checkout v0.9.28 # or the one configured in our Cargo.lock

# Build the relay chain Node
cargo build --release

# Check if the help page prints to ensure the node is built correctly
./target/release/polkadot --help
```
2. Compile the Parachain collator binary:
```sh
cargo b -rp tangle-collator
```
3. Install [Deno](https://deno.land) (if it is not installed):
  * Mac: `brew install deno`
  * Others: See the steps [here](https://deno.land/#installation)
4. Run the `run-parachain.ts` script and follow the on screen instructions:

```sh
./scripts/run-parachain.ts
```

