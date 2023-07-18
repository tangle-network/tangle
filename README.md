<div align="center">
<a href="https://www.webb.tools/">
    
![Webb Logo](./assets/webb_banner_light.png#gh-light-mode-only)
![Webb Logo](./assets/webb_banner_dark.png#gh-dark-mode-only)
  </a>
  </div>
<h1 align="left"> The Tangle Network </h1>
<p align="left">
    <strong>An MPC based bridging system for privacy-preserving applications. </strong>
</p>

<div align="left" >

[![License Apache 2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg?style=flat-square)](https://opensource.org/licenses/Apache-2.0)
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
    <li><a href="#standalone">Run Standalone Testnet</a></li>
    <li><a href="#contribute">Contributing</a></li>
    <li><a href="#license">License</a></li>
  </ul>  
</details>

<h1 id="start"> Getting Started </h1>

The Tangle Network contains runtimes for standalone node featuring Webb's DKG and privacy pallet protocols. If you would like to familiarize yourself with our DKG protocol check out following repo:

- [dkg-substrate](https://github.com/webb-tools/dkg-substrate)

<h1 id="prerequisites"> Prerequisites</h1>

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


<h3 id="standalone"> Run Standalone Local Testnet </h3>

1. Build `tangle-standalone` node.
```bash
  cargo build --release 
```
2. Execute tangle network setup script.
```bash
./scripts/run-standalone-local.sh --clean
```
This should start the local testnet, you can view the logs in /tmp directory for all the authorities and use [polkadotJS](https://polkadot.js.org/apps/#/explorer) to view the running testnet.


<h2 id="contribute"> Contributing </h2>

Interested in contributing to the Webb Relayer Network? Thank you so much for your interest! We are always appreciative for contributions from the open-source community!

If you have a contribution in mind, please check out our [Contribution Guide](./.github/CONTRIBUTING.md) for information on how to do so. We are excited for your first contribution!

<h2 id="license"> License </h2>

Licensed under <a href="LICENSE">Apache 2.0 license</a>.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this crate by you, as defined in the Apache 2.0 license, shall be licensed as above, without any additional terms or conditions.
