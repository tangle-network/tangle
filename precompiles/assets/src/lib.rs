#![cfg_attr(not(feature = "std"), no_std)]

use fp_evm::PrecompileHandle;
use frame_support::{
	dispatch::{GetDispatchInfo, PostDispatchInfo},
	traits::{fungibles::Inspect, OriginTrait},
};
use pallet_evm::AddressMapping;
use parity_scale_codec::MaxEncodedLen;
use precompile_utils::{prelude::*, solidity};
use sp_core::U256;
use sp_runtime::traits::{Dispatchable, StaticLookup};
use sp_std::marker::PhantomData;

type BalanceOf<Runtime> = <Runtime as pallet_assets::Config>::Balance;

pub type AssetIdOf<Runtime> = <Runtime as pallet_assets::Config>::AssetIdParameter;
pub type RawAssetIdOf<Runtime> = <Runtime as pallet_assets::Config>::AssetId;

pub struct AssetsPrecompile<Runtime>(PhantomData<Runtime>);

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

impl<Runtime> AssetsPrecompile<Runtime>
where
	Runtime: pallet_assets::Config + pallet_evm::Config,
	Runtime::RuntimeCall: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	<Runtime::RuntimeCall as Dispatchable>::RuntimeOrigin: From<Option<<<Runtime as pallet_evm::Config>::AccountProvider as fp_evm::AccountProvider>::AccountId>>,
	Runtime::RuntimeCall: From<pallet_assets::Call<Runtime>>,
	Runtime::AccountId: From<<<Runtime as pallet_evm::Config>::AccountProvider as fp_evm::AccountProvider>::AccountId>,
	AssetIdOf<Runtime>: TryFrom<U256> + Into<U256>,
	RawAssetIdOf<Runtime>: TryFrom<U256> + Into<U256>,
	BalanceOf<Runtime>: TryFrom<U256> + Into<U256> + solidity::Codec,
{
	/// Helper method to convert U256 to AssetId
	fn u256_to_asset_id(asset_id: U256) -> EvmResult<AssetIdOf<Runtime>> {
		asset_id.try_into().map_err(|_| revert("Asset ID out of bounds"))
	}

	fn u256_to_raw_asset_id(asset_id: U256) -> EvmResult<RawAssetIdOf<Runtime>> {
		asset_id.try_into().map_err(|_| revert("Asset ID out of bounds"))
	}
}

#[precompile_utils::precompile]
impl<Runtime> AssetsPrecompile<Runtime>
where
	Runtime: pallet_assets::Config + pallet_evm::Config,
	Runtime::RuntimeCall: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	<Runtime::RuntimeCall as Dispatchable>::RuntimeOrigin: From<Option<<<Runtime as pallet_evm::Config>::AccountProvider as fp_evm::AccountProvider>::AccountId>>,
	Runtime::RuntimeCall: From<pallet_assets::Call<Runtime>>,
	Runtime::AccountId: From<<<Runtime as pallet_evm::Config>::AccountProvider as fp_evm::AccountProvider>::AccountId>,
	BalanceOf<Runtime>: TryFrom<U256> + Into<U256> + solidity::Codec,
	AssetIdOf<Runtime>: TryFrom<U256> + Into<U256>,
	RawAssetIdOf<Runtime>: TryFrom<U256> + Into<U256>,
	Runtime: tangle_primitives::traits::assets::NextAssetId<AssetIdOf<Runtime>>,
{
	#[precompile::public("create(uint256,address,uint256)")]
	fn create(
		handle: &mut impl PrecompileHandle,
		id: U256,
		admin: Address,
		min_balance: U256,
	) -> EvmResult {
		handle.record_cost(RuntimeHelper::<Runtime>::db_write_gas_cost())?;

		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let admin_evm = Runtime::AddressMapping::into_account_id(admin.0);
		let admin: Runtime::AccountId = admin_evm.into();
		let asset_id = Self::u256_to_asset_id(id)?;
		let min_balance: BalanceOf<Runtime> = min_balance
			.try_into()
			.map_err(|_| revert("Min balance amount exceeds bounds"))?;

		let call = pallet_assets::Call::<Runtime>::create {
			id: asset_id,
			admin: Runtime::Lookup::unlookup(admin),
			min_balance,
		};

		RuntimeHelper::<Runtime>::try_dispatch(handle, <Runtime as frame_system::Config>::RuntimeOrigin::signed(origin.into()), call, 0)?;
		Ok(())
	}

	#[precompile::public("startDestroy(uint256)")]
	fn start_destroy(handle: &mut impl PrecompileHandle, id: U256) -> EvmResult {
		handle.record_cost(RuntimeHelper::<Runtime>::db_write_gas_cost())?;

		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let asset_id = Self::u256_to_asset_id(id)?;

		let call = pallet_assets::Call::<Runtime>::start_destroy { id: asset_id };

		RuntimeHelper::<Runtime>::try_dispatch(handle, <Runtime as frame_system::Config>::RuntimeOrigin::signed(origin.into()), call, 0)?;
		Ok(())
	}

	// Add other asset operations like mint, burn, transfer
	#[precompile::public("mint(uint256,address,uint256)")]
	fn mint(
		handle: &mut impl PrecompileHandle,
		id: U256,
		beneficiary: Address,
		amount: U256,
	) -> EvmResult {
		handle.record_cost(RuntimeHelper::<Runtime>::db_write_gas_cost())?;

		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let beneficiary_evm = Runtime::AddressMapping::into_account_id(beneficiary.0);
		let beneficiary: Runtime::AccountId = beneficiary_evm.into();
		let asset_id = Self::u256_to_asset_id(id)?;
		let amount: BalanceOf<Runtime> =
			amount.try_into().map_err(|_| revert("Amount exceeds bounds"))?;

		let call = pallet_assets::Call::<Runtime>::mint {
			id: asset_id,
			beneficiary: Runtime::Lookup::unlookup(beneficiary),
			amount,
		};

		RuntimeHelper::<Runtime>::try_dispatch(handle, <Runtime as frame_system::Config>::RuntimeOrigin::signed(origin.into()), call, 0)?;
		Ok(())
	}

	#[precompile::public("transfer(uint256,address,uint256)")]
	fn transfer(
		handle: &mut impl PrecompileHandle,
		id: U256,
		target: Address,
		amount: U256,
	) -> EvmResult {
		handle.record_cost(RuntimeHelper::<Runtime>::db_write_gas_cost())?;

		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let target_evm = Runtime::AddressMapping::into_account_id(target.0);
		let target: Runtime::AccountId = target_evm.into();
		let asset_id = Self::u256_to_asset_id(id)?;
		let amount: BalanceOf<Runtime> =
			amount.try_into().map_err(|_| revert("Amount exceeds bounds"))?;

		let call = pallet_assets::Call::<Runtime>::transfer {
			id: asset_id,
			target: Runtime::Lookup::unlookup(target),
			amount,
		};

		RuntimeHelper::<Runtime>::try_dispatch(handle, <Runtime as frame_system::Config>::RuntimeOrigin::signed(origin.into()), call, 0)?;
		Ok(())
	}

	// ----- View functions ------ //
	#[precompile::public("totalSupply(uint256)")]
	#[precompile::view]
	fn total_supply(handle: &mut impl PrecompileHandle, asset_id: U256) -> EvmResult<U256> {
		// Storage item: Asset:
		// Blake2_128(16) + AssetId(16) + AssetDetails((4 * AccountId(20)) + (3 * Balance(16)) + 15)
		handle.record_db_read::<Runtime>(175)?;
		let asset_id = Self::u256_to_raw_asset_id(asset_id)?;
		Ok(pallet_assets::Pallet::<Runtime>::total_issuance(asset_id).into())
	}

	#[precompile::public("balanceOf(uint256,address)")]
	#[precompile::view]
	fn balance_of(
		handle: &mut impl PrecompileHandle,
		asset_id: U256,
		who: Address,
	) -> EvmResult<U256> {
		handle.record_db_read::<Runtime>(
			87 + <Runtime as pallet_assets::Config>::Extra::max_encoded_len(),
		)?;

		let who_evm = Runtime::AddressMapping::into_account_id(who.into());
		let who: Runtime::AccountId = who_evm.into();
		let asset_id = Self::u256_to_raw_asset_id(asset_id)?;

		let amount: U256 = { pallet_assets::Pallet::<Runtime>::balance(asset_id, &who).into() };
		Ok(amount)
	}

	#[precompile::public("nextAssetId()")]
	#[precompile::view]
	fn next_asset_id(handle: &mut impl PrecompileHandle) -> EvmResult<U256> {
		// Storage item: Asset:
		// Blake2_128(16) + AssetId(16) + AssetDetails((4 * AccountId(20)) + (3 * Balance(16)) + 15)
		handle.record_db_read::<Runtime>(175)?;
		let next_asset_id =
			Runtime::next_asset_id().ok_or_else(|| revert("No next asset ID available"))?;
		Ok(next_asset_id.into())
	}
}
