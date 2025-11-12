# Tangle Runtime Migration Testing with Chopsticks

This directory contains tools for rigorously testing the Polkadot SDK stable2503 runtime upgrade using Chopsticks and try-runtime-cli.

## Quick Start

### 1. Install Dependencies

```bash
# Install try-runtime-cli
cargo install --git https://github.com/paritytech/try-runtime-cli --locked

# Install Chopsticks
npm install -g @acala-network/chopsticks
```

### 2. Build Runtime with try-runtime Feature

```bash
# From repository root
cargo build --release --features try-runtime --package tangle-mainnet-runtime
cargo build --release --features try-runtime --package tangle-testnet-runtime
```

### 3. Run Quick Test

```bash
cd chopsticks
./scripts/test-migration.sh mainnet
```

## Directory Structure

```
chopsticks/
├── configs/
│   ├── mainnet.yml          # Mainnet fork configuration
│   └── testnet.yml          # Testnet fork configuration
├── scripts/
│   ├── test-migration.sh    # Automated migration testing
│   ├── fork-mainnet.sh      # Launch mainnet fork
│   └── fork-testnet.sh      # Launch testnet fork
├── db/                       # Cached chain state (gitignored)
├── snapshots/                # Chain snapshots (gitignored)
├── docs/
│   └── MIGRATION_GUIDE.md   # Detailed migration testing guide
└── README.md                 # This file
```

## Testing Workflow

### Phase 1: Fork Networks

**Fork Mainnet:**
```bash
npx @acala-network/chopsticks --config=./configs/mainnet.yml --port=8000
```

**Fork Testnet:**
```bash
npx @acala-network/chopsticks --config=./configs/testnet.yml --port=8001
```

### Phase 2: Test Migrations

**Test Mainnet Migration:**
```bash
RUST_LOG=runtime=debug try-runtime \
  --runtime ../target/release/wbuild/tangle-mainnet-runtime/tangle_mainnet_runtime.wasm \
  on-runtime-upgrade live --uri ws://localhost:8000
```

**Test Testnet Migration:**
```bash
RUST_LOG=runtime=debug try-runtime \
  --runtime ../target/release/wbuild/tangle-testnet-runtime/tangle_testnet_runtime.wasm \
  on-runtime-upgrade live --uri ws://localhost:8001
```

### Phase 3: Validate Results

Check for:
- ✅ All pre-upgrade hooks pass
- ✅ All post-upgrade hooks pass
- ✅ Storage versions updated correctly
- ✅ No decoding failures
- ✅ Weight within block limits

## Key Migration Issues to Address

### 1. Currency Trait Bound Issues (CRITICAL)

**Status:** Mainnet migrations are currently commented out due to Currency trait issues

**Location:** `runtime/mainnet/src/lib.rs:29-30`

**Action Required:** Update migrations to use `fungible` traits instead of deprecated `Currency` trait

### 2. Missing Polkadot SDK Migrations

Check if these pallets require migrations from stable2503:
- [ ] pallet-staking (Currency→Fungible migration)
- [ ] pallet-session (session keys structure)
- [ ] pallet-balances (storage format updates)
- [ ] pallet-assets (NextAssetId removal - already done in testnet)

### 3. Custom Pallet Migrations

Current migrations:
- ✅ `pallet-multi-asset-delegation`: DelegatorMetadata migration
- ✅ `pallet-rewards`: Percentage→Perbill migration
- ✅ Testnet: MigrateSessionKeys, RemoveNextAssetId

**Verify:** All custom pallets have correct storage versions

## Identifying Missing Migrations

### Method 1: Review try-runtime Output

```bash
# Look for storage version mismatches
RUST_LOG=runtime=trace try-runtime ... 2>&1 | grep -i "version\|migration"
```

### Method 2: Check Polkadot SDK Migrations

```bash
# Clone and check SDK migrations
git clone https://github.com/paritytech/polkadot-sdk.git
cd polkadot-sdk && git checkout stable2503
find substrate/frame -name "migrations.rs" | xargs cat
```

### Method 3: Storage Version Audit

```bash
# Check all pallet storage versions
grep -r "const STORAGE_VERSION" ../pallets/ ../runtime/
```

## Common Issues

### Issue: "trait bound Currency not satisfied"

**Solution:** Update to use fungible traits:
```rust
// Old
use frame_support::traits::Currency;

// New
use frame_support::traits::fungible::{Inspect, Mutate};
```

### Issue: "Storage version mismatch"

**Solution:** Add VersionedMigration to Executive tuple

### Issue: "PoV size exceeds limit"

**Solution:** Implement multi-block migration with cursor

## Resources

- [Polkadot SDK Migrations Guide](https://paritytech.github.io/polkadot-sdk/master/polkadot_sdk_docs/reference_docs/frame_runtime_upgrades_and_migrations/)
- [try-runtime-cli GitHub](https://github.com/paritytech/try-runtime-cli)
- [Chopsticks GitHub](https://github.com/AcalaNetwork/chopsticks)
- [Full Migration Testing Guide](./docs/MIGRATION_GUIDE.md)

## Support

For issues or questions:
- Check `./docs/MIGRATION_GUIDE.md` for detailed instructions
- Review Polkadot SDK stable2503 release notes
- Consult Substrate Stack Exchange

---

**IMPORTANT:** Always test on testnet fork first, then mainnet fork, before deploying to live networks.
