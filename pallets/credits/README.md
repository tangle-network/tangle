# Cloud Credits Pallet

## Overview

The Cloud Credits pallet provides an on-chain mechanism for tracking potential usage credits
earned through staking TNT or burning TNT. It is designed to work with an off-chain system
that listens to events to manage actual user credit balances.

It integrates with a staking system (like `pallet-multi-asset-delegation`) to reward
users who stake TNT tokens by tracking passively accrued potential credits within a defined time window.

### Key Features:

-   **Staking-Based Potential Credit Accrual:** Tracks potential credits earned based on
    TNT stake via `StakingInfo`. Accrual is **capped** to a configurable time window
    (`ClaimWindowBlocks`). Users do not accrue additional potential credits for periods
    longer than this window without claiming.
-   **TNT Burning Event:** Burning TNT emits an event (`CreditsGrantedFromBurn`) indicating
    potential credits granted for immediate off-chain use.
-   **Credit Claiming Event:** Users initiate a claim on-chain with an off-chain ID. The
    pallet calculates the potential credits accrued within the `ClaimWindowBlocks` ending
    at the current block. It verifies the requested amount against this calculated value and
    emits a `CreditsClaimed` event. **No on-chain balance is stored or deducted.**
-   **Window Cap (Implicit Decay):** Inactivity beyond the `ClaimWindowBlocks` simply results
    in no further potential credit accrual for that past period; there is no percentage decay
    applied to previously accrued potential amounts.

## Integration

This pallet relies on:

-   An implementation of `tangle_primitives::traits::MultiAssetDelegationInfo` (`Config::StakingInfo`)
    to query the active TNT stake for users.
-   An implementation of `frame_support::traits::tokens::fungibles::{Inspect, Mutate}`
    (`Config::Currency`) to handle TNT token balance checks and burning.
-   `frame_system` for basic system types and block numbers.
-   `sp_arithmetic::Perbill` for decay calculations.
-   **An external off-chain system** to listen for `CreditsGrantedFromBurn` and `CreditsClaimed`
    events and manage the actual credit balances associated with off-chain user accounts.

## Terminology

-   **TNT:** The utility token used for staking and burning.
-   **Potential Credits:** A value calculated on-chain based on staking or burning, used only for
    verification during claims and emitted in events. Not stored on-chain.
-   **Claim Window:** A configurable duration (`ClaimWindowBlocks`) representing the maximum
    period for which potential credits can be accrued before claiming.
-   **Claiming:** An on-chain action that calculates potential credits earned within the current
    claim window, verifies a requested amount, and emits an event for off-chain processing.
    This action also updates the `LastRewardUpdateBlock` marker.

## Interface

### Config Trait

-   `RuntimeEvent`: The overarching event type.
-   `Currency`: The fungibles token trait (`Inspect`, `Mutate`) for TNT.
-   `AssetId`: The Asset ID type.
-   `TntAssetId`: The specific Asset ID for TNT.
-   `StakingInfo`: Provides staking information.
-   `StakeTiers`: Defines potential credit emission rates.
-   `BurnConversionRate`: Rate for converting burned TNT to potential credits.
-   `ClaimWindowBlocks`: The maximum accrual window duration in blocks.
-   `CreditBurnTarget`: Optional account for burned TNT.
-   `MaxOffchainAccountIdLength`: Max length for the off-chain ID during claim.
-   `MaxTiers`: Constant for BoundedVec limit.

### Dispatchable Functions (Extrinsics)

-   `burn(origin, amount)`: Burns TNT, calculates potential credits, emits `CreditsGrantedFromBurn`.
-   `claim_credits(origin, amount_to_claim, offchain_account_id)`: Calculates potential
    credits earned within the claim window, verifies `amount_to_claim` against
    this value, emits `CreditsClaimed` event.

### Storage Items

-   `LastRewardUpdateBlock`: Tracks the last block number potential staking rewards were accounted for.
    Effectively marks the start for the _next_ potential accrual window upon claim/burn.

### Events

-   `CreditsGrantedFromBurn`: Emitted when TNT is burned.
-   `CreditsClaimed`: Emitted when a user successfully validates a claim amount against
    their calculated potential credits within the window.

### Errors

-   `InsufficientTntBalance`: Not enough TNT to burn.
-   `ClaimAmountExceedsWindowAllowance`: Requested claim amount exceeds calculated potential credits
    within the allowed window.
-   `OffchainAccountIdTooLong`: Provided off-chain ID during claim exceeds the maximum length.
-   `Overflow`: Arithmetic overflow occurred.
-   `NoStakeTiersConfigured`: Runtime configuration for `StakeTiers` is missing/empty.
-   `AmountZero`: Trying to burn or claim zero amount.
-   `BurnTransferNotImplemented`: `CreditBurnTarget` is configured, but transfer logic isn't enabled.
