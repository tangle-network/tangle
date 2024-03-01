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
cargo install subxt-cli
```

2. To Save the metadata of `tangle`:
Run the release build of the `tangle` node, then on another terminal run:

```bash
subxt metadata -f bytes > ./metadata/tangle-runtime.scale
```

3. Generating the rust code from the metadata:

```bash
cargo build
```

> Tip: See the [build.rs](./build.rs) file to see how everything is being generated.


