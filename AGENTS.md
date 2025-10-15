# Repository Guidelines

## Project Structure & Module Organization
- `node/` hosts the Substrate node binary; treat `runtime/mainnet` and `runtime/testnet` as the authoritative runtime crates.
- Pallets live under `pallets/…`, with matching runtime APIs in `pallets/*/rpc` and precompiles in `precompiles/`.
- `client/` contains RPC layers and tracing utilities, while `chainspecs/` and `deployment/` hold network configuration and release artifacts.
- Scripts for local orchestration sit in `scripts/`; TypeScript simulations are in `user-simulation/` for scenario-driven testing.

## Build, Test, and Development Commands
- `nix flake develop` opens a fully provisioned shell when using Nix.
- `cargo check -p node` validates core changes quickly; prefer before pushing.
- `cargo build --release --features testnet` produces the testnet node; swap features to target mainnet.
- `./scripts/run-standalone-local.sh --clean` spins up a fresh local testnet with authorities and logs under `/tmp`.
- `npx @acala-network/chopsticks@latest --config=scripts/chopsticks.yml` forks the live chain for rapid iteration.
- From `user-simulation/`, use `yarn install && yarn start` to exercise end-to-end flows against a local node.

## Coding Style & Naming Conventions
- Stick to the pinned toolchain in `rust-toolchain.toml` (Rust 1.86 plus `rustfmt`, `clippy`, and `wasm32-unknown-unknown` target).
- Format via `cargo fmt` (hard tabs, 100-column width) and lint with `cargo clippy --workspace --all-targets`.
- Prefer `snake_case` for modules/functions, `UpperCamelCase` for types, and `SCREAMING_SNAKE_CASE` for constants; mirror existing pallet naming when adding crates.
- Run `dprint fmt` on TOML manifests when touching dependency metadata.

## Testing Guidelines
- `cargo test --workspace` must pass; add focused crates with `-p pallet-name` for faster loops.
- Mirror runtime invariants in Rust unit tests; use benchmarks or fuzzers under `pallets/*/benchmarking` and `pallets/*/fuzzer` when logic is math-heavy.
- Execute `yarn test` in `user-simulation/` before merging features that affect external RPC flows.
- Document new integration scenarios in `scripts/` (e.g., additional Chopsticks configs) when manual steps are required.

## Commit & Pull Request Guidelines
- Follow the existing Conventional Commit pattern (`feat:`, `fix:`, `docs:`, `chore:`) seen in `git log`.
- Keep commits scoped to one logical change and include relevant crate paths in the body when touching multiple pallets.
- PRs should summarize motivation, list test commands run, and link issues or RFCs; attach screenshots only when UX or telemetry dashboards change.
- Request reviews from runtime and node owners for consensus-critical updates; surface migration notes for storage changes.
