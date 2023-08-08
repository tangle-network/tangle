<div align="center">
<a href="https://www.webb.tools/">
    
![Webb Logo](./assets/webb_banner_light.png#gh-light-mode-only)
![Webb Logo](./assets/webb_banner_dark.png#gh-dark-mode-only)
  </a>
  </div>
<h1 align="left"> The Tangle Network </h1>
<p align="left">
    <strong>An MPC based governance system for cross-chain zero-knowledge applications. </strong>
</p>

<div align="left" >

[![Twitter](https://img.shields.io/twitter/follow/webbprotocol.svg?style=flat-square&label=Twitter&color=1DA1F2)](https://twitter.com/webbprotocol)
[![Telegram](https://img.shields.io/badge/Telegram-gray?logo=telegram)](https://t.me/webbprotocol)
[![Discord](https://img.shields.io/discord/833784453251596298.svg?style=flat-square&label=Discord&logo=discord)](https://discord.gg/cv8EfJu3Tn)

</div>

<!-- TABLE OF CONTENTS -->
<h2 id="table-of-contents"> Table of Contents</h2>

<details open="open">
  <summary>Table of Contents</summary>
  <ul>
    <li><a href="#start"> Getting Started</a></li>
    <li><a href="#prerequisites">Prerequisites</a></li>
    <li><a href="#nix">Installation using Nix</a></li>
    <li><a href="#standalone">Run Standalone Testnet</a></li>
    <li><a href="#relayer">Running Standalone Node with Webb Relayer</a></li>
    <li><a href="#contribute">Contributing</a></li>
    <li><a href="#license">License</a></li>
  </ul>  
</details>

<h1 id="start"> Getting Started </h1>

The Tangle Network contains runtimes for standalone node featuring Webb's DKG and privacy pallet protocols.If you would like to familiarize yourself with Tangle and DKG protocol check out following repo and docs:

- [Dkg Substrate Protocol](https://github.com/webb-tools/dkg-substrate)
- [Tangle Docs](https://docs.webb.tools/docs/projects/tangle-network/overview/)
- [Tangle Website](https://tangle.webb.tools/)


<h2 id="prerequisites"> Prerequisites</h2>

This guide uses <https://rustup.rs> installer and the `rustup` tool to manage the Rust toolchain.

First install and configure `rustup`:

```bash
# Install
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
# Configure
source ~/.cargo/env
```
Great! Now your Rust environment is ready!

**Note:** You may need additional dependencies, checkout [substrate.io](https://docs.substrate.io/v3/getting-started/installation) for more information.

<h2 id="nix"> Installation using Nix </h2>
If you want to use Nix for development, please follow following instructions

1. Install [Nix](https://nixos.org/download.html)
2. Enable Flakes (if you are not already see here: [Flakes](https://nixos.wiki/wiki/Flakes))
3. If you have [`direnv`](https://github.com/nix-community/nix-direnv#installation) installed, everything should work out of the box.
4. Alternatively, you can run `nix flake develop` in the root of this repo to get a shell with all the dependencies installed.


<h2 id="standalone"> Run Standalone Local Testnet </h2>

1. Build `tangle-standalone` node.
```bash
  cargo build --release 
```
2. Execute tangle network setup script.
```bash
./scripts/run-standalone-local.sh --clean
```
This should start the local testnet, you can view the logs in /tmp directory for all the authorities and use [polkadotJS](https://polkadot.js.org/apps/#/explorer) to view the running testnet.


<h2 id="relayer"> Run Standalone Node with Webb Relayer</h2>

Tangle standalone node ships with [Webb Relayer](https://github.com/webb-tools/relayer) in the node itself, which is useful to run them together.
For instructions on how to run Tangle Standalone Node with Webb Relayer, Please refer to [this document](./RELAYER.md).

<h2 id="contribute"> Contributing </h2>

Interested in contributing to the Webb Relayer Network? Thank you so much for your interest! We are always appreciative for contributions from the open-source community!

If you have a contribution in mind, please check out our [Contribution Guide](./.github/CONTRIBUTING.md) for information on how to do so. We are excited for your first contribution!

<h2 id="license"> License </h2>

Licensed under <a href="LICENSE">GNU General Public License v3.0</a>.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this crate by you, as defined in the GNU General Public License v3.0 license, shall be licensed as above, without any additional terms or conditions.

## Troubleshooting
The linking phase may fail due to not finding libgmp (i.e., "could not find library -lgmp") when building on a mac M1. To fix this problem, run:

```bash
brew install gmp
# make sure to run the commands below each time when starting a new env, or, append them to .zshrc
export LIBRARY_PATH=$LIBRARY_PATH:/opt/homebrew/lib
export INCLUDE_PATH=$INCLUDE_PATH:/opt/homebrew/include
```

