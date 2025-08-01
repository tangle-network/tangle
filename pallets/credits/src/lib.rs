//! # Cloud Credits Pallet
//!
//! ## Overview
//!
//! The Cloud Credits pallet provides an on-chain mechanism for tracking potential usage credits
//! earned through staking TNT or burning TNT. It is designed to work with an off-chain system
//! that listens to events to manage actual user credit balances.
//!
//! It integrates with a staking system (like `pallet-multi-asset-delegation`) to reward
//! users who stake TNT tokens by tracking passively accrued potential credits within a defined time
//! window.
//!
//! ### Key Features:
//!
//! - **Staking-Based Potential Credit Accrual:** Tracks potential credits earned based on TNT stake
//!   via `MultiAssetDelegationInfo`. Accrual is **capped** to a configurable time window
//!   (`ClaimWindowBlocks`). Users do not accrue additional potential credits for periods longer
//!   than this window without claiming.
//! - **Stake Tier Configuration:** Credit emission rates based on stake size are defined via
//!   `StakeTier` structs, which are configured during genesis and stored on-chain.
//! - **TNT Burning Event:** Burning TNT emits an event (`CreditsGrantedFromBurn`) indicating
//!   potential credits granted for immediate off-chain use.
//! - **Credit Claiming Event:** Users initiate a claim on-chain with an off-chain ID. The pallet
//!   calculates the potential credits accrued within the `ClaimWindowBlocks` ending at the current
//!   block. It verifies the requested amount against this calculated value and emits a
//!   `CreditsClaimed` event. **No on-chain balance is stored or deducted.**
//! - **Window Cap:** Inactivity beyond the `ClaimWindowBlocks` simply results in no further
//!   potential credit accrual for that past period.
//!
//! ## Integration
//!
//! This pallet relies on:
//!
//! - An implementation of `tangle_primitives::traits::MultiAssetDelegationInfo`
//!   (`Config::MultiAssetDelegationInfo`) to query the active TNT stake for users.
//! - An implementation of `frame_support::traits::Currency` (`Config::Currency`) to handle TNT
//!   token balance checks and burning.
//! - `frame_system` for basic system types and block numbers.
//! - **An external off-chain system** to listen for `CreditsGrantedFromBurn` and `CreditsClaimed`
//!   events and manage the actual credit balances associated with off-chain user accounts.
//!
//! ## Terminology
//!
//! - **TNT:** The utility token used for staking and burning.
//! - **Potential Credits:** A value calculated on-chain based on staking or burning, used only for
//!   verification during claims and emitted in events. Not stored on-chain.
//! - **Claim Window:** A configurable duration (`ClaimWindowBlocks`) representing the maximum
//!   period for which potential credits can be accrued before claiming.
//! - **Claiming:** An on-chain action that calculates potential credits earned within the current
//!   claim window, verifies a requested amount, and emits an event for off-chain processing. This
//!   action also updates the `LastRewardUpdateBlock` marker.
//! - **Stake Tier:** A configuration struct defining a TNT stake threshold and the corresponding
//!   potential credit emission rate per block.

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::traits::Currency;
pub use pallet::*;

pub mod types;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

mod weights;
pub use weights::WeightInfo;

pub type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

#[frame_support::pallet(dev_mode)]
pub mod pallet {
	use super::WeightInfo;
	use crate::{types::*, BalanceOf};
	use core::cmp::max;
	use frame_support::{
		pallet_prelude::*,
		traits::{Currency, ExistenceRequirement, LockableCurrency, ReservableCurrency},
	};
	use frame_system::pallet_prelude::*;
	use scale_info::prelude::vec::Vec;
	use sp_runtime::traits::{CheckedMul, MaybeDisplay, SaturatedConversion, Saturating, Zero};
	use sp_std::fmt::Debug;
	use tangle_primitives::{rewards::AssetType, traits::MultiAssetDelegationInfo};

	// Move STORAGE_VERSION inside the pallet mod
	const STORAGE_VERSION: StorageVersion = StorageVersion::new(2);

	#[pallet::pallet]
	#[pallet::without_storage_info]
	#[pallet::storage_version(STORAGE_VERSION)]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The overarching event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// The fungibles token trait for managing the TNT token.
		/// Ensure the implementation's Balance type satisfies necessary bounds (like Zero,
		/// From<BlockNumber> etc.)
		type Currency: Currency<Self::AccountId>
			+ ReservableCurrency<Self::AccountId>
			+ LockableCurrency<Self::AccountId>;

		/// The Asset ID type used by the Currency trait and MultiAssetDelegationInfo.
		type AssetId: Parameter
			+ Member
			+ MaybeDisplay
			+ Ord
			+ MaxEncodedLen
			+ Copy
			+ Debug
			+ Default;

		/// The provider for checking the active TNT stake.
		/// Ensure BalanceOf<Self> here resolves correctly to T::Currency::Balance.
		type MultiAssetDelegationInfo: MultiAssetDelegationInfo<
			Self::AccountId,
			BalanceOf<Self>,
			BlockNumberOf<Self>,
			Self::AssetId,
			AssetType<Self::AssetId>,
		>;

		/// The conversion rate for burning TNT to credits.
		#[pallet::constant]
		type BurnConversionRate: Get<BalanceOf<Self>>;

		/// The maximum window (in blocks) for which credits can be accrued before claiming.
		#[pallet::constant]
		type ClaimWindowBlocks: Get<BlockNumberOf<Self>>;

		/// Optional: An account to send burned TNT to. If None, `Currency::burn_from` is used.
		#[pallet::constant]
		type CreditBurnRecipient: Get<Option<Self::AccountId>>;

		/// The maximum length allowed for an off-chain account ID string.
		#[pallet::constant]
		type MaxOffchainAccountIdLength: Get<u32>;

		/// The maximum number of stake tiers.
		#[pallet::constant]
		type MaxStakeTiers: Get<u32>;

		/// Type for the origin that is allowed to update stake tiers.
		type ForceOrigin: frame_support::traits::EnsureOrigin<Self::RuntimeOrigin>;

		/// The maximum rate per block for a stake tier.
		#[pallet::constant]
		type MaxRatePerBlock: Get<BalanceOf<Self>>;

		/// The weight information for the pallet.
		type WeightInfo: WeightInfo;
	}

	// --- Storage Items ---

	#[pallet::storage]
	#[pallet::getter(fn last_reward_update_block)]
	pub type LastRewardUpdateBlock<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, BlockNumberOf<T>, ValueQuery>;

	/// Storage for the configured staking tiers.
	#[pallet::storage]
	#[pallet::getter(fn stake_tiers)]
	pub type StoredStakeTiers<T: Config> =
		StorageValue<_, BoundedVec<StakeTier<BalanceOf<T>>, T::MaxStakeTiers>, ValueQuery>;

	/// Storage for asset-specific staking tiers.
	/// Each asset can have its own set of stake tiers and rates.
	#[pallet::storage]
	#[pallet::getter(fn asset_stake_tiers)]
	pub type AssetStakeTiers<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::AssetId,
		BoundedVec<StakeTier<BalanceOf<T>>, T::MaxStakeTiers>,
		OptionQuery,
	>;

	// --- Genesis Configuration ---

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		/// Initial staking tiers for credit accrual.
		/// Should be sorted by threshold ascending.
		pub stake_tiers: Vec<StakeTier<BalanceOf<T>>>,
	}

	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			GenesisConfig { stake_tiers: Default::default() }
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
		fn build(&self) {
			let bounded_tiers: BoundedVec<_, T::MaxStakeTiers> = self
				.stake_tiers
				.clone()
				.try_into()
				.expect("Genesis config stake_tiers exceed maximum length");
			// Ensure tiers are sorted by threshold, crucial for get_current_rate logic.
			// We expect genesis to provide sorted data, but panic here if not as it's a config
			// error.
			assert!(
				bounded_tiers.windows(2).all(|w| w[0].threshold <= w[1].threshold),
				"Genesis stake_tiers must be sorted by threshold ascending"
			);
			StoredStakeTiers::<T>::put(bounded_tiers);
		}
	}

	// --- Events ---
	/// Events emitted by this pallet.
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// TNT tokens were successfully burned, granting potential off-chain credits.
		/// Credits granted = amount_burned * conversion_rate.
		/// [who, amount_burned, credits_granted, offchain_account_id]
		CreditsGrantedFromBurn {
			who: T::AccountId,
			tnt_burned: BalanceOf<T>,
			credits_granted: BalanceOf<T>,
		},
		/// Credits were claimed from staking rewards, within the allowed window.
		/// [who, amount_claimed, offchain_account_id]
		CreditsClaimed {
			who: T::AccountId,
			amount_claimed: BalanceOf<T>,
			offchain_account_id: OffchainAccountIdOf<T>,
		},
		/// Stake tiers were updated.
		StakeTiersUpdated,
		/// Asset-specific stake tiers were updated.
		AssetStakeTiersUpdated { asset_id: T::AssetId },
	}

	// --- Errors ---
	#[pallet::error]
	pub enum Error<T> {
		/// Insufficient TNT balance to perform the burn operation.
		InsufficientTntBalance,
		/// The requested claim amount exceeds the maximum calculated within the allowed window.
		ClaimAmountExceedsWindowAllowance,
		/// Invalid claim ID (e.g., too long).
		InvalidClaimId,
		/// No stake tiers are configured or the stake amount is below the lowest tier threshold.
		NoValidTier,
		/// Amount specified for burn or claim must be greater than zero.
		AmountZero,
		/// Cannot transfer burned tokens to target account (feature not fully implemented).
		BurnTransferNotImplemented,
		/// The stake tiers are not properly sorted by threshold.
		StakeTiersNotSorted,
		/// There are no stake tiers provided for the update.
		EmptyStakeTiers,
		/// Amount overflowed.
		Overflow,
		/// The stake tiers are too large to fit into the storage.
		StakeTiersOverflow,
		/// No stake tiers configured for this asset.
		AssetRatesNotConfigured,
		/// Rate per block exceeds maximum allowed value.
		RateTooHigh,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Burn TNT for potential off-chain credits. Updates reward tracking block.
		#[pallet::call_index(0)]
		#[pallet::weight(T::WeightInfo::burn())]
		pub fn burn(
			origin: OriginFor<T>,
			#[pallet::compact] amount: BalanceOf<T>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(amount > Zero::zero(), Error::<T>::AmountZero);

			// Update reward block tracking first
			Self::update_reward_block(&who)?;

			Self::burn_tnt(&who, amount)?;

			let conversion_rate = T::BurnConversionRate::get();
			let credits_granted =
				amount.checked_mul(&conversion_rate).ok_or(Error::<T>::Overflow)?;
			ensure!(credits_granted > Zero::zero(), Error::<T>::Overflow);

			Self::deposit_event(Event::CreditsGrantedFromBurn {
				who,
				tnt_burned: amount,
				credits_granted,
			});
			Ok(())
		}

		/// Claim potential credits accrued within the allowed window. Emits event for off-chain
		/// processing.
		#[pallet::call_index(1)]
		#[pallet::weight(T::WeightInfo::claim_credits())]
		pub fn claim_credits(
			origin: OriginFor<T>,
			#[pallet::compact] amount_to_claim: BalanceOf<T>,
			offchain_account_id: OffchainAccountIdOf<T>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(amount_to_claim > Zero::zero(), Error::<T>::AmountZero);
			ensure!(
				offchain_account_id.len() <= T::MaxOffchainAccountIdLength::get() as usize,
				Error::<T>::InvalidClaimId
			);

			let current_block = frame_system::Pallet::<T>::block_number();

			// Calculate maximum claimable amount within the window and update the block tracker
			let max_claimable_in_window =
				Self::update_reward_block_and_get_accrued_amount(&who, current_block)?;

			// Verify requested amount against the calculated allowance
			ensure!(
				amount_to_claim <= max_claimable_in_window,
				Error::<T>::ClaimAmountExceedsWindowAllowance
			);

			// Emit event with the *requested* amount
			Self::deposit_event(Event::CreditsClaimed {
				who,
				amount_claimed: amount_to_claim,
				offchain_account_id,
			});
			Ok(())
		}

		/// Claim potential credits accrued within the allowed window for a specific asset.
		/// Emits event for off-chain processing.
		#[pallet::call_index(2)]
		#[pallet::weight(T::WeightInfo::claim_credits_with_asset())]
		pub fn claim_credits_with_asset(
			origin: OriginFor<T>,
			#[pallet::compact] amount_to_claim: BalanceOf<T>,
			offchain_account_id: OffchainAccountIdOf<T>,
			asset_id: T::AssetId,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(amount_to_claim > Zero::zero(), Error::<T>::AmountZero);
			ensure!(
				offchain_account_id.len() <= T::MaxOffchainAccountIdLength::get() as usize,
				Error::<T>::InvalidClaimId
			);

			let current_block = frame_system::Pallet::<T>::block_number();

			// Calculate maximum claimable amount for the specified asset
			let max_claimable_in_window =
				Self::update_reward_block_and_get_accrued_amount_for_asset(
					&who,
					current_block,
					asset_id,
				)?;

			// Verify requested amount against the calculated allowance
			ensure!(
				amount_to_claim <= max_claimable_in_window,
				Error::<T>::ClaimAmountExceedsWindowAllowance
			);

			// Emit event with the *requested* amount
			Self::deposit_event(Event::CreditsClaimed {
				who,
				amount_claimed: amount_to_claim,
				offchain_account_id,
			});
			Ok(())
		}

		/// Update the stake tiers. This function can only be called by the configured ForceOrigin.
		/// Stake tiers must be provided in ascending order by threshold.
		///
		/// Parameters:
		/// - `origin`: Must be the ForceOrigin
		/// - `new_tiers`: A vector of StakeTier structs representing the new tiers configuration
		///
		/// Emits `StakeTiersUpdated` on success.
		///
		/// Weight: O(n) where n is the number of tiers
		#[pallet::call_index(3)]
		#[pallet::weight(T::WeightInfo::set_stake_tiers())]
		pub fn set_stake_tiers(
			origin: OriginFor<T>,
			new_tiers: Vec<StakeTier<BalanceOf<T>>>,
		) -> DispatchResult {
			// Ensure the call is from the configured ForceOrigin
			T::ForceOrigin::ensure_origin(origin)?;

			// Check that we have at least one tier
			ensure!(!new_tiers.is_empty(), Error::<T>::EmptyStakeTiers);

			// Ensure tiers are properly sorted by threshold in ascending order
			for i in 1..new_tiers.len() {
				ensure!(
					new_tiers[i - 1].threshold <= new_tiers[i].threshold,
					Error::<T>::StakeTiersNotSorted
				);
			}

			// Validate that rates don't exceed maximum allowed value
			for tier in &new_tiers {
				let max_rate = BalanceOf::<T>::saturated_from(T::MaxRatePerBlock::get());
				ensure!(tier.rate_per_block <= max_rate, Error::<T>::RateTooHigh);
			}

			// Try to create a bounded vector
			let bounded_tiers =
				BoundedVec::<StakeTier<BalanceOf<T>>, T::MaxStakeTiers>::try_from(new_tiers)
					.map_err(|_| Error::<T>::StakeTiersOverflow)?;

			// Update storage
			StoredStakeTiers::<T>::set(bounded_tiers);

			// Emit event
			Self::deposit_event(Event::<T>::StakeTiersUpdated);

			Ok(())
		}

		/// Set stake tiers for a specific asset. This function can only be called by the configured
		/// ForceOrigin. Stake tiers must be provided in ascending order by threshold.
		///
		/// Parameters:
		/// - `origin`: Must be the ForceOrigin
		/// - `asset_id`: The asset ID to configure stake tiers for
		/// - `new_tiers`: A vector of StakeTier structs representing the new tiers configuration
		///   for this asset
		///
		/// Emits `AssetStakeTiersUpdated` on success.
		///
		/// Weight: O(n) where n is the number of tiers
		#[pallet::call_index(4)]
		#[pallet::weight(T::WeightInfo::set_asset_stake_tiers())]
		pub fn set_asset_stake_tiers(
			origin: OriginFor<T>,
			asset_id: T::AssetId,
			new_tiers: Vec<StakeTier<BalanceOf<T>>>,
		) -> DispatchResult {
			// Ensure the call is from the configured ForceOrigin
			T::ForceOrigin::ensure_origin(origin)?;

			// Check that we have at least one tier
			ensure!(!new_tiers.is_empty(), Error::<T>::EmptyStakeTiers);

			// Ensure tiers are properly sorted by threshold in ascending order
			for i in 1..new_tiers.len() {
				ensure!(
					new_tiers[i - 1].threshold <= new_tiers[i].threshold,
					Error::<T>::StakeTiersNotSorted
				);
			}

			// Validate that rates don't exceed maximum allowed value
			for tier in &new_tiers {
				let max_rate = BalanceOf::<T>::saturated_from(T::MaxRatePerBlock::get());
				ensure!(tier.rate_per_block <= max_rate, Error::<T>::RateTooHigh);
			}

			// Try to create a bounded vector
			let bounded_tiers =
				BoundedVec::<StakeTier<BalanceOf<T>>, T::MaxStakeTiers>::try_from(new_tiers)
					.map_err(|_| Error::<T>::StakeTiersOverflow)?;

			// Update storage for the specific asset
			AssetStakeTiers::<T>::insert(asset_id, bounded_tiers);

			// Emit event
			Self::deposit_event(Event::<T>::AssetStakeTiersUpdated { asset_id });

			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		/// Calculates potential credits accrued within the allowed window ending now,
		/// without updating the last reward block. This is useful for RPC queries.
		///
		/// ## Calculation Logic:
		/// 1. Determine the relevant time window for accrual:
		///     * `last_update` = `LastRewardUpdateBlock<T>::get(who)`
		///     * `window` = `T::ClaimWindowBlocks::get()`
		///     * `effective_start_block` = `max(last_update, current_block.saturating_sub(window))`
		///     * `effective_end_block` = `current_block`
		///     * If `effective_start_block >= effective_end_block`, accrued credits = 0.
		/// 2. Calculate the number of blocks in this window:
		///     * `blocks_in_window` = `effective_end_block.saturating_sub(effective_start_block)`
		/// 3. Fetch the user's current total staked TNT amount (`staked_amount`).
		/// 4. Determine the credit emission `rate` per block based on `staked_amount` using
		///    `get_current_rate`.
		/// 5. Calculate the accrued credits (using saturating math):
		///     * `accrued_credits` = `rate.saturating_mul(BalanceOf::<T>::from(blocks_in_window.
		///       unique_saturated_into::<u32>()))`
		///
		/// # Returns
		/// The calculated potential credits accrued within the window, or `DispatchError`.
		pub fn get_accrued_amount(
			who: &T::AccountId,
			current_block: Option<BlockNumberOf<T>>,
		) -> Result<BalanceOf<T>, DispatchError> {
			let current_block =
				current_block.unwrap_or_else(|| frame_system::Pallet::<T>::block_number());
			let last_update = LastRewardUpdateBlock::<T>::get(who);
			if last_update >= current_block {
				return Ok(Zero::zero());
			}

			let window = T::ClaimWindowBlocks::get();
			// Calculate the earliest block to consider for accrual (start of the window)
			let start_block = max(last_update, current_block.saturating_sub(window));

			// Ensure we don't calculate for blocks past the current one if window is large
			let effective_end_block = current_block;
			if start_block >= effective_end_block {
				return Ok(Zero::zero());
			}

			// Fetch stake *once* for the current block (simplification: assumes stake is constant
			// during window) A more complex approach could sample stake at intervals, but adds
			// significant complexity.
			let staked_amount =
				T::MultiAssetDelegationInfo::get_user_deposit_by_asset_type(who, AssetType::Tnt)
					.unwrap_or(Zero::zero());

			if staked_amount.is_zero() {
				return Ok(Zero::zero());
			}
			let rate = Self::get_current_rate(staked_amount);
			if rate.is_zero() {
				return Ok(Zero::zero());
			}

			// Calculate blocks within the effective window
			let blocks_in_window = effective_end_block.saturating_sub(start_block);
			// Already checked if start_block >= effective_end_block, so blocks_in_window > 0 here
			// unless effective_end_block == start_block, but that case is covered too.
			// We still check for zero just in case, although it should be unreachable.
			if blocks_in_window.is_zero() {
				return Ok(Zero::zero()); // Should be unreachable given prior checks
			}

			// Convert BlockNumber to u64 safely for the multiplication
			let blocks_in_window_u64: u64 = blocks_in_window.saturated_into();
			if blocks_in_window_u64 == 0 {
				return Ok(Zero::zero());
			}

			let new_credits =
				rate.saturating_mul(BalanceOf::<T>::saturated_from(blocks_in_window_u64));

			Ok(new_credits)
		}

		/// Calculates potential credits accrued within the allowed window ending now,
		/// and updates the last reward block.
		///
		/// This function calls `get_accrued_amount` to calculate the credits and then
		/// updates the `LastRewardUpdateBlock<T>` for the user to `current_block`.
		///
		/// # Returns
		/// The calculated potential credits accrued within the window, or `DispatchError`.
		pub fn update_reward_block_and_get_accrued_amount(
			who: &T::AccountId,
			current_block: BlockNumberOf<T>,
		) -> Result<BalanceOf<T>, DispatchError> {
			// Calculate accrued amount using the shared logic
			let result = Self::get_accrued_amount(who, Some(current_block));

			// Update the block regardless of calculation result
			LastRewardUpdateBlock::<T>::insert(who, current_block);

			result
		}

		/// Helper to ONLY update the reward block (e.g., for burn).
		pub fn update_reward_block(who: &T::AccountId) -> DispatchResult {
			let current_block = frame_system::Pallet::<T>::block_number();
			LastRewardUpdateBlock::<T>::mutate(who, |last_update| {
				if *last_update < current_block {
					*last_update = current_block;
				}
			});
			Ok(())
		}

		/// Burns TNT, returning an error if CreditBurnRecipient is set.
		fn burn_tnt(who: &T::AccountId, amount: BalanceOf<T>) -> DispatchResult {
			ensure!(T::Currency::free_balance(who) >= amount, Error::<T>::InsufficientTntBalance);

			match T::CreditBurnRecipient::get() {
				Some(recipient) => {
					T::Currency::ensure_can_withdraw(
						who,
						amount,
						frame_support::traits::WithdrawReasons::TRANSFER,
						T::Currency::free_balance(who).saturating_sub(amount),
					)?;
					T::Currency::transfer(who, &recipient, amount, ExistenceRequirement::KeepAlive)?
				},
				None => {
					let imbalance = T::Currency::withdraw(
						who,
						amount,
						frame_support::traits::WithdrawReasons::TRANSFER,
						ExistenceRequirement::KeepAlive,
					)?;
					drop(imbalance);
				},
			}
			Ok(())
		}

		/// Determines the credit emission rate per block based on the staked amount.
		/// Reads the tiers from storage.
		///
		/// # Arguments
		/// * `staked_amount`: The amount of staked TNT.
		///
		/// # Returns
		/// * `BalanceOf<T>`: The appropriate credit emission rate.
		pub fn get_current_rate(staked_amount: BalanceOf<T>) -> BalanceOf<T> {
			// Read tiers from storage
			let tiers = StoredStakeTiers::<T>::get();
			if tiers.is_empty() {
				// Handle case where no tiers are configured (e.g., during genesis or if cleared)
				return BalanceOf::<T>::zero();
			}

			// Iterate tiers in reverse (highest threshold first) because they are stored ascending
			for tier in tiers.iter().rev() {
				if staked_amount >= tier.threshold {
					return tier.rate_per_block;
				}
			}
			// If staked amount is below the lowest threshold, rate is zero.
			BalanceOf::<T>::zero()
		}

		/// Get credit emission rate for a specific asset and staked amount
		pub fn get_current_rate_for_asset(
			staked_amount: BalanceOf<T>,
			asset_id: T::AssetId,
		) -> Result<BalanceOf<T>, DispatchError> {
			// Try to get asset-specific tiers first
			if let Some(tiers) = AssetStakeTiers::<T>::get(asset_id) {
				for tier in tiers.iter().rev() {
					if staked_amount >= tier.threshold {
						return Ok(tier.rate_per_block);
					}
				}
				return Ok(BalanceOf::<T>::zero());
			}

			// If no asset-specific tiers, check if it's TNT (use global tiers for backward
			// compatibility)
			let native_asset_id = Default::default(); // Native asset ID (TNT)
			if asset_id == native_asset_id {
				return Ok(Self::get_current_rate(staked_amount));
			}

			// No rates configured for this asset
			Err(Error::<T>::AssetRatesNotConfigured.into())
		}

		/// Calculate accrued credits for a specific asset
		pub fn get_accrued_amount_for_asset(
			who: &T::AccountId,
			current_block: Option<BlockNumberOf<T>>,
			asset_id: T::AssetId,
		) -> Result<BalanceOf<T>, DispatchError> {
			let current_block =
				current_block.unwrap_or_else(|| frame_system::Pallet::<T>::block_number());
			let last_update = LastRewardUpdateBlock::<T>::get(who);
			if last_update >= current_block {
				return Ok(Zero::zero());
			}

			let window = T::ClaimWindowBlocks::get();
			let start_block = max(last_update, current_block.saturating_sub(window));
			let effective_end_block = current_block;

			if start_block >= effective_end_block {
				return Ok(Zero::zero());
			}

			// Get staked amount for the specified asset
			let staked_amount = Self::get_staked_amount_for_asset(who, asset_id)?;

			if staked_amount.is_zero() {
				return Ok(Zero::zero());
			}

			// Use asset-specific rate calculation
			let rate = Self::get_current_rate_for_asset(staked_amount, asset_id)?;

			if rate.is_zero() {
				return Ok(Zero::zero());
			}

			let blocks_in_window = effective_end_block.saturating_sub(start_block);
			if blocks_in_window.is_zero() {
				return Ok(Zero::zero());
			}

			let blocks_in_window_u64: u64 = blocks_in_window.saturated_into();
			if blocks_in_window_u64 == 0 {
				return Ok(Zero::zero());
			}

			let new_credits =
				rate.saturating_mul(BalanceOf::<T>::saturated_from(blocks_in_window_u64));
			Ok(new_credits)
		}

		/// Get staked amount for a specific asset
		fn get_staked_amount_for_asset(
			who: &T::AccountId,
			asset_id: T::AssetId,
		) -> Result<BalanceOf<T>, DispatchError> {
			let asset_type = Self::asset_id_to_asset_type(asset_id)?;

			let staked_amount =
				T::MultiAssetDelegationInfo::get_user_deposit_by_asset_type(who, asset_type)
					.unwrap_or(Zero::zero());

			Ok(staked_amount)
		}

		/// Convert AssetId to AssetType
		fn asset_id_to_asset_type(
			asset_id: T::AssetId,
		) -> Result<AssetType<T::AssetId>, DispatchError> {
			let native_asset_id = Default::default();
			if asset_id == native_asset_id {
				Ok(AssetType::Tnt)
			} else {
				Ok(AssetType::Native(asset_id))
			}
		}

		/// Update reward block and get accrued amount for a specific asset
		pub fn update_reward_block_and_get_accrued_amount_for_asset(
			who: &T::AccountId,
			current_block: BlockNumberOf<T>,
			asset_id: T::AssetId,
		) -> Result<BalanceOf<T>, DispatchError> {
			let result = Self::get_accrued_amount_for_asset(who, Some(current_block), asset_id);
			LastRewardUpdateBlock::<T>::insert(who, current_block);
			result
		}
	}
}
