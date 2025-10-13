# Guide: Integrating Real pallet-rewards into Services E2E Tests

## Current State vs Desired State

### Current: MockRewardsManager (Thread-Local Storage)
```rust
// pallets/services/src/mock.rs (lines 451-555)
thread_local! {
    static PENDING_REWARDS: RefCell<BTreeMap<AccountId, Vec<(u64, Balance)>>> = ...;
}

pub struct MockRewardsManager;
impl RewardRecorder for MockRewardsManager {
    fn record_reward(...) {
        // ❌ Stores in thread-local, NOT runtime storage
        PENDING_REWARDS.with(|rewards| { ... });
    }
}

// Tests manually simulate claims
fn simulate_operator_claim(...) {
    let pending = MockRewardsManager::get_pending_rewards(operator);
    Balances::transfer(...);  // ❌ Manual transfer
    MockRewardsManager::clear_pending_rewards(operator);  // ❌ Manual cleanup
}
```

### Desired: Real pallet-rewards (Runtime Storage)
```rust
// Would use actual pallet
type RewardRecorder = Rewards;

// Tests use real extrinsic
assert_ok!(Rewards::claim_rewards(RuntimeOrigin::signed(operator)));
// ✅ Real storage operations
// ✅ Real transfer logic
// ✅ Real error handling
```

## Step-by-Step Integration

### Step 1: Add pallet-rewards to Runtime

**File:** `pallets/services/src/mock.rs`

Add parameter types before the Config impl:
```rust
parameter_types! {
    pub RewardsPalletId: PalletId = PalletId(*b"py/rwrds");  // 8 bytes
    pub const MaxDepositCap: Balance = 1_000_000_000_000;
    pub const MaxIncentiveCap: Balance = 100_000_000;
    pub const MaxApy: Perbill = Perbill::from_percent(20);
    pub const MinDepositCap: Balance = 0;
    pub const MinIncentiveCap: Balance = 0;
    pub const MaxPendingRewardsPerOperator: u32 = 100;  // Already exists
}
```

Add Config implementation after `MultiAssetDelegation::Config`:
```rust
impl pallet_rewards::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type AssetId = AssetId;
    type Currency = Balances;
    type PalletId = RewardsPalletId;
    type VaultId = u32;  // Simple u32 vault IDs
    type DelegationManager = MultiAssetDelegation;  // Use real pallet!
    type ForceOrigin = frame_system::EnsureRoot<AccountId>;
    type MaxApy = MaxApy;
    type MaxDepositCap = MaxDepositCap;
    type MaxIncentiveCap = MaxIncentiveCap;
    type MinIncentiveCap = MinIncentiveCap;
    type MinDepositCap = MinDepositCap;
    type MaxVaultNameLength = ConstU32<64>;
    type MaxVaultLogoLength = ConstU32<256>;
    type VaultMetadataOrigin = frame_system::EnsureSigned<AccountId>;
    type MaxPendingRewardsPerOperator = MaxPendingRewardsPerOperator;
    type WeightInfo = ();
}
```

### Step 2: Add Rewards to construct_runtime!

```rust
construct_runtime!(
    pub enum Runtime {
        System: frame_system,
        Timestamp: pallet_timestamp,
        Balances: pallet_balances,
        Assets: pallet_assets,
        Services: pallet_services,
        EVM: pallet_evm,
        Ethereum: pallet_ethereum,
        Session: pallet_session,
        Staking: pallet_staking,
        Historical: pallet_session_historical,
        MultiAssetDelegation: pallet_multi_asset_delegation,
        Rewards: pallet_rewards,  // ← ADD THIS
    }
);
```

### Step 3: Update Services Config

Change RewardRecorder from MockRewardsManager to Rewards:

```rust
impl pallet_services::Config for Runtime {
    // ... other config items remain the same
    type RewardRecorder = Rewards;  // ← Change from MockRewardsManager
    type RewardsManager = MockRewardsManager;  // Keep this for now
    // ... rest unchanged
}
```

### Step 4: Remove/Keep MockRewardsManager for RewardsManager trait

The MockRewardsManager is still needed for the `RewardsManager` trait (delegation tracking), but NOT for `RewardRecorder`:

```rust
// Keep this for RewardsManager trait
pub struct MockRewardsManager;
impl RewardsManager<...> for MockRewardsManager {
    fn record_delegate(...) { ... }  // Keep these
    fn record_undelegate(...) { ... }
    // ... other delegation methods
}

// REMOVE the RewardRecorder impl - using real Rewards pallet now!
// DELETE lines 541-555 in services/src/mock.rs
```

### Step 5: Initialize Rewards Pallet Account in Tests

In `new_test_ext_raw_authorities()`, ensure rewards pallet account has funds:

```rust
pub fn new_test_ext_raw_authorities(authorities: Vec<AccountId>) -> sp_io::TestExternalities {
    // ... existing setup

    let rewards_account = <Rewards as tangle_primitives::traits::RewardRecorder<_, _, _>>::account_id();
    balances.push((rewards_account, 1_000_000_u128));  // Give pallet initial funds

    pallet_balances::GenesisConfig::<Runtime> { balances }
        .assimilate_storage(&mut t)
        .unwrap();

    // ... rest of setup
}
```

### Step 6: Update E2E Tests

**File:** `pallets/services/src/tests/operator_rewards_e2e.rs`

Remove the `simulate_operator_claim` helper and use real extrinsic:

```rust
// DELETE THIS:
fn simulate_operator_claim(operator: &AccountId, rewards_account: &AccountId) -> Balance {
    let pending_rewards = MockRewardsManager::get_pending_rewards(operator);
    // ... manual transfer logic
}

// REPLACE WITH real extrinsic calls:
#[test]
fn test_full_e2e_native_payment_with_claim() {
    new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE]).execute_with(|| {
        // Setup remains same...

        // Payment processing (already real)
        assert_ok!(Services::charge_payment(&customer, &customer, payment));
        assert_ok!(Services::distribute_service_payment(&service, &developer, payment, &model));

        // ✅ Use real claim_rewards extrinsic
        let operator_before = Balances::free_balance(&operator);

        assert_ok!(Rewards::claim_rewards(RuntimeOrigin::signed(operator.clone())));

        let operator_after = Balances::free_balance(&operator);
        assert_eq!(operator_after - operator_before, 8_500, "Operator should receive 8,500");

        // ✅ Verify events
        System::assert_has_event(RuntimeEvent::Rewards(
            pallet_rewards::Event::OperatorRewardsClaimed {
                operator: operator.clone(),
                amount: 8_500,
            }
        ));
    });
}
```

### Step 7: Add New Test Cases

Now you can test real scenarios:

```rust
#[test]
fn test_insufficient_rewards_pallet_balance() {
    new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE]).execute_with(|| {
        let operator = mock_pub_key(BOB);
        let customer = mock_pub_key(ALICE);
        let rewards_account = Rewards::account_id();

        // Drain rewards pallet (keep only existential deposit)
        let rewards_balance = Balances::free_balance(&rewards_account);
        Balances::make_free_balance_be(&rewards_account, 1);

        // Record a large reward
        assert_ok!(Rewards::record_reward(&operator, 0, 10_000, &PricingModel::PayOnce { amount: 10_000 }));

        // Attempt to claim should fail
        assert_noop!(
            Rewards::claim_rewards(RuntimeOrigin::signed(operator)),
            pallet_rewards::Error::<Runtime>::TransferFailed
        );
    });
}

#[test]
fn test_max_pending_rewards_limit() {
    new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE]).execute_with(|| {
        let operator = mock_pub_key(BOB);

        // Record MaxPendingRewardsPerOperator rewards (100)
        for service_id in 0..100 {
            assert_ok!(Rewards::record_reward(
                &operator,
                service_id,
                100,
                &PricingModel::PayOnce { amount: 100 }
            ));
        }

        // 101st reward should fail
        assert_noop!(
            Rewards::record_reward(&operator, 100, 100, &PricingModel::PayOnce { amount: 100 }),
            pallet_rewards::Error::<Runtime>::TooManyPendingRewards
        );

        // Claim rewards
        assert_ok!(Rewards::claim_rewards(RuntimeOrigin::signed(operator.clone())));

        // Now can record new rewards
        assert_ok!(Rewards::record_reward(&operator, 100, 100, &PricingModel::PayOnce { amount: 100 }));
    });
}

#[test]
fn test_multiple_operators_concurrent_claims() {
    new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE]).execute_with(|| {
        let bob = mock_pub_key(BOB);
        let charlie = mock_pub_key(CHARLIE);
        let customer = mock_pub_key(ALICE);

        // Record rewards for both operators
        assert_ok!(Rewards::record_reward(&bob, 0, 5_000, &PricingModel::PayOnce { amount: 10_000 }));
        assert_ok!(Rewards::record_reward(&charlie, 0, 3_000, &PricingModel::PayOnce { amount: 10_000 }));

        // Both claim concurrently
        assert_ok!(Rewards::claim_rewards(RuntimeOrigin::signed(bob.clone())));
        assert_ok!(Rewards::claim_rewards(RuntimeOrigin::signed(charlie.clone())));

        // Verify balances updated correctly
        // ... assertions
    });
}
```

## Benefits of Integration

### Before (Mocked)
- ❌ Thread-local storage, not runtime storage
- ❌ Manual transfer simulation
- ❌ No real error conditions tested
- ❌ No bounded vec limit testing
- ❌ No event verification
- ❌ Can't test pallet account balance issues

### After (Real)
- ✅ Uses actual runtime storage
- ✅ Real `Currency::transfer()` logic
- ✅ Tests real error conditions
- ✅ Tests `MaxPendingRewardsPerOperator` limit
- ✅ Verifies actual events
- ✅ Tests pallet account insufficient balance
- ✅ Tests concurrent claims
- ✅ **90% realistic** (only EVM remains mocked)

## Testing Matrix

| Scenario | Current (Mocked) | With Real Rewards |
|----------|-----------------|-------------------|
| Basic claim | ⚠️ Simulated | ✅ Real extrinsic |
| Insufficient pallet balance | ❌ Can't test | ✅ Tests TransferFailed |
| Max pending rewards | ❌ Can't test | ✅ Tests TooManyPendingRewards |
| Concurrent claims | ⚠️ Simulated | ✅ Real storage contention |
| Event emission | ❌ No events | ✅ OperatorRewardsClaimed event |
| Multi-block claims | ⚠️ Simulated | ✅ Real storage persistence |
| Asset type rewards | ❌ Not implemented | ✅ Can extend for custom assets |

## Migration Checklist

- [ ] Add pallet-rewards parameter types to services/src/mock.rs
- [ ] Add pallet_rewards::Config impl to services/src/mock.rs
- [ ] Add Rewards to construct_runtime!
- [ ] Change Services::Config::RewardRecorder from MockRewardsManager to Rewards
- [ ] Remove RewardRecorder impl from MockRewardsManager (keep RewardsManager impl)
- [ ] Initialize rewards pallet account in new_test_ext_raw_authorities()
- [ ] Remove simulate_operator_claim() helper from operator_rewards_e2e.rs
- [ ] Update all tests to use Rewards::claim_rewards() extrinsic
- [ ] Add new test cases for error conditions
- [ ] Add tests for MaxPendingRewardsPerOperator limit
- [ ] Add tests for insufficient pallet balance
- [ ] Run full test suite: `cargo test --package pallet-services --lib`
- [ ] Verify all 94+ tests pass

## Expected Test Count After Integration

- Current: 94 tests (87 existing + 7 E2E)
- After: ~100 tests (94 existing + 6 new real-rewards tests)

## Notes

1. **Backwards Compatibility**: Old tests will continue to work, they'll just use real storage now
2. **Performance**: Minimal impact since we're already using other real pallets
3. **Debugging**: Can now inspect `PendingOperatorRewards` storage in tests
4. **Future**: Enables testing multi-asset rewards when implemented
