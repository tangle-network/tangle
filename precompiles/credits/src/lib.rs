#![cfg_attr(not(feature = "std"), no_std)]

use fp_evm::PrecompileHandle;
use frame_support::dispatch::{GetDispatchInfo, PostDispatchInfo};
use pallet_credits::types::{BlockNumberOf, OffchainAccountIdOf, StakeTier};
use pallet_evm::AddressMapping;
use precompile_utils::{prelude::*, solidity};
use sp_core::U256;
use sp_runtime::traits::{Dispatchable, UniqueSaturatedInto};
use sp_std::{marker::PhantomData, vec, vec::Vec};

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

/// A precompile to wrap the functionality from pallet-credits.
pub struct CreditsPrecompile<Runtime>(PhantomData<Runtime>);

impl<Runtime> CreditsPrecompile<Runtime>
where
	Runtime: pallet_credits::Config + pallet_evm::Config,
	Runtime::RuntimeCall: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	<Runtime::RuntimeCall as Dispatchable>::RuntimeOrigin: From<Option<Runtime::AccountId>>,
	Runtime::RuntimeCall: From<pallet_credits::Call<Runtime>>,
{
	pub fn new() -> Self {
		Self(PhantomData)
	}

	/// Helper to convert U256 to Balance
	fn u256_to_balance(value: U256) -> EvmResult<pallet_credits::BalanceOf<Runtime>> {
		value.try_into().map_err(|_| revert("Amount overflow"))
	}

	/// Helper to convert Balance to U256
	fn balance_to_u256(value: pallet_credits::BalanceOf<Runtime>) -> U256 {
		value.unique_saturated_into()
	}
}

#[precompile_utils::precompile]
impl<Runtime> CreditsPrecompile<Runtime>
where
	Runtime: pallet_credits::Config + pallet_evm::Config,
	Runtime::RuntimeCall: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	<Runtime::RuntimeCall as Dispatchable>::RuntimeOrigin: From<Option<Runtime::AccountId>>,
	Runtime::RuntimeCall: From<pallet_credits::Call<Runtime>>,
{
	#[precompile::public("burn(uint256)")]
	fn burn(handle: &mut impl PrecompileHandle, amount: U256) -> EvmResult<bool> {
		handle.record_cost(RuntimeHelper::<Runtime>::db_write_gas_cost())?;

		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let amount = Self::u256_to_balance(amount)?;

		let call = pallet_credits::Call::<Runtime>::burn { amount };

		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;
		Ok(true)
	}

	#[precompile::public("claim_credits(uint256,bytes)")]
	fn claim_credits(
		handle: &mut impl PrecompileHandle,
		amount_to_claim: U256,
		offchain_account_id: BoundedBytes<Runtime::MaxOffchainAccountIdLength>,
	) -> EvmResult<bool> {
		handle.record_cost(RuntimeHelper::<Runtime>::db_write_gas_cost())?;

		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let amount_to_claim = Self::u256_to_balance(amount_to_claim)?;

		// Convert BoundedBytes to BoundedVec<u8>
		let offchain_account_id_bytes: Vec<u8> = offchain_account_id.into();
		let offchain_account_id: OffchainAccountIdOf<Runtime> = offchain_account_id_bytes
			.try_into()
			.map_err(|_| revert("Offchain account ID too long"))?;

		let call =
			pallet_credits::Call::<Runtime>::claim_credits { amount_to_claim, offchain_account_id };

		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;
		Ok(true)
	}

	#[precompile::public("get_current_rate(uint256)")]
	#[precompile::view]
	fn get_current_rate(
		handle: &mut impl PrecompileHandle,
		staked_amount: U256,
	) -> EvmResult<U256> {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		let staked_amount = Self::u256_to_balance(staked_amount)?;
		let rate = pallet_credits::Pallet::<Runtime>::get_current_rate(staked_amount);

		Ok(Self::balance_to_u256(rate))
	}

	#[precompile::public("calculate_accrued_credits(address)")]
	#[precompile::view]
	fn calculate_accrued_credits(
		handle: &mut impl PrecompileHandle,
		account: Address,
	) -> EvmResult<U256> {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		let account_id = Runtime::AddressMapping::into_account_id(account.into());
		let current_block = frame_system::Pallet::<Runtime>::block_number();

		// Call the internal pallet function to calculate accrued credits
		let accrued_amount =
			pallet_credits::Pallet::<Runtime>::update_reward_block_and_get_accrued_amount(
				&account_id,
				current_block,
			)
			.unwrap_or_else(|_| Default::default());

		Ok(Self::balance_to_u256(accrued_amount))
	}

	#[precompile::public("get_stake_tiers()")]
	#[precompile::view]
	fn get_stake_tiers(handle: &mut impl PrecompileHandle) -> EvmResult<(Vec<U256>, Vec<U256>)> {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		let tiers = pallet_credits::Pallet::<Runtime>::stake_tiers();
		let mut thresholds = Vec::new();
		let mut rates = Vec::new();

		for tier in tiers.iter() {
			thresholds.push(Self::balance_to_u256(tier.threshold));
			rates.push(Self::balance_to_u256(tier.rate_per_block));
		}

		Ok((thresholds, rates))
	}
}
