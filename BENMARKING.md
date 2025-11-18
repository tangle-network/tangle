# Benchmarking Criteria and Best Practices

This document outlines the criteria and best practices for writing Substrate benchmarks in the Tangle codebase.

## Core Principles

### 1. Worst-Case Scenarios
**Always benchmark worst-case scenarios to ensure accurate weight calculations.**

- **Amounts**: Use maximum allowed values or values that exercise the most expensive path
  - Example: `T::Currency::minimum_balance() * 1000u32.into()` for large amounts
  - Example: Use maximum tier thresholds for stake-based calculations
  - Example: Use `T::MaxStakeTiers::get()` for tier configurations

- **BoundedVec/Strings**: Use maximum allowed lengths
  - Example: `vec![b'A'; T::MaxVaultNameLength::get() as usize]` for names
  - Example: `vec![b'B'; T::MaxVaultLogoLength::get() as usize]` for logos
  - Example: Use `T::MaxOffchainAccountIdLength` for claim IDs

- **Collections**: Use maximum allowed sizes
  - Example: `T::MaxStakeTiers::get()` for tier arrays
  - Example: `T::MaxDelegatorBlueprints::get()` for blueprint selections
  - Example: Maximum number of delegations, operators, etc.

- **Block Numbers**: Use full claim windows or delay periods
  - Example: `T::ClaimWindowBlocks::get()` for credit accrual windows
  - Example: `T::LeaveDelegatorsDelay::get()` for withdrawal delays
  - Example: `T::DelegationBondLessDelay::get()` for delegation delays

### 2. Account Funding
**Always ensure accounts have sufficient balance before operations.**

- **Fund all accounts** involved in transactions (caller, operator, pallet account, etc.)
- Use helper functions for consistent funding:
  ```rust
  fn fund_account<T: Config>(who: &T::AccountId) {
      let balance = T::Currency::minimum_balance() * INITIAL_BALANCE.into();
      T::Currency::make_free_balance_be(who, balance);
  }
  ```
- **Fund pallet accounts** when they receive transfers:
  ```rust
  let pallet_account_id = Pallet::<T>::pallet_account();
  fund_account::<T>(&pallet_account_id);
  ```
- **Fund EVM-mapped accounts** when using EVM addresses:
  ```rust
  let evm_account: T::AccountId = T::EvmAddressMapping::into_account_id(evm_address);
  fund_account::<T>(&evm_account);
  ```

### 3. Origin Handling
**Match the expected origin type for each extrinsic.**

- **Signed origins**: Use `RawOrigin::Signed(caller.clone())` for user actions
- **Root origins**: Use `RawOrigin::Root` for admin functions
- **Pallet origins**: Use `Pallet::<T>::pallet_account()` for pallet-originated calls
  - Example: `execute_withdraw` with EVM address must use pallet account origin
- **EnsureOrigin**: Use `T::ForceOrigin::try_successful_origin()` for custom origins
  ```rust
  let origin = T::VaultMetadataOrigin::try_successful_origin()
      .map_err(|_| BenchmarkError::Weightless)?;
  ```

### 4. Storage Setup
**Set up all required storage state before executing benchmarks.**

- **Configure tiers** (global vs asset-specific):
  - `claim_credits` uses `get_current_rate` → reads from `StoredStakeTiers` (global tiers)
  - `claim_credits_with_asset` uses `get_current_rate_for_asset` → reads from `AssetStakeTiers` (asset-specific)
  - Always set up the correct tier type based on the function being benchmarked

- **Set up delegations**:
  ```rust
  setup_delegation::<T>(&account, stake_amount, asset).unwrap();
  ```

- **Set up operators**:
  ```rust
  MultiAssetDelegation::<T>::join_operators(
      RawOrigin::Signed(operator.clone()).into(),
      bond_amount
  )?;
  ```

- **Set up staking ledger** (for nomination benchmarks):
  ```rust
  assert_ok!(T::StakingInterface::bond(who, nomination_amount, who));
  assert_ok!(T::StakingInterface::nominate(who, vec![operator.clone()]));
  ```

- **Set up reward pools**:
  ```rust
  OperatorRewardPools::<T>::insert(&operator, pool);
  DelegatorRewardDebts::<T>::insert(&delegator, &operator, debt);
  ```

- **Set up metadata**:
  ```rust
  VaultMetadataStore::<T>::insert(vault_id, metadata);
  ```

### 5. Delay Handling
**Correctly advance block numbers for time-dependent operations.**

- **Execute withdrawals**: Use `LeaveDelegatorsDelay`
  ```rust
  let current_round = Pallet::<T>::current_round();
  CurrentRound::<T>::put(current_round + T::LeaveDelegatorsDelay::get());
  ```

- **Execute operator unstake**: Use `LeaveOperatorsDelay`
  ```rust
  CurrentRound::<T>::put(current_round + T::LeaveOperatorsDelay::get());
  ```

- **Credit accrual**: Advance by claim window
  ```rust
  let window = T::ClaimWindowBlocks::get();
  let end_block = start_block.saturating_add(window);
  frame_system::Pallet::<T>::set_block_number(end_block);
  ```

### 6. Verification Blocks
**Always verify the benchmark executed correctly.**

- **Check storage updates**:
  ```rust
  verify {
      let delegator = Delegators::<T>::get(&caller).unwrap();
      assert_eq!(delegator.deposits.get(&asset).unwrap().amount, amount);
  }
  ```

- **Check state changes**:
  ```rust
  verify {
      assert!(Operators::<T>::contains_key(&caller));
  }
  ```

- **Check removals**:
  ```rust
  verify {
      assert!(!delegator.withdraw_requests.iter().any(|r| r.asset == asset && r.amount == amount));
  }
  ```

- **Use specific assertions** to avoid false positives from previous benchmark runs:
  ```rust
  // Good: Specific check
  assert!(!delegator.withdraw_requests.iter().any(|r| r.asset == asset && r.amount == amount));
  
  // Bad: Too broad, may fail if other requests exist
  assert!(delegator.withdraw_requests.is_empty());
  ```

### 7. Asset Handling
**Properly handle asset types and EVM addresses.**

- **Native assets**: Use `Asset::Custom(0_u32.into())` or `Asset::Custom(native_asset_id::<T>())`
- **EVM addresses**: 
  - When `Asset::Custom` is used with `Some(evm_address)`, the caller must be the mapped EVM account
  - When `Asset::Custom` is used with `None`, use regular account
  ```rust
  // For Asset::Custom with no EVM address
  let evm_address = None;
  
  // For Asset::Custom with EVM address
  let evm_address = Some(H160::repeat_byte(1));
  let evm_account: T::AccountId = T::EvmAddressMapping::into_account_id(evm_address.unwrap());
  fund_account::<T>(&evm_account);
  ```

### 8. Error Handling
**Handle errors gracefully and ensure setup succeeds.**

- **Use `Result` return types** for benchmarks that may fail
- **Unwrap setup operations** only when you're certain they'll succeed:
  ```rust
  setup_delegation::<T>(&account, max_stake_amount, asset).unwrap();
  ```
- **Map errors** appropriately:
  ```rust
  .map_err(|_| BenchmarkError::Weightless)?;
  ```
- **Assert critical conditions**:
  ```rust
  assert!(!max_claimable.is_zero(), "Setup must result in non-zero credits");
  ```

### 9. Helper Functions
**Create reusable helper functions for common setup patterns.**

- **Account setup**:
  ```rust
  fn setup_benchmark<T: Config>() -> Result<T::AccountId, &'static str> {
      let caller: T::AccountId = whitelisted_caller();
      fund_account::<T>(&caller);
      Ok(caller)
  }
  ```

- **Delegation setup**:
  ```rust
  fn setup_delegation<T: Config>(
      delegator: &T::AccountId,
      stake_amount: BalanceOf<T>,
      asset_id: Asset<T::AssetId>,
  ) -> Result<(), &'static str> {
      // ... setup logic
  }
  ```

- **Nominator setup** (for staking):
  ```rust
  fn setup_nominator<T: Config>(
      who: &T::AccountId,
      operator: &T::AccountId,
      asset_id: Asset<T::AssetId>,
      stake_amount: BalanceOf<T>,
      delegation_amount: BalanceOf<T>,
      nomination_amount: BalanceOf<T>,
  ) -> Result<(), &'static str> {
      // ... setup logic including staking ledger
  }
  ```

### 10. Data Validation
**Validate that setup produces expected results before benchmarking.**

- **Check non-zero amounts**:
  ```rust
  let max_claimable = Credits::<T>::get_accrued_amount(&account, Some(end_block))
      .map_err(|_| BenchmarkError::Weightless)?;
  assert!(!max_claimable.is_zero(), "Setup must result in non-zero credits");
  ```

- **Verify storage state** before operations:
  ```rust
  // Verify withdraw request exists before execution
  let metadata = Delegators::<T>::get(&evm_account).unwrap();
  assert!(
      metadata.withdraw_requests.iter().any(|r| r.asset == asset && r.amount == amount),
      "Withdraw request must exist before execution"
  );
  ```

### 11. Comments and Documentation
**Document complex setup logic and explain why choices were made.**

- **Explain worst-case choices**:
  ```rust
  // Setup: Use maximum stake tier threshold for worst case scenario
  let stored_tiers = Credits::<T>::stake_tiers();
  let max_stake_amount = stored_tiers.iter().map(|t| t.threshold).max().unwrap_or(10_000u32.into());
  ```

- **Explain tier selection**:
  ```rust
  // Setup global stake tiers for the benchmark with maximum rate
  // claim_credits uses get_current_rate which reads from StoredStakeTiers (global tiers)
  ```

- **Explain origin choices**:
  ```rust
  // Execute withdraw uses LeaveDelegatorsDelay for readiness check
  ```

- **Explain delay choices**:
  ```rust
  // Advance blocks by the full claim window for worst case scenario
  ```

## Common Pitfalls and Solutions

### 1. InsufficientBalance
**Problem**: Account doesn't have enough balance for the operation.

**Solution**: Always fund accounts using `fund_account::<T>(&account)` before operations.

### 2. Bad Origin
**Problem**: Wrong origin type passed to extrinsic.

**Solution**: 
- Check the pallet's origin requirements (`ensure_signed`, `ensure_pallet`, etc.)
- Use `Pallet::<T>::pallet_account()` for pallet-originated calls
- Use `RawOrigin::Root` for admin functions

### 3. Zero Rate/Credits
**Problem**: Rate calculation returns zero because tiers aren't configured.

**Solution**:
- Understand which tier storage is used (`StoredStakeTiers` vs `AssetStakeTiers`)
- Set up the correct tier type before calculating rates
- Verify rates are non-zero before proceeding

### 4. NotNominator
**Problem**: Staking interface can't find nominator data.

**Solution**: Set up staking ledger directly via storage manipulation:
```rust
assert_ok!(T::StakingInterface::bond(who, nomination_amount, who));
assert_ok!(T::StakingInterface::nominate(who, vec![operator.clone()]));
```

### 5. Funds Unavailable
**Problem**: Withdrawal can't be executed because funds aren't available.

**Solution**:
- Ensure deposits are made before withdrawals
- Advance rounds correctly using the right delay (`LeaveDelegatorsDelay` vs `DelegationBondLessDelay`)
- Fund pallet account if it receives transfers

### 6. Verification Failures
**Problem**: Assertions fail even though the operation succeeded.

**Solution**:
- Use specific assertions that check for exact values rather than broad checks
- Check for specific `(asset, amount)` pairs rather than checking if collections are empty
- Verify state after the operation, not before

## Checklist for New Benchmarks

- [ ] Use worst-case amounts (maximum allowed values)
- [ ] Use worst-case data sizes (maximum lengths for strings/BoundedVecs)
- [ ] Fund all accounts involved in the transaction
- [ ] Set up all required storage state (tiers, delegations, operators, etc.)
- [ ] Use correct origin type for the extrinsic
- [ ] Advance block numbers correctly for time-dependent operations
- [ ] Handle asset types and EVM addresses correctly
- [ ] Add verification blocks to check the operation succeeded
- [ ] Validate setup produces expected results (non-zero amounts, etc.)
- [ ] Add comments explaining complex setup logic
- [ ] Test the benchmark runs successfully before committing

## Testing Benchmarks

Run benchmarks with:
```bash
# Test specific pallet benchmarks
cargo test --features runtime-benchmarks -p pallet-name --lib benchmarking

# Generate weights
bash scripts/generate-weights.sh [testnet|mainnet]
```

## References

- [Substrate Benchmarking Documentation](https://docs.substrate.io/reference/how-to-guides/weights/add-benchmarks/)
- Framework benchmarking examples in `pallets/*/src/benchmarking.rs`
- Test files for understanding expected behavior: `pallets/*/src/tests*.rs`

