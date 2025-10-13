# Multi-Asset USD-Weighted Reward Distribution Design

## Current Status

### What's Implemented (Phase 1)
- ✅ Basic exposure-weighted distribution using `exposure_percent`
- ✅ Fixed reward shares: 85% operators / 10% developer / 5% protocol
- ✅ Distributes based on percentage commitments only
- ✅ Works for job-level payments (PayOnce, Subscription, EventDriven)

### What's Missing (Phase 2 - Current Task)
- ❌ Multi-asset value consideration (TNT, WETH, USDC, etc.)
- ❌ USD denomination using oracle price feeds
- ❌ Actual restaked amount weighting (not just percentages)
- ❌ QoS metrics integration
- ❌ Service-level payment distribution (currently goes to MBSM)

## Problem Statement

**Current Limitation:**
An operator committing 50% exposure with $1M in restaked assets gets the same reward as an operator committing 50% exposure with $10K in restaked assets.

**Desired Behavior:**
Rewards should be proportional to the USD value of assets at risk, not just the percentage commitment.

## Available Data Sources

### 1. MultiAssetDelegationInfo Trait
```rust
fn get_total_delegation_by_asset(operator: &AccountId, asset: &Asset<AssetId>) -> Balance;
```
- **Location**: `primitives/src/traits/multi_asset_delegation.rs:85`
- **Returns**: Total delegated amount for specific operator + asset pair
- **Assets Available**: Native (TNT), WETH, USDC, WBTC, etc.

### 2. Oracle Price Feeds
```rust
impl<T: Config> DataProvider<OracleKey, OracleValue> for pallet_oracle::Pallet<T> {
    fn get(key: &OracleKey) -> Option<OracleValue>;
}
```
- **Location**: `pallets/oracle/src/lib.rs:303-306`
- **Returns**: `TimestampedValue { value, timestamp }`
- **Usage**: Get USD price for any asset (e.g., "TNT/USD", "WETH/USD")

### 3. Operator Security Commitments
```rust
pub struct AssetSecurityCommitment<AssetId> {
    pub asset: Asset<AssetId>,
    pub exposure_percent: Percent,  // E.g., 50% for this asset
}
```
- **Location**: Service `operator_security_commitments` field
- **Contains**: Which assets each operator committed and at what exposure percentage

### 4. QoS Metrics (Future Integration)
- **Location**: `pallets/services/src/functions/qos.rs`
- **Available Metrics**:
  - Heartbeat status
  - Uptime percentage
  - Slashing history
  - **Not yet integrated into rewards**

## Proposed Algorithm: USD-Weighted Distribution

### Step 1: Calculate Each Operator's USD Value at Risk

For each operator `i`:

```
USD_value[i] = Σ (for each committed asset a) {
    delegated_amount = get_total_delegation_by_asset(operator[i], asset[a])
    exposure_percent = commitment[i][a].exposure_percent
    usd_price = oracle.get("asset[a]/USD")

    // Calculate USD value considering exposure commitment
    (delegated_amount * exposure_percent / 100) * usd_price
}
```

**Example:**
- Operator Bob commits: 50% TNT, 50% WETH
- Bob's delegations: 10,000 TNT, 5 WETH
- Oracle prices: TNT/USD = $0.50, WETH/USD = $3,000
- Bob's USD at risk:
  - TNT: (10,000 * 0.50) * $0.50 = $2,500
  - WETH: (5 * 0.50) * $3,000 = $7,500
  - **Total: $10,000**

### Step 2: Calculate Proportional Share

```
total_usd_at_risk = Σ USD_value[all operators]

operator[i]_share = (USD_value[i] / total_usd_at_risk) * operator_total_rewards
```

**Example:**
- Bob: $10,000 at risk
- Alice: $5,000 at risk
- Charlie: $5,000 at risk
- Total: $20,000
- Operator pool: 8,500 tokens (85% of 10,000 payment)

Rewards:
- Bob: (10,000 / 20,000) * 8,500 = 4,250 tokens
- Alice: (5,000 / 20,000) * 8,500 = 2,125 tokens
- Charlie: (5,000 / 20,000) * 8,500 = 2,125 tokens

### Step 3: Optional QoS Multiplier (Future)

```
qos_score[i] = calculate_qos_score(operator[i])  // 0.5 to 1.5 range
adjusted_share[i] = operator[i]_share * qos_score[i]

// Renormalize to ensure total = operator_total_rewards
final_share[i] = adjusted_share[i] * (operator_total_rewards / Σ adjusted_share)
```

## Implementation Plan

### Phase 2A: Add Oracle Integration (Current Priority)

1. **Add Oracle Config to Services Pallet**
```rust
pub trait Config: frame_system::Config {
    // ... existing config

    /// Oracle pallet for USD price feeds
    type Oracle: DataProvider<AssetId, Balance>;

    /// Asset ID representing USD (for oracle keys)
    type UsdAssetId: Get<Self::AssetId>;
}
```

2. **Create Multi-Asset Value Calculator**
```rust
// In reward_distribution.rs
impl<T: Config> Pallet<T> {
    /// Calculate USD value of operator's committed assets
    fn calculate_operator_usd_value(
        operator: &T::AccountId,
        commitments: &[AssetSecurityCommitment<T::AssetId>],
    ) -> Result<BalanceOf<T>, DispatchError> {
        let mut total_usd_value = BalanceOf::<T>::zero();

        for commitment in commitments {
            // Get delegated amount for this asset
            let delegated = T::OperatorDelegationManager::get_total_delegation_by_asset(
                operator,
                &commitment.asset
            );

            // Apply exposure percentage
            let exposed_amount = commitment.exposure_percent.mul_floor(delegated);

            // Get USD price from oracle
            let usd_price = Self::get_asset_usd_price(&commitment.asset)?;

            // Calculate USD value
            let usd_value = exposed_amount
                .checked_mul(&usd_price)
                .ok_or(Error::<T>::ArithmeticOverflow)?;

            total_usd_value = total_usd_value
                .checked_add(&usd_value)
                .ok_or(Error::<T>::ArithmeticOverflow)?;
        }

        Ok(total_usd_value)
    }

    fn get_asset_usd_price(asset: &Asset<T::AssetId>) -> Result<BalanceOf<T>, DispatchError> {
        // Query oracle for "asset/USD" price
        // Handle staleness, missing prices, etc.
    }
}
```

3. **Update `distribute_to_operators` Function**
Replace the current exposure-percent-based calculation with USD-value-based calculation.

### Phase 2B: Add Fallback Strategy

When oracle prices are unavailable or stale:
1. **Fallback to exposure-only weighting** (current implementation)
2. **Log warning event** for monitoring
3. **Continue operation** (don't block payments)

```rust
fn distribute_to_operators_with_fallback(...) -> DispatchResult {
    match Self::try_usd_weighted_distribution(...) {
        Ok(()) => Ok(()),
        Err(OracleError) => {
            log::warn!("Oracle unavailable, using exposure-only distribution");
            Self::distribute_by_exposure_only(...)  // Current implementation
        }
    }
}
```

### Phase 2C: QoS Integration (Future)

Once QoS metrics are finalized:
```rust
fn calculate_qos_multiplier(operator: &T::AccountId) -> Perbill {
    // Factors:
    // - Heartbeat uptime: 0.8 - 1.0
    // - Slash history: 0.5 - 1.0
    // - Job completion rate: 0.9 - 1.1
    // Combined: 0.5 - 1.5 range
}
```

## Configuration & Governance

### Oracle Key Format
Standardize oracle keys for asset prices:
- Native: `"TNT/USD"`
- ERC20: `"WETH/USD"`, `"USDC/USD"`
- Custom assets: `"ASSET_{id}/USD"`

### Staleness Thresholds
```rust
parameter_types! {
    pub const MaxPriceStaleness: BlockNumber = 100;  // ~10 minutes
}
```

### Enable/Disable USD Weighting
Add runtime configuration:
```rust
pub enum RewardDistributionMode {
    ExposureOnly,      // Phase 1 (current)
    UsdWeighted,       // Phase 2A
    UsdWithQoS,        // Phase 2C
}
```

## Testing Strategy

### Unit Tests
- ✅ Test exposure-only distribution (existing)
- ⏳ Test USD-weighted with mock oracle
- ⏳ Test fallback when oracle fails
- ⏳ Test price staleness handling
- ⏳ Test zero/negative price handling

### Integration Tests
- ⏳ Multi-operator with different asset mixes
- ⏳ Oracle price updates mid-service
- ⏳ Compare exposure-only vs USD-weighted outcomes
- ⏳ Edge cases: extreme price ratios, dust amounts

### Scenario Tests
| Scenario | Bob (TNT+WETH) | Alice (USDC) | Expected |
|----------|----------------|--------------|----------|
| Equal USD value | $10K | $10K | 50/50 split |
| 2:1 USD ratio | $10K | $5K | 66/33 split |
| TNT price doubles | $20K | $5K | 80/20 split |

## Migration Path

### Backward Compatibility
- Phase 1 (exposure-only) remains as fallback
- Existing services continue working
- New services can opt into USD weighting

### Upgrade Path
1. Deploy oracle pallet if not present
2. Feed initial asset prices
3. Deploy updated services pallet
4. Enable USD weighting via governance
5. Monitor rewards distribution

## Open Questions

1. **Oracle Reliability**: What if oracle is manipulated or goes offline?
   - **Answer**: Use median of multiple operators, with staleness checks and fallback

2. **Asset Decimals**: How to handle different decimal places (USDC=6, WETH=18)?
   - **Answer**: Normalize to common denomination before USD conversion

3. **Service-Level Payments**: Should upfront payments also use this distribution?
   - **Answer**: Yes, but requires MBSM integration changes (future work)

4. **Gas Costs**: USD calculation adds oracle reads - is it worth it?
   - **Answer**: Profile and optimize; consider caching prices per block

## Success Criteria

✅ **Phase 1 Complete**: Exposure-weighted distribution working
⏳ **Phase 2A Complete**: USD-weighted distribution with oracle integration
⏳ **Phase 2B Complete**: Robust fallback and error handling
⏳ **Phase 2C Complete**: QoS metrics integrated

### Metrics
- Reward distribution reflects actual economic risk
- Oracle integration is reliable with <1% downtime fallback usage
- Gas costs increase by <20% compared to Phase 1
- Zero reward calculation errors in production

## Next Steps

1. ✅ Document current implementation
2. ⏳ Get stakeholder approval for USD weighting approach
3. ⏳ Implement oracle integration in services pallet
4. ⏳ Add multi-asset USD value calculator
5. ⏳ Update distribution logic with fallback
6. ⏳ Write comprehensive tests
7. ⏳ Deploy to testnet and monitor
8. ⏳ Production deployment with feature flag

---

**Status**: Phase 1 Complete, Phase 2A In Progress
**Last Updated**: 2025-10-12
**Author**: Claude Code (Audit & Implementation)
