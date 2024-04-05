<h1 align="center">Tangle-Subxt</h1>

<p align="center">
    <strong>Rust interface to interact with tangle node via RPC</strong>
    <br />
</p>

<br />

### Downloading metadata from a Substrate node

Use the [`subxt-cli`](https://lib.rs/crates/subxt-cli) tool to download the metadata for your target runtime from a node.

1. Install:
```bash
cargo install subxt-cli@0.34.0 --force
```

2. To Save the metadata of `tangle`:
Run the release build of the `tangle` node, then on another terminal run:

```bash
subxt metadata -f bytes > ./metadata/tangle-mainnet-runtime.scale
```

3. Generating the subxt code from the metadata:

```bash
subxt codegen --file metadata/tangle-mainnet-runtime.scale --derive Clone --derive Eq --derive PartialEq | rustfmt --edition=2018 --emit=stdout > src/tangle_mainnet_runtime.rs
```

### Local Testing

You can run following tests to trigger Job pallet events for local development.

1. Run Local Tangle network 
```bash
./scripts/run-standalone-local.sh --clean
```
2. Run Test
```bash
cargo test test_job_submission_event
```

