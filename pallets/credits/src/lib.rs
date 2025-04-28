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
//! - An implementation of `tangle_primitives::traits::MultiAssetDelegationInfo` (`Config::StakingInfo`)
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

pub use pallet::*;

pub mod types;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[frame_support::pallet(dev_mode)]
pub mod pallet {
	use crate::types::*;
	use frame_support::{
		pallet_prelude::{ConstU32, *},
		traits::{
			tokens::{
				fungibles::{Inspect, Mutate},
				Fortitude, Precision, Preservation,
			},
			EnsureOriginWithArg,
		},
		PalletId,
	};
	use frame_system::pallet_prelude::*;
	use sp_arithmetic::Perbill;
	use sp_runtime::traits::{MaybeDisplay, Saturating, Zero};
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
	const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

	// Define BalanceOf here based on the Currency trait
	pub type BalanceOf<T> = <T as Config>::Currency::Balance;

	#[pallet::pallet]
	#[pallet::without_storage_info]
	// #[pallet::storage_version(STORAGE_VERSION)] // Comment out storage_version for now
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The overarching event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// The fungibles token trait for managing the TNT token.
		/// Ensure the implementation's Balance type satisfies necessary bounds (like Zero,
		/// From<BlockNumber> etc.)
		type Currency: Inspect<Self::AccountId, AssetId = Self::AssetId, Balance = BalanceOf<Self>>
			+ Mutate<Self::AccountId, AssetId = Self::AssetId, Balance = BalanceOf<Self>>;

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

		/// Defines the decay curve steps based on blocks since last interaction.
		#[pallet::constant]
		type DecaySteps: Get<BoundedVec<(BlockNumberOf<Self>, Perbill), ConstU32<MAX_DECAY_STEPS>>>;

		/// Optional: An account to send burned TNT to. If None, `Currency::burn_from` is used.
		#[pallet::constant]
		type CreditBurnTarget: Get<Option<Self::AccountId>>;

		/// Origin that can perform administrative actions.
		type AdminOrigin: EnsureOrigin<Self::RuntimeOrigin>;

		/// The maximum length allowed for an off-chain account ID string.
		#[pallet::constant]
		type MaxOffchainAccountIdLength: Get<u32>;

		/// The PalletId for deriving sovereign accounts.
		#[pallet::constant]
		type PalletId: Get<PalletId>;
	}

	// --- Storage Items ---

	#[pallet::storage]
	#[pallet::getter(fn credit_balance)]
	pub type CreditBalances<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, BalanceOf<T>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn linked_account)]
	pub type LinkedAccounts<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, OffchainAccountIdOf<T>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn last_reward_update_block)]
	pub type LastRewardUpdateBlock<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, BlockNumberOf<T>, ValueQuery>;

	/// Stores the block number of the last interaction (claim or update) for decay calculation.
	#[pallet::storage]
	#[pallet::getter(fn last_interaction_block)]
	pub type LastInteractionBlock<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, BlockNumberOf<T>, ValueQuery>;

	// --- Events ---
	/// Events emitted by this pallet.
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// An on-chain account has been successfully linked to an off-chain ID.
		/// \[who, offchain_account_id\]
		AccountLinked { who: T::AccountId, offchain_account_id: OffchainAccountIdOf<T> },
		/// TNT tokens were successfully burned in exchange for credits.
		/// \[who, tnt_burned, credits_granted\]
		CreditsBurned { who: T::AccountId, tnt_burned: BalanceOf<T>, credits_granted: BalanceOf<T> },
		/// Credits were successfully claimed (implying off-chain usage).
		/// The amount represents the effectively claimed value after decay.
		/// \[who, amount, offchain_account_id\]
		CreditsClaimed {
			who: T::AccountId,
			amount: BalanceOf<T>,
			offchain_account_id: OffchainAccountIdOf<T>,
		},
		/// A user's credit balance was updated due to staking rewards.
		/// \[who, new_credits, total_balance\]
		CreditsAccrued { who: T::AccountId, new_credits: BalanceOf<T>, total_balance: BalanceOf<T> },
		/// An admin force-set a user's credit balance.
		/// \[who, new_balance\]
		AdminCreditBalanceSet { who: T::AccountId, new_balance: BalanceOf<T> },
		/// An admin force-linked an account.
		/// \[who, offchain_account_id\]
		AdminAccountLinked { who: T::AccountId, offchain_account_id: OffchainAccountIdOf<T> },
	}

	// --- Errors ---
	#[pallet::error]
	pub enum Error<T> {
		InsufficientTntBalance,
		InsufficientCreditBalance,
		AccountNotLinked,
		OffchainAccountMismatch,
		OffchainAccountIdTooLong,
		AlreadyLinked,
		Overflow,
		NotStaking,
		NoStakeTiersConfigured,
		/// Amount to burn or claim must be greater than zero.
		AmountZero,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Links the sender's on-chain account to an off-chain identifier and resets decay timer.
		///
		/// # Arguments
		/// * `origin`: The origin of the call.
		/// * `offchain_account_id`: The off-chain account ID to link.
		///
		/// # Errors
		/// * `AlreadyLinked`: The account is already linked.
		/// * `Overflow`: The credit balance overflowed.
		///
		/// # Weight
		/// * `reads_writes(1, 2)`: Reads: LinkedAccounts. Writes: LinkedAccounts, LastInteractionBlock
		#[pallet::call_index(0)]
		#[pallet::weight(T::DbWeight::get().reads_writes(1, 2))]
		pub fn link_account(
			origin: OriginFor<T>,
			offchain_account_id: OffchainAccountIdOf<T>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(!LinkedAccounts::<T>::contains_key(&who), Error::<T>::AlreadyLinked);
			LinkedAccounts::<T>::insert(&who, offchain_account_id.clone());

			let current_block = frame_system::Pallet::<T>::block_number();
			LastInteractionBlock::<T>::insert(&who, current_block);

			Self::deposit_event(Event::AccountLinked { who, offchain_account_id });
			Ok(())
		}

		/// Burns TNT tokens for immediate credits. Accrues staking rewards first.
		///
		/// # Arguments
		/// * `origin`: The origin of the call.
		/// * `amount`: The amount of TNT to burn.
		///
		/// # Errors
		/// * `AmountZero`: The amount to burn must be greater than zero.
		/// * `InsufficientTntBalance`: The user does not have enough TNT to burn.
		/// * `Overflow`: The credit balance overflowed.
		///
		/// # Weight
		/// * `reads_writes(3, 3)`: Reads: Balance, LastUpdate, StakeInfo/Tiers. Writes: Balance, CreditBalance, LastUpdate
		#[pallet::call_index(1)]
		#[pallet::weight(T::DbWeight::get().reads_writes(3, 3))]
		pub fn burn(
			origin: OriginFor<T>,
			#[pallet::compact] amount: BalanceOf<T>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(amount > Zero::zero(), Error::<T>::AmountZero);

			Self::accrue_credits(&who)?; // Accrue staking rewards
			Self::burn_tnt(&who, amount)?; // Burn TNT

			let conversion_rate = T::BurnConversionRate::get();
			let credits_granted = amount.saturating_mul(conversion_rate);

			CreditBalances::<T>::try_mutate(&who, |balance| -> DispatchResult {
				*balance = balance.checked_add(&credits_granted).ok_or(Error::<T>::Overflow)?;
				Ok(())
			})?;
			// Burn does NOT update LastInteractionBlock
			Self::deposit_event(Event::CreditsBurned { who, tnt_burned: amount, credits_granted });
			Ok(())
		}

		/// Updates accrued credits based on staking and resets the decay timer.
		///
		/// # Arguments
		/// * `origin`: The origin of the call.
		///
		/// # Errors
		/// * `Overflow`: The credit balance overflowed.
		///
		/// # Weight
		/// * `reads_writes(4, 3)`: Reads: LastUpdate, StakeInfo, Tiers, LastInteract. Writes: LastUpdate, CreditBalance,
		#[pallet::call_index(2)]
		#[pallet::weight(T::DbWeight::get().reads_writes(4, 3))]
		pub fn trigger_credit_update(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			Self::accrue_credits(&who)?; // Accrue raw credits

			let current_block = frame_system::Pallet::<T>::block_number();
			LastInteractionBlock::<T>::insert(&who, current_block); // Reset decay timer
			Ok(())
		}

		/// Claims credits, applying decay based on time since last interaction.
		///
		/// # Arguments
		/// * `origin`: The origin of the call.
		/// * `amount_to_claim`: The amount of credits to claim.
		/// * `target_offchain_account_id`: The off-chain account ID to link.
		///
		/// # Errors
		/// * `AmountZero`: The amount to claim must be greater than zero.
		/// # Weight
		/// * `reads_writes(5, 3)`: Reads: LinkedAcc, LastInteract, LastUpdate, StakeInfo, Tiers. Writes: LastUpdate,
		/// CreditBalance, LastInteract
		#[pallet::call_index(3)]
		#[pallet::weight(T::DbWeight::get().reads_writes(5, 3))]
		pub fn claim_credits(
			origin: OriginFor<T>,
			#[pallet::compact] amount_to_claim: BalanceOf<T>,
			target_offchain_account_id: OffchainAccountIdOf<T>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(amount_to_claim > Zero::zero(), Error::<T>::AmountZero);

			let linked_account =
				LinkedAccounts::<T>::get(&who).ok_or(Error::<T>::AccountNotLinked)?;
			ensure!(
				linked_account == target_offchain_account_id,
				Error::<T>::OffchainAccountMismatch
			);

			Self::accrue_credits(&who)?; // Update raw balance

			let current_block = frame_system::Pallet::<T>::block_number();
			let last_interaction = LastInteractionBlock::<T>::get(&who);
			let elapsed_blocks = if last_interaction.is_zero() {
				Zero::zero()
			} else {
				current_block.saturating_sub(last_interaction)
			};
			let decay_factor = Self::calculate_decay_factor(elapsed_blocks);

			let raw_balance = CreditBalances::<T>::get(&who);
			let effective_claimable_balance = decay_factor.mul_floor(raw_balance);
			ensure!(
				amount_to_claim <= effective_claimable_balance,
				Error::<T>::InsufficientCreditBalance
			);

			// Deduct the equivalent raw amount that corresponds to the claimed decayed amount
			let raw_amount_to_deduct = if decay_factor.is_zero() {
				raw_balance // Should be unreachable if amount_to_claim > 0
			} else {
				decay_factor.saturating_reciprocal_mul_ceil(amount_to_claim)
			};

			CreditBalances::<T>::try_mutate(&who, |balance| -> DispatchResult {
				*balance = balance
					.checked_sub(&raw_amount_to_deduct)
					.ok_or(Error::<T>::InsufficientCreditBalance)?;
				Ok(())
			})?;

			LastInteractionBlock::<T>::insert(&who, current_block); // Reset decay timer

			Self::deposit_event(Event::CreditsClaimed {
				who,
				amount: amount_to_claim,
				offchain_account_id: linked_account,
			});
			Ok(())
		}

		/// Forcefully sets credit balance and resets decay timer. Requires AdminOrigin.
		///
		/// # Arguments
		/// * `origin`: The origin of the call.
		/// * `who`: The AccountId to set the balance for.
		/// * `new_balance`: The new balance to set.
		///
		/// # Weight
		/// * `writes(2)`: Writes: CreditBalance, LastInteraction
		#[pallet::call_index(4)]
		#[pallet::weight(T::DbWeight::get().writes(2))]
		pub fn force_set_credit_balance(
			origin: OriginFor<T>,
			who: T::AccountId,
			#[pallet::compact] new_balance: BalanceOf<T>,
		) -> DispatchResult {
			T::AdminOrigin::ensure_origin(origin)?;
			CreditBalances::<T>::insert(&who, new_balance);

			let current_block = frame_system::Pallet::<T>::block_number();
			LastInteractionBlock::<T>::insert(&who, current_block); // Reset decay

			Self::deposit_event(Event::AdminCreditBalanceSet { who, new_balance });
			Ok(())
		}

		/// Forcefully links an account and resets decay timer. Requires AdminOrigin.
		///
		/// # Arguments
		/// * `origin`: The origin of the call.
		/// * `who`: The AccountId to link.
		/// * `offchain_account_id`: The off-chain account ID to link.
		///
		/// # Weight
		/// * `writes(2)`: Writes: LinkedAccount, LastInteraction
		#[pallet::call_index(5)]
		#[pallet::weight(T::DbWeight::get().writes(2))]
		pub fn force_link_account(
			origin: OriginFor<T>,
			who: T::AccountId,
			offchain_account_id: OffchainAccountIdOf<T>,
		) -> DispatchResult {
			T::AdminOrigin::ensure_origin(origin)?;
			LinkedAccounts::<T>::insert(&who, offchain_account_id.clone());

			let current_block = frame_system::Pallet::<T>::block_number();
			LastInteractionBlock::<T>::insert(&who, current_block); // Reset decay

			Self::deposit_event(Event::AdminAccountLinked { who, offchain_account_id });
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		/// Determines the appropriate credit emission rate based on the staked amount.
		///
		/// # Arguments
		/// * `staked_amount`: The amount of staked TNT.
		///
		/// # Returns
		/// * `BalanceOf<T>`: The appropriate credit emission rate.
		fn get_current_rate(staked_amount: BalanceOf<T>) -> BalanceOf<T> {
			let tiers = T::StakeTiers::get();
			for tier in tiers.iter().rev() {
				if staked_amount >= tier.threshold {
					return tier.rate_per_block;
				}
			}
			BalanceOf::<T>::zero()
		}

		/// Calculates and adds accrued credits based on staking duration and amount.
		///
		/// # Arguments
		/// * `who`: The AccountId to accrue credits for.
		///
		/// # Errors
		/// * `Overflow`: The credit balance overflowed.
		fn accrue_credits(who: &T::AccountId) -> DispatchResult {
			let current_block = frame_system::Pallet::<T>::block_number();
			let last_update = LastRewardUpdateBlock::<T>::get(who);

			if last_update >= current_block {
				return Ok(()); // Already processed in this block
			}

			let tnt_asset_id = T::TntAssetId::get();
			let tnt_asset = tangle_primitives::services::Asset::Custom(tnt_asset_id);
			let maybe_deposit_info = T::StakingInfo::get_user_deposit_with_locks(who, tnt_asset);

			let staked_amount = match maybe_deposit_info {
				Some(deposit_info) => {
					let locked_total =
						deposit_info.amount_with_locks.map_or(BalanceOf::<T>::zero(), |locks| {
							locks.iter().fold(BalanceOf::<T>::zero(), |acc, lock| {
								acc.saturating_add(lock.amount)
							})
						});
					deposit_info.unlocked_amount.saturating_add(locked_total)
				},
				None => BalanceOf::<T>::zero(),
			};

			// Update last reward block *before* calculating rewards based on it
			LastRewardUpdateBlock::<T>::insert(who, current_block);

			if staked_amount.is_zero() {
				return Ok(());
			}

			let rate = Self::get_current_rate(staked_amount);
			if rate.is_zero() {
				return Ok(());
			}

			// Calculate blocks passed since the *previous* update block
			let blocks_since_last_update = current_block.saturating_sub(last_update);
			if blocks_since_last_update.is_zero() {
				return Ok(()); // Should not happen due to initial check
			}

			let new_credits = rate.saturating_mul(
				<BalanceOf<T>>::try_from(blocks_since_last_update).unwrap_or_else(|_| Zero::zero()),
			);

			if new_credits > BalanceOf::<T>::zero() {
				let mut final_balance = BalanceOf::<T>::zero();
				CreditBalances::<T>::try_mutate(who, |balance| -> DispatchResult {
					*balance = balance.checked_add(&new_credits).ok_or(Error::<T>::Overflow)?;
					final_balance = *balance; // Capture final balance for event
					Ok(())
				})?;
				Self::deposit_event(Event::CreditsAccrued {
					who: who.clone(),
					new_credits,
					total_balance: final_balance,
				});
			}

			Ok(())
		}

		/// Burns the specified amount of TNT from the user's account.
		///
		/// Checks sufficient balance first. If `CreditBurnTarget` is configured, attempts
		/// to transfer the tokens there (currently returns error as `Transfer` trait is not bound).
		/// Otherwise, uses `Currency::burn_from`.
		///
		/// # Arguments
		/// * `who`: The AccountId burning the tokens.
		/// * `amount`: The amount of TNT to burn.
		fn burn_tnt(who: &T::AccountId, amount: BalanceOf<T>) -> DispatchResult {
			let tnt_asset_id = T::TntAssetId::get();
			ensure!(
				T::Currency::balance(tnt_asset_id, who) >= amount,
				Error::<T>::InsufficientTntBalance
			);

			match T::CreditBurnTarget::get() {
				Some(target_account) => {
					Err(DispatchError::Other("CreditBurnTarget transfer not implemented"))
				},
				None => {
					T::Currency::burn_from(
						tnt_asset_id,
						who,
						amount,
						Preservation::Preserve,
						Precision::Exact,
						Fortitude::Dangerous,
					)?;
				},
			}
			Ok(())
		}

		/// Calculates the decay factor (percentage remaining) based on elapsed blocks since last
		/// interaction.
		///
		/// Reads `DecaySteps` from config (assumed sorted ascending by duration).
		/// Finds the latest step whose duration is less than or equal to `elapsed_blocks`
		/// and returns the corresponding `Perbill` factor.
		/// Returns `Perbill::one()` (100%) if no steps apply or elapsed time is zero.
		///
		/// # Arguments
		/// * `elapsed_blocks`: The number of blocks since the user's last interaction.
		fn calculate_decay_factor(elapsed_blocks: BlockNumberOf<T>) -> Perbill {
			let decay_steps = T::DecaySteps::get();
			if decay_steps.is_empty() || elapsed_blocks.is_zero() {
				return Perbill::one();
			}
			let mut applicable_factor = Perbill::one();
			for (duration, factor) in decay_steps.iter() {
				if elapsed_blocks >= *duration {
					applicable_factor = *factor;
				} else {
					break;
				}
			}
			applicable_factor
		}
	}
}
