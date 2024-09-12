#![cfg_attr(not(feature = "std"), no_std)]

use fp_evm::PrecompileHandle;
use frame_support::dispatch::{DispatchResult, GetDispatchInfo, PostDispatchInfo};
use pallet_evm::AddressMapping;
use pallet_services::{Field, OperatorPreferences};
use parity_scale_codec::Decode;
use precompile_utils::prelude::*;
use sp_core::{H160, U256};
use sp_runtime::traits::Dispatchable;
use sp_std::{marker::PhantomData, vec::Vec};
use tangle_primitives::services::ServiceBlueprint;

/// Precompile for the `ServiceBlueprint` pallet.
pub struct ServiceBlueprintPrecompile<Runtime>(PhantomData<Runtime>);

#[precompile_utils::precompile]
impl<Runtime> ServiceBlueprintPrecompile<Runtime>
where
	Runtime: pallet_services::Config + pallet_evm::Config,
	Runtime::RuntimeCall: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	<Runtime::RuntimeCall as Dispatchable>::RuntimeOrigin: From<Option<Runtime::AccountId>>,
	Runtime::RuntimeCall: From<pallet_services::Call<Runtime>>,
{
	/// Create a new blueprint.
	#[precompile::public("createBlueprint(bytes32)")]
	fn create_blueprint(handle: &mut impl PrecompileHandle, blueprint_data: Vec<u8>) -> EvmResult {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);

		let blueprint: ServiceBlueprint<Runtime::Constraints> =
			Decode::decode(&mut &blueprint_data[..])
				.map_err(|_| revert("Invalid blueprint data"))?;

		let call = pallet_services::Call::<Runtime>::create_blueprint { blueprint };

		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(())
	}

	/// Register as an operator for a specific blueprint.
	#[precompile::public("registerOperator(uint256,bytes)")]
	fn register_operator(
		handle: &mut impl PrecompileHandle,
		blueprint_id: U256,
		preferences: Vec<u8>,
		registration_args: Vec<u8>,
	) -> EvmResult {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);

		let blueprint_id: u64 = blueprint_id.as_u64();
		let preferences: OperatorPreferences = Decode::decode(&mut &preferences[..])
			.map_err(|_| revert("Invalid preferences data"))?;

		let registration_args: Vec<Field<Runtime::Constraints, Runtime::AccountId>> =
			Decode::decode(&mut &registration_args[..])
				.map_err(|_| revert("Invalid registration arguments"))?;

		let call = pallet_services::Call::<Runtime>::register {
			blueprint_id,
			preferences,
			registration_args,
		};

		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(())
	}

	/// Unregister as an operator from a blueprint.
	#[precompile::public("unregisterOperator(uint256)")]
	fn unregister_operator(handle: &mut impl PrecompileHandle, blueprint_id: U256) -> EvmResult {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);

		let blueprint_id: u64 = blueprint_id.as_u64();

		let call = pallet_services::Call::<Runtime>::unregister { blueprint_id };

		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(())
	}

	/// Request a new service.
	#[precompile::public("requestService(uint256,bytes,bytes,bytes)")]
	fn request_service(
		handle: &mut impl PrecompileHandle,
		blueprint_id: U256,
		permitted_callers_data: Vec<u8>,
		service_providers_data: Vec<u8>,
		request_args_data: Vec<u8>,
	) -> EvmResult {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);

		let blueprint_id: u64 = blueprint_id.as_u64();
		let permitted_callers: Vec<Runtime::AccountId> =
			Decode::decode(&mut &permitted_callers_data[..])
				.map_err(|_| revert("Invalid permitted callers data"))?;

		let service_providers: Vec<Runtime::AccountId> =
			Decode::decode(&mut &service_providers_data[..])
				.map_err(|_| revert("Invalid service providers data"))?;

		let request_args: Vec<Field<Runtime::Constraints, Runtime::AccountId>> =
			Decode::decode(&mut &request_args_data[..])
				.map_err(|_| revert("Invalid request arguments data"))?;

		let call = pallet_services::Call::<Runtime>::request {
			blueprint_id,
			permitted_callers,
			service_providers,
			ttl: 10000_u32.into(),
			request_args,
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
}
