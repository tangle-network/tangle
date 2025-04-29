//! # Cloud Credits Pallet
//!
//! ## Overview
//!
//! The Cloud Credits pallet provides an on-chain mechanism for users to acquire and manage
//! usage credits, primarily intended for accessing off-chain services like AI assistants.
//! It integrates with a staking system (like `pallet-multi-asset-delegation`) to reward
//! users who stake TNT tokens with passively accrued credits.
//!
//! ### Key Features:
//! - **Staking-Based Credit Accrual:** Users automatically earn credits based on the amount
//!   of TNT they have staked via a configured `StakingInfo` provider. Credit emission rates
//!   are tiered based on stake size.
//! - **TNT Burning:** Users can burn TNT tokens for an immediate, one-time grant of credits.
//! - **Account Linking:** Users link their on-chain account (`AccountId`) to an off-chain
//!   identifier (e.g., GitHub handle, email hash) to facilitate off-chain credit redemption.
//! - **Credit Claiming:** Users initiate a claim on-chain, signaling the intention to use
//!   credits off-chain. This reduces the on-chain balance.
//! - **Activity-Based Decay:** To strongly incentivize weekly interaction, the
//!   *claimable* portion of a user's credit balance decays significantly if they do not
//!   claim or actively update their credits frequently (ideally weekly). The raw accrued balance
//!   remains, but its effective claimable value diminishes rapidly with prolonged inactivity.
//! - **Admin Controls:** Provides administrative functions to manage credit balances and account links.
//!
//! ## Integration
//!
//! This pallet relies on:
//! - An implementation of `MultiAssetDelegationInfo` (`Config::StakingInfo`)
//!   to query the active TNT stake for users.
//! - An implementation of `frame_support::traits::tokens::fungibles` (`Config::Currency`) to handle
//!   TNT token balances and burning.
//! - `frame_system` for basic system types and block numbers.
//! - `sp_arithmetic::Perbill` for decay calculations.
//!
//! ## Terminology
//! - **TNT:** The primary utility token used for staking and burning.
//! - **Credits:** An on-chain numerical balance representing usage rights for off-chain services.
//!   Credits are not transferable tokens themselves.
//! - **Staking:** Locking TNT tokens via the `StakingInfo` provider (e.g., `pallet-multi-asset-delegation`).
//! - **Burning:** Permanently destroying TNT tokens in exchange for immediate credits.
//! - **Linking:** Associating an on-chain `AccountId` with an off-chain identifier.
//! - **Claiming:** Reducing the on-chain credit balance, implying off-chain usage.
//! - **Interaction:** An action (linking, claiming, triggering update, admin action) that resets the decay timer.
//! - **Decay:** Reduction in the *claimable percentage* of the raw credit balance over time due to inactivity. Designed to be aggressive after a grace period (e.g., 1 week) to encourage regular claims.

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::traits::Currency;
pub use pallet::*;

pub mod types;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

#[frame_support::pallet(dev_mode)]
pub mod pallet {
	use crate::{types::*, BalanceOf};
	use core::cmp::max;
	use frame_support::{
		pallet_prelude::{ConstU32, *},
		traits::{
			tokens::{
				fungibles::{Inspect, Mutate},
				Fortitude, Precision, Preservation,
			},
			Currency, EnsureOriginWithArg, ExistenceRequirement, LockableCurrency,
			ReservableCurrency,
		},
		PalletId,
	};
	use frame_system::pallet_prelude::*;
	use sp_arithmetic::Perbill;
	use sp_runtime::traits::{CheckedSub, MaybeDisplay, Saturating, Zero};
	use sp_std::fmt::Debug;
	use tangle_primitives::traits::MultiAssetDelegationInfo;

	// Define Max Decay Steps Const
	#[cfg(feature = "runtime-benchmarks")]
	pub const MAX_DECAY_STEPS: u32 = 5;
	#[cfg(not(feature = "runtime-benchmarks"))]
	pub const MAX_DECAY_STEPS: u32 = 5;

	// Define Max Tiers Const
	#[cfg(feature = "runtime-benchmarks")]
	pub const MAX_TIERS: u32 = 10;
	#[cfg(not(feature = "runtime-benchmarks"))]
	pub const MAX_TIERS: u32 = 10;

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

		/// The Asset ID type used by the Currency trait and StakingInfo.
		type AssetId: Parameter + Member + MaybeDisplay + Ord + MaxEncodedLen + Copy + Debug;

		/// The specific Asset ID for the TNT token.
		#[pallet::constant]
		type TntAssetId: Get<Self::AssetId>;

		/// The provider for checking the active TNT stake.
		/// Ensure BalanceOf<Self> here resolves correctly to T::Currency::Balance.
		type StakingInfo: MultiAssetDelegationInfo<
			Self::AccountId,
			BalanceOf<Self>,
			BlockNumberOf<Self>,
			Self::AssetId,
		>;

		/// The defined staking tiers for credit emission, sorted by threshold ascending.
		#[pallet::constant]
		type StakeTiers: Get<BoundedVec<StakeTier<BalanceOf<Self>>, ConstU32<MAX_TIERS>>>;

		/// The conversion rate for burning TNT to credits.
		#[pallet::constant]
		type BurnConversionRate: Get<BalanceOf<Self>>;

		/// The maximum window (in blocks) for which credits can be accrued before claiming.
		#[pallet::constant]
		type ClaimWindowBlocks: Get<BlockNumberOf<Self>>;

		/// Optional: An account to send burned TNT to. If None, `Currency::burn_from` is used.
		#[pallet::constant]
		type CreditBurnTarget: Get<Option<Self::AccountId>>;

		/// Origin that can perform administrative actions.
		type AdminOrigin: EnsureOrigin<Self::RuntimeOrigin>;

		/// The maximum length allowed for an off-chain account ID string.
		#[pallet::constant]
		type MaxOffchainAccountIdLength: Get<u32>;

		/// The maximum decay steps.
		#[pallet::constant]
		type MaxDecaySteps: Get<u32>;

		/// The maximum tiers.
		#[pallet::constant]
		type MaxTiers: Get<u32>;
	}

	// --- Storage Items ---

	#[pallet::storage]
	#[pallet::getter(fn last_reward_update_block)]
	pub type LastRewardUpdateBlock<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, BlockNumberOf<T>, ValueQuery>;

	// --- Events ---
	/// Events emitted by this pallet.
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// TNT tokens were successfully burned, granting potential off-chain credits.
		/// \[who, tnt_burned, credits_granted]
		CreditsGrantedFromBurn {
			who: T::AccountId,
			tnt_burned: BalanceOf<T>,
			credits_granted: BalanceOf<T>,
		},
		/// A user successfully claimed credits, emitting details for off-chain processing.
		/// The amount is the value requested by the user, verified against the claimable window.
		/// \[who, amount_claimed, offchain_account_id]
		CreditsClaimed {
			who: T::AccountId,
			amount_claimed: BalanceOf<T>,
			offchain_account_id: OffchainAccountIdOf<T>,
		},
	}

	// --- Errors ---
	#[pallet::error]
	pub enum Error<T> {
		/// Insufficient TNT balance to perform the burn operation.
		InsufficientTntBalance,
		/// The requested claim amount exceeds the maximum calculated within the allowed window.
		ClaimAmountExceedsWindowAllowance,
		/// The provided off-chain account ID exceeds the maximum allowed length.
		OffchainAccountIdTooLong,
		/// An arithmetic operation resulted in an overflow.
		Overflow,
		/// No staking tiers are configured in the runtime.
		NoStakeTiersConfigured,
		/// Amount specified for burn or claim must be greater than zero.
		AmountZero,
		/// Cannot transfer burned tokens to target account (feature not fully implemented).
		BurnTransferNotImplemented,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Burn TNT for potential off-chain credits. Updates reward tracking block.
		#[pallet::call_index(0)]
		#[pallet::weight(T::DbWeight::get().reads_writes(2, 2))]
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
			let credits_granted = amount.saturating_mul(conversion_rate);
			ensure!(credits_granted > Zero::zero(), Error::<T>::Overflow);

			Self::deposit_event(Event::CreditsGrantedFromBurn {
				who,
				tnt_burned: amount,
				credits_granted,
			});
			Ok(())
		}

		/// Claim potential credits accrued within the allowed window. Emits event for off-chain processing.
		#[pallet::call_index(1)]
		#[pallet::weight(T::DbWeight::get().reads_writes(2, 1))]
		pub fn claim_credits(
			origin: OriginFor<T>,
			#[pallet::compact] amount_to_claim: BalanceOf<T>,
			offchain_account_id: OffchainAccountIdOf<T>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(amount_to_claim > Zero::zero(), Error::<T>::AmountZero);
			ensure!(
				offchain_account_id.len() <= T::MaxOffchainAccountIdLength::get() as usize,
				Error::<T>::OffchainAccountIdTooLong
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
	}

	impl<T: Config> Pallet<T> {
		/// Calculates potential credits accrued within the allowed window ending now,
		/// and updates the last reward block.
		fn update_reward_block_and_get_accrued_amount(
			who: &T::AccountId,
			current_block: BlockNumberOf<T>,
		) -> Result<BalanceOf<T>, DispatchError> {
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

			// Fetch stake *once* for the current block (simplification: assumes stake is constant during window)
			// A more complex approach could sample stake at intervals, but adds significant complexity.
			let tnt_asset_id = T::TntAssetId::get();
			let tnt_asset = tangle_primitives::services::Asset::Custom(tnt_asset_id);
			let maybe_deposit_info = T::StakingInfo::get_user_deposit_with_locks(who, tnt_asset);
			let staked_amount = maybe_deposit_info.map_or(Zero::zero(), |deposit_info| {
				let locked_total = deposit_info.amount_with_locks.map_or(Zero::zero(), |locks| {
					locks.iter().fold(Zero::zero(), |acc, lock| acc.saturating_add(lock.amount))
				});
				deposit_info.unlocked_amount.saturating_add(locked_total)
			});

			// Update the block *before* calculation
			LastRewardUpdateBlock::<T>::insert(who, current_block);

			if staked_amount.is_zero() {
				return Ok(Zero::zero());
			}
			let rate = Self::get_current_rate(staked_amount);
			if rate.is_zero() {
				return Ok(Zero::zero());
			}

			// Calculate blocks within the effective window
			let blocks_in_window = effective_end_block.saturating_sub(start_block);
			if blocks_in_window.is_zero() {
				return Ok(Zero::zero());
			}

			let multiplier =
				<BalanceOf<T>>::try_from(blocks_in_window).map_err(|_| Error::<T>::Overflow)?;
			let new_credits = rate.saturating_mul(multiplier);

			Ok(new_credits)
		}

		/// Helper to ONLY update the reward block (e.g., for burn).
		fn update_reward_block(who: &T::AccountId) -> DispatchResult {
			let current_block = frame_system::Pallet::<T>::block_number();
			let last_update = LastRewardUpdateBlock::<T>::get(who);
			if last_update < current_block {
				LastRewardUpdateBlock::<T>::insert(who, current_block);
			}
			Ok(())
		}

		/// Burns TNT, returning an error if CreditBurnTarget is set.
		fn burn_tnt(who: &T::AccountId, amount: BalanceOf<T>) -> DispatchResult {
			ensure!(T::Currency::free_balance(who) >= amount, Error::<T>::InsufficientTntBalance);

			match T::CreditBurnTarget::get() {
				Some(_) => Err(Error::<T>::BurnTransferNotImplemented.into()),
				None => {
					T::Currency::transfer(who, who, amount, ExistenceRequirement::KeepAlive)?;
					Ok(())
				},
			}
		}

		/// Determines the credit emission rate per block based on the staked amount.
		///
		/// # Arguments
		/// * `staked_amount`: The amount of staked TNT.
		///
		/// # Returns
		/// * `BalanceOf<T>`: The appropriate credit emission rate.
		pub(crate) fn get_current_rate(staked_amount: BalanceOf<T>) -> BalanceOf<T> {
			let tiers = T::StakeTiers::get();
			for tier in tiers.iter().rev() {
				if staked_amount >= tier.threshold {
					return tier.rate_per_block;
				}
			}
			BalanceOf::<T>::zero()
		}
	}
}
