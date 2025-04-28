# Cloud Credits Pallet

## Overview

The Cloud Credits pallet provides an on-chain mechanism for users to acquire and manage
usage credits, primarily intended for accessing off-chain services like AI assistants.
It integrates with a staking system (like `pallet-multi-asset-delegation`) to reward
users who stake TNT tokens with passively accrued credits.

### Key Features:

-   **Staking-Based Credit Accrual:** Users automatically earn credits based on the amount
    of TNT they have staked via a configured `StakingInfo` provider. Credit emission rates
    are tiered based on stake size.
-   **TNT Burning:** Users can burn TNT tokens for an immediate, one-time grant of credits.
-   **Account Linking:** Users link their on-chain account (`AccountId`) to an off-chain
    identifier (e.g., GitHub handle, email hash) to facilitate off-chain credit redemption.
-   **Credit Claiming:** Users initiate a claim on-chain, signaling the intention to use
    credits off-chain. This reduces the on-chain balance.
-   **Activity-Based Decay:** To strongly incentivize weekly interaction, the
    _claimable_ portion of a user's credit balance decays significantly if they do not
    claim or actively update their credits frequently (ideally weekly). The raw accrued balance
    remains, but its effective claimable value diminishes rapidly with prolonged inactivity.
-   **Admin Controls:** Provides administrative functions to manage credit balances and account links.

## Integration

This pallet relies on:

-   An implementation of `tangle_primitives::traits::MultiAssetDelegationInfo` (`Config::StakingInfo`)
    to query the active TNT stake for users.
-   An implementation of `frame_support::traits::tokens::fungibles` (`Config::Currency`) to handle
    TNT token balances, transfers (for burning to a target address), and burning.
-   `frame_system` for basic system types and block numbers.
-   `sp_arithmetic::Perbill` for decay calculations.

## Terminology

-   **TNT:** The primary utility token used for staking and burning.
-   **Credits:** An on-chain numerical balance representing usage rights for off-chain services.
    Credits are not transferable tokens themselves.
-   **Staking:** Locking TNT tokens via the `StakingInfo` provider (e.g., `pallet-multi-asset-delegation`).
-   **Burning:** Permanently destroying TNT tokens in exchange for immediate credits.
-   **Linking:** Associating an on-chain `AccountId` with an off-chain identifier.
-   **Claiming:** Reducing the on-chain credit balance, implying off-chain usage.
-   **Interaction:** An action (linking, claiming, triggering update) that resets the decay timer.
-   **Decay:** Reduction in the _claimable percentage_ of the raw credit balance over time due to inactivity. Designed to be aggressive after a grace period (e.g., 1 week) to encourage regular claims.

## Interface

### Config Trait

-   `RuntimeEvent`: The overarching event type.
-   `Currency`: The fungibles token trait for managing the TNT token.
-   `AssetId`: The Asset ID type used by `Currency` and `StakingInfo`.
-   `TntAssetId`: The specific Asset ID for the TNT token (constant).
-   `StakingInfo`: Provides staking information via `MultiAssetDelegationInfo`.
-   `StakeTiers`: Defines credit emission rates based on stake amount (constant).
-   `BurnConversionRate`: Rate for converting burned TNT to credits (constant).
-   `DecaySteps`: Defines the aggressive decay curve based on inactivity, aiming to incentivize weekly interaction (constant).
-   `CreditBurnTarget`: Optional account to send burned TNT to (constant).
-   `AdminOrigin`: Origin authorized for administrative actions.
-   `MaxOffchainAccountIdLength`: Maximum length for the linked off-chain ID (constant).
-   `PalletId`: PalletId for deriving sovereign accounts (constant).

### Dispatchable Functions (Extrinsics)

-   `link_account(origin, offchain_account_id)`: Links the sender's account to an off-chain ID and resets the decay timer.
-   `burn(origin, amount)`: Burns TNT for immediate credits. Accrues staking rewards first. Does _not_ reset the decay timer.
-   `trigger_credit_update(origin)`: Updates accrued staking rewards and resets the decay timer.
-   `claim_credits(origin, amount_to_claim, target_offchain_account_id)`: Claims credits, applying decay based on inactivity, and resets the decay timer.
-   `force_set_credit_balance(origin, who, new_balance)`: (Admin) Sets an account's credit balance and resets the decay timer.
-   `force_link_account(origin, who, offchain_account_id)`: (Admin) Links an account to an off-chain ID and resets the decay timer.

### Storage Items

-   `CreditBalances`: Stores the raw (undecayed) credit balance per account.
-   `LinkedAccounts`: Stores the mapping from on-chain `AccountId` to the off-chain ID.
-   `LastRewardUpdateBlock`: Tracks the last block number rewards were calculated for an account.
-   `LastInteractionBlock`: Tracks the last block number an account interacted (linked, claimed, updated), used for decay calculation.

### Events

-   `AccountLinked`: Emitted when an account is linked.
-   `CreditsBurned`: Emitted when TNT is burned for credits.
-   `CreditsClaimed`: Emitted when credits are claimed (after decay application).
-   `CreditsAccrued`: Emitted when staking rewards are added to the raw balance.
-   `AdminCreditBalanceSet`: Emitted when an admin sets a balance.
-   `AdminAccountLinked`: Emitted when an admin links an account.

### Errors

-   `InsufficientTntBalance`: Not enough TNT to burn.
-   `InsufficientCreditBalance`: Not enough _claimable_ credits after decay.
-   `AccountNotLinked`: Attempting to claim without linking first.
-   `OffchainAccountMismatch`: Provided off-chain ID doesn't match the linked one during claim.
-   `OffchainAccountIdTooLong`: Provided off-chain ID exceeds the maximum length.
-   `AlreadyLinked`: Trying to link an account that is already linked.
-   `Overflow`: Arithmetic overflow occurred.
-   `NotStaking`: (Implicit) User has no stake according to `StakingInfo`.
-   `NoStakeTiersConfigured`: Runtime configuration for `StakeTiers` is missing/empty.
-   `AmountZero`: Trying to burn or claim zero amount.
