#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::too_many_arguments)]

use fp_evm::{PrecompileFailure, PrecompileHandle};
use frame_support::dispatch::{GetDispatchInfo, PostDispatchInfo};
use frame_system::pallet_prelude::BlockNumberFor;
use pallet_evm::AddressMapping;
use pallet_services::types::BalanceOf;
use parity_scale_codec::Decode;
use precompile_utils::prelude::*;
use sp_core::U256;
use sp_runtime::{traits::Dispatchable, Percent};
use sp_std::{marker::PhantomData, vec, vec::Vec};
use tangle_primitives::services::{
	Asset, AssetSecurityRequirement, Field, MembershipModel, ServiceBlueprint,
};

#[cfg(test)]
mod mock;
#[cfg(test)]
mod mock_evm;
#[cfg(test)]
mod tests;

/// Precompile for the `Services` pallet.
pub struct ServicesPrecompile<Runtime>(PhantomData<Runtime>);

#[precompile_utils::precompile]
impl<Runtime> ServicesPrecompile<Runtime>
where
	Runtime: pallet_services::Config + pallet_evm::Config,
	Runtime::RuntimeCall: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	<Runtime::RuntimeCall as Dispatchable>::RuntimeOrigin: From<Option<Runtime::AccountId>>,
	Runtime::RuntimeCall: From<pallet_services::Call<Runtime>>,
{
	// Errors for the `Services` precompile.

	/// Found an invalid permitted callers list.
	const INVALID_PERMITTED_CALLERS: [u8; 32] = keccak256!("InvalidPermittedCallers()");
	/// Found an invalid service providers list.
	const INVALID_OPERATORS_LIST: [u8; 32] = keccak256!("InvalidOperatorsList()");
	/// Found an invalid request arguments.
	const INVALID_REQUEST_ARGUMENTS: [u8; 32] = keccak256!("InvalidRequestArguments()");
	/// Invalid TTL.
	const INVALID_TTL: [u8; 32] = keccak256!("InvalidTTL()");
	/// Found an invalid amount / value.
	const INVALID_AMOUNT: [u8; 32] = keccak256!("InvalidAmount()");
	/// Value must be zero for ERC20 payment asset.
	const VALUE_NOT_ZERO_FOR_ERC20: [u8; 32] = keccak256!("ValueMustBeZeroForERC20()");
	/// Value must be zero for custom payment asset.
	const VALUE_NOT_ZERO_FOR_CUSTOM_ASSET: [u8; 32] = keccak256!("ValueMustBeZeroForCustomAsset()");
	/// Payment asset should be either custom or ERC20.
	const PAYMENT_ASSET_SHOULD_BE_CUSTOM_OR_ERC20: [u8; 32] =
		keccak256!("PaymentAssetShouldBeCustomOrERC20()");

	/// Create a new blueprint.
	#[precompile::public("createBlueprint(bytes)")]
	fn create_blueprint(
		handle: &mut impl PrecompileHandle,
		blueprint_data: UnboundedBytes,
	) -> EvmResult {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);

		let blueprint_data: Vec<u8> = blueprint_data.into();
		let blueprint: ServiceBlueprint<Runtime::Constraints> =
			Decode::decode(&mut &blueprint_data[..])
				.map_err(|_| revert("Invalid blueprint data"))?;

		let call = pallet_services::Call::<Runtime>::create_blueprint {
			metadata: blueprint.metadata.name.as_str().as_bytes().to_vec().try_into().unwrap(),
			typedef: ServiceBlueprint {
				metadata: blueprint.metadata.clone(),
				jobs: blueprint.jobs.clone(),
				registration_params: blueprint.registration_params.clone(),
				request_params: blueprint.request_params.clone(),
				manager: blueprint.manager,
				master_manager_revision: blueprint.master_manager_revision,
				sources: blueprint.sources.clone(),
				supported_membership_models: blueprint.supported_membership_models.clone(),
				pricing_model: tangle_primitives::services::PricingModel::PayOnce {
					amount: Default::default(),
				},
			},
			membership_model: MembershipModel::Fixed { min_operators: 1 },
			security_requirements: vec![],
			price_targets: None,
			pricing_model: tangle_primitives::services::PricingModel::PayOnce {
				amount: Default::default(),
			},
		};

		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(())
	}

	/// Request a new service.
	#[precompile::public(
		"requestService(uint256,bytes[],bytes,bytes,bytes,uint256,uint256,address,uint256,uint32,uint32)"
	)]
	#[precompile::payable]
	fn request_service(
		handle: &mut impl PrecompileHandle,
		blueprint_id: U256,
		asset_security_requirements: Vec<UnboundedBytes>,
		permitted_callers_data: UnboundedBytes,
		service_providers_data: UnboundedBytes,
		request_args_data: UnboundedBytes,
		ttl: U256,
		payment_asset_id: U256,
		payment_token_address: Address,
		amount: U256,
		min_operators: u32,
		max_operators: u32,
	) -> EvmResult {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let msg_sender = handle.context().caller;
		let origin = Runtime::AddressMapping::into_account_id(msg_sender);

		let blueprint_id: u64 = blueprint_id.as_u64();
		let asset_security_requirements_data: Vec<Vec<u8>> =
			asset_security_requirements.into_iter().map(|x| x.into()).collect();
		let permitted_callers_data: Vec<u8> = permitted_callers_data.into();
		let service_providers_data: Vec<u8> = service_providers_data.into();
		let request_args_data: Vec<u8> = request_args_data.into();

		let permitted_callers: Vec<Runtime::AccountId> =
			Decode::decode(&mut &permitted_callers_data[..])
				.map_err(|_| revert_custom_error(Self::INVALID_PERMITTED_CALLERS))?;

		let operators: Vec<Runtime::AccountId> =
			Decode::decode(&mut &service_providers_data[..])
				.map_err(|_| revert_custom_error(Self::INVALID_OPERATORS_LIST))?;

		let request_args: Vec<Field<Runtime::Constraints, Runtime::AccountId>> =
			Decode::decode(&mut &request_args_data[..])
				.map_err(|_| revert_custom_error(Self::INVALID_REQUEST_ARGUMENTS))?;

		let asset_security_requirements: Vec<AssetSecurityRequirement<Runtime::AssetId>> =
			asset_security_requirements_data
				.into_iter()
				.map(|req| Decode::decode(&mut &req[..]))
				.collect::<Result<_, _>>()
				.map_err(|_| revert_custom_error(Self::INVALID_REQUEST_ARGUMENTS))?;

		let value_bytes = {
			let value = handle.context().apparent_value;
			let mut value_bytes = [0u8; core::mem::size_of::<U256>()];
			value.to_little_endian(&mut value_bytes);
			value_bytes
		};
		let value = BalanceOf::<Runtime>::decode(&mut &value_bytes[..])
			.map_err(|_| revert_custom_error(Self::INVALID_AMOUNT))?;

		let ttl_bytes = {
			let mut ttl_bytes = [0u8; core::mem::size_of::<U256>()];
			ttl.to_little_endian(&mut ttl_bytes);
			ttl_bytes
		};

		let ttl = BlockNumberFor::<Runtime>::decode(&mut &ttl_bytes[..])
			.map_err(|_| revert_custom_error(Self::INVALID_TTL))?;

		let amount = {
			let mut amount_bytes = [0u8; core::mem::size_of::<U256>()];
			amount.to_little_endian(&mut amount_bytes);
			BalanceOf::<Runtime>::decode(&mut &amount_bytes[..])
				.map_err(|_| revert_custom_error(Self::INVALID_AMOUNT))?
		};

		const ZERO_ADDRESS: [u8; 20] = [0; 20];

		let (payment_asset, amount) = match (payment_asset_id.as_u32(), payment_token_address.0 .0)
		{
			(0, ZERO_ADDRESS) => (Asset::Custom(0u32.into()), value),
			(0, erc20_token) => {
				if value != Default::default() {
					return Err(revert_custom_error(Self::VALUE_NOT_ZERO_FOR_ERC20));
				}
				(Asset::Erc20(erc20_token.into()), amount)
			},
			(other_asset_id, ZERO_ADDRESS) => {
				if value != Default::default() {
					return Err(revert_custom_error(Self::VALUE_NOT_ZERO_FOR_CUSTOM_ASSET));
				}
				(Asset::Custom(other_asset_id.into()), amount)
			},
			(_other_asset_id, _erc20_token) =>
				return Err(revert_custom_error(Self::PAYMENT_ASSET_SHOULD_BE_CUSTOM_OR_ERC20)),
		};

		let membership_model = if max_operators == 0 {
			MembershipModel::Fixed { min_operators }
		} else if max_operators == u32::MAX {
			MembershipModel::Dynamic { min_operators, max_operators: None }
		} else {
			MembershipModel::Dynamic { min_operators, max_operators: Some(max_operators) }
		};

		let call = pallet_services::Call::<Runtime>::request {
			evm_origin: Some(msg_sender),
			blueprint_id,
			permitted_callers,
			operators,
			request_args,
			asset_security_requirements,
			ttl,
			payment_asset,
			value: amount,
			membership_model,
		};

		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(())
	}

	/// Terminate a service.
	#[precompile::public("terminateService(uint256)")]
	fn terminate_service(handle: &mut impl PrecompileHandle, service_id: U256) -> EvmResult {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);

		let service_id: u64 = service_id.as_u64();

		let call = pallet_services::Call::<Runtime>::terminate { service_id };

		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(())
	}

	/// Call a job in the service.
	#[precompile::public("callJob(uint256,uint8,bytes)")]
	fn call_job(
		handle: &mut impl PrecompileHandle,
		service_id: U256,
		job: u8,
		args_data: UnboundedBytes,
	) -> EvmResult {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let service_id: u64 = service_id.as_u64();
		let args: Vec<u8> = args_data.into();

		let decoded_args: Vec<Field<Runtime::Constraints, Runtime::AccountId>> =
			Decode::decode(&mut &args[..])
				.map_err(|_| revert("Invalid job call arguments data"))?;

		let call = pallet_services::Call::<Runtime>::call { service_id, job, args: decoded_args };

		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(())
	}

	/// Slash an operator (offender) for a service id with a given percent of their exposed stake
	/// for that service.
	///
	/// The caller needs to be an authorized Slash Origin for this service.
	/// Note that this does not apply the slash directly, but instead schedules a deferred call to
	/// apply the slash by another entity.
	#[precompile::public("slash(bytes,uint256,uint8)")]
	fn slash(
		handle: &mut impl PrecompileHandle,
		offender: UnboundedBytes,
		service_id: U256,
		percent: u8,
	) -> EvmResult {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let caller = handle.context().caller;
		let origin = Runtime::AddressMapping::into_account_id(caller);
		let service_id: u64 = service_id.as_u64();
		let percent: Percent = Percent::from_percent(percent);
		let offender_bytes: Vec<_> = offender.into();
		let offender: Runtime::AccountId = Decode::decode(&mut &offender_bytes[..])
			.map_err(|_| revert("Invalid offender account id"))?;

		let call = pallet_services::Call::<Runtime>::slash {
			offender,

			service_id,
			slash_percent: percent,
		};
		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(())
	}

	/// Dispute an Unapplied Slash for a service id.
	///
	/// The caller needs to be an authorized Dispute Origin for this service.
	#[precompile::public("dispute(uint32,uint32)")]
	fn dispute(handle: &mut impl PrecompileHandle, era: u32, index: u32) -> EvmResult {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let caller = handle.context().caller;
		let origin = Runtime::AddressMapping::into_account_id(caller);

		// inside this call, we do check if the caller is authorized to dispute the slash
		let call = pallet_services::Call::<Runtime>::dispute { era, index };
		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(())
	}
}

/// Revert with Custom Error Selector
fn revert_custom_error(err: [u8; 32]) -> PrecompileFailure {
	let selector = &err[0..4];
	let mut output = sp_std::vec![0u8; 32];
	output[0..4].copy_from_slice(selector);
	PrecompileFailure::Revert { exit_status: fp_evm::ExitRevert::Reverted, output }
}
