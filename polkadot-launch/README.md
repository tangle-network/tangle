### Run local testnet with [polkadot-launch](https://github.com/paritytech/polkadot-launch)

Install polkadot-launch using npm or yarn

```
npm install -g polkadot-launch
```

Build Polkadot for relay chain:

```bash
git clone -n https://github.com/paritytech/polkadot.git
git checkout v0.9.22
cargo build --release
```

Build Egg-net parachain:

```bash
cargo build --release --p egg-collator
```

Update `dkg-launch.json` to relevant paths for Polkadot and Parachain binary.

```bash
"bin": "<YOUR-PATH>"
```

Launch local Polkadot relay chain and Egg-net parachain:

```bash
polkadot-launch dkg-launch.json
```

Expected result:

```
POLKADOT LAUNCH SUCCESSFUL 🚀
```
