# Examples

You can run this examples to trigger Job pallet events for local development.

- Run Local Tangle network 
```bash
./scripts/run-standalone-local.sh --clean
```
- Create profile
```bash
cargo run --package examples-profile --example create_profile
```
- Deploy Contract
```bash
cargo run --package examples-contracts --example deploy_contract
```
- Submit Proposal
```bash
cargo run --package examples-contracts --example submit_proposal
```
