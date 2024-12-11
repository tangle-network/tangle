#[cfg(not(feature = "std"))]
use alloc::{boxed::Box, string::String, vec, vec::Vec};

#[cfg(feature = "std")]
use std::{boxed::Box, string::String, vec::Vec};

use ethabi::{Function, StateMutability, Token};
use frame_support::dispatch::{DispatchErrorWithPostInfo, PostDispatchInfo};
use sp_core::{H160, U256};
use sp_runtime::traits::{UniqueSaturatedInto, Zero};
use tangle_primitives::services::{
	Asset, BlueprintServiceManager, Field, MasterBlueprintServiceManagerRevision,
	OperatorPreferences, Service, ServiceBlueprint,
};

use super::*;
use crate::types::BalanceOf;

#[allow(clippy::too_many_arguments)]
impl<T: Config> Pallet<T> {
	/// Returns the account id of the pallet.
	///
	/// This function retrieves the account id associated with the pallet by converting
	/// the pallet evm address to an account id.
	///
	/// # Returns
	/// * `T::AccountId` - The account id of the pallet.
	pub fn account_id() -> T::AccountId {
		T::EvmAddressMapping::into_account_id(Self::address())
	}

	/// Returns the EVM address of the pallet.
	///
	/// # Returns
	/// * `H160` - The address of the pallet.
	pub fn address() -> H160 {
		T::PalletEVMAddress::get()
	}

	/// Get the address of the master blueprint service manager at a given revision.
	///
	/// # Parameters
	/// * `revision` - The revision of the master blueprint service manager.
	///
	/// # Returns
	/// * `Result<H160, Error<T>>` - The address of the master blueprint service manager.
	/// * `Error<T>` - The error type.
	#[doc(alias = "get_master_blueprint_service_manager_address")]
	#[inline(always)]
	pub fn mbsm_address(revision: u32) -> Result<H160, Error<T>> {
		MasterBlueprintServiceManagerRevisions::<T>::get()
			.get(revision as usize)
			.cloned()
			.ok_or(Error::<T>::MasterBlueprintServiceManagerRevisionNotFound)
	}

	/// Get the latest revision of the master blueprint service manager.
	///
	/// # Returns
	/// * `u32` - The latest revision of the master blueprint service manager.
	#[doc(alias = "get_master_blueprint_service_manager_revision")]
	#[inline(always)]
	pub fn mbsm_latest_revision() -> u32 {
		MasterBlueprintServiceManagerRevisions::<T>::decode_len()
			.map(|len| len.saturating_sub(1) as u32)
			.unwrap_or(0)
	}

	/// Get the address of the master blueprint service manager for the given blueprint.
	///
	/// # Parameters
	/// * `blueprint` - The service blueprint.
	///
	/// # Returns
	/// * `Result<H160, Error<T>>` - The address of the master blueprint service manager.
	/// * `Error<T>` - The error type.
	pub fn mbsm_address_of(blueprint: &ServiceBlueprint<T::Constraints>) -> Result<H160, Error<T>> {
		match blueprint.master_manager_revision {
			MasterBlueprintServiceManagerRevision::Specific(rev) => Self::mbsm_address(rev),
			MasterBlueprintServiceManagerRevision::Latest =>
				Self::mbsm_address(Self::mbsm_latest_revision()),
			other => unimplemented!("Got unexpected case for {:?}", other),
		}
	}

	/// Hook to be called when a blueprint is created.
	///
	/// This function is called when a blueprint is created. It performs an EVM call
	/// to the `onBlueprintCreated` function of the service blueprint's manager contract.
	///
	/// # Parameters
	/// * `blueprint` - The service blueprint.
	/// * `blueprint_id` - The blueprint ID.
	/// * `owner` - The owner of the blueprint.
	///
	/// # Returns
	/// * `Result<(bool, Weight), DispatchErrorWithPostInfo>` - A tuple containing a boolean
	///   indicating whether the blueprint creation is allowed and the weight of the operation.
	pub fn on_blueprint_created_hook(
		blueprint: &ServiceBlueprint<T::Constraints>,
		blueprint_id: u64,
		owner: &T::AccountId,
	) -> Result<(bool, Weight), DispatchErrorWithPostInfo> {
		// To the BSM
		#[allow(deprecated)]
		let (allowed0, weight0) = match blueprint.manager {
			BlueprintServiceManager::Evm(bsm) => {
				let mbsm = Self::mbsm_address_of(blueprint)?;
				let f = Function {
					name: String::from("onBlueprintCreated"),
					inputs: vec![
						ethabi::Param {
							name: String::from("blueprintId"),
							kind: ethabi::ParamType::Uint(64),
							internal_type: None,
						},
						ethabi::Param {
							name: String::from("owner"),
							kind: ethabi::ParamType::Address,
							internal_type: None,
						},
						ethabi::Param {
							name: String::from("mbsm"),
							kind: ethabi::ParamType::Address,
							internal_type: None,
						},
					],
					outputs: Default::default(),
					constant: None,
					state_mutability: StateMutability::Payable,
				};
				let args = &[
					Token::Uint(ethabi::Uint::from(blueprint_id)),
					Token::Address(T::EvmAddressMapping::into_address(owner.clone())),
					Token::Address(mbsm),
				];
				let data = f.encode_input(args).map_err(|_| Error::<T>::EVMAbiEncode)?;
				let gas_limit = 300_000;
				let value = U256::zero();
				let info = Self::evm_call(Self::address(), bsm, value, data, gas_limit)?;
				let weight = Self::weight_from_call_info(&info);
				(info.exit_reason.is_succeed(), weight)
			},
			_ => unimplemented!("Got unexpected case for {:?}", blueprint.manager),
		};
		// To the MBSM
		#[allow(deprecated)]
		let (allowed1, weight1) = Self::dispatch_hook(
			blueprint,
			Function {
				name: String::from("onBlueprintCreated"),
				inputs: vec![
					ethabi::Param {
						name: String::from("blueprintId"),
						kind: ethabi::ParamType::Uint(64),
						internal_type: None,
					},
					ethabi::Param {
						name: String::from("owner"),
						kind: ethabi::ParamType::Address,
						internal_type: None,
					},
					ServiceBlueprint::<T::Constraints>::to_ethabi_param(),
				],
				outputs: Default::default(),
				constant: None,
				state_mutability: StateMutability::Payable,
			},
			&[
				Token::Uint(ethabi::Uint::from(blueprint_id)),
				Token::Address(T::EvmAddressMapping::into_address(owner.clone())),
				blueprint.to_ethabi(),
			],
			Zero::zero(),
		)?;
		Ok((allowed0 && allowed1, weight0.saturating_add(weight1)))
	}

	/// Hook to be called upon a new operator registration on a blueprint.
	///
	/// This function is called when a service is registered. It performs an EVM call
	/// to the `onRegister` function of the service blueprint's manager contract.
	///
	/// # Parameters
	/// * `blueprint` - The service blueprint.
	/// * `blueprint_id` - The blueprint ID.
	/// * `prefrences` - The operator preferences.
	/// * `registration_args` - The registration arguments.
	/// * `value` - The value to be sent with the call.
	///
	/// # Returns
	/// * `Result<(bool, Weight), DispatchErrorWithPostInfo>` - A tuple containing a boolean
	///   indicating whether the registration is allowed and the weight of the operation.
	pub fn on_register_hook(
		blueprint: &ServiceBlueprint<T::Constraints>,
		blueprint_id: u64,
		prefrences: &OperatorPreferences,
		registration_args: &[Field<T::Constraints, T::AccountId>],
		value: BalanceOf<T>,
	) -> Result<(bool, Weight), DispatchErrorWithPostInfo> {
		#[allow(deprecated)]
		Self::dispatch_hook(
			blueprint,
			Function {
				name: String::from("onRegister"),
				inputs: vec![
					ethabi::Param {
						name: String::from("blueprintId"),
						kind: ethabi::ParamType::Uint(64),
						internal_type: None,
					},
					OperatorPreferences::to_ethabi_param(),
					ethabi::Param {
						name: String::from("registrationInputs"),
						kind: ethabi::ParamType::Bytes,
						internal_type: None,
					},
				],
				outputs: Default::default(),
				constant: None,
				state_mutability: StateMutability::Payable,
			},
			&[
				Token::Uint(ethabi::Uint::from(blueprint_id)),
				prefrences.to_ethabi(),
				Token::Bytes(Field::encode_to_ethabi(registration_args)),
			],
			value,
		)
	}

	/// Hook to be called upon an operator unregistration on a blueprint.
	///
	/// This function is called when an operator is unregistered. It performs an EVM call
	/// to the `onUnregister` function of the service blueprint's manager contract.
	///
	/// # Parameters
	/// * `blueprint` - The service blueprint.
	/// * `blueprint_id` - The blueprint ID.
	/// * `prefrences` - The operator preferences.
	///
	/// # Returns
	/// * `Result<(bool, Weight), DispatchErrorWithPostInfo>` - A tuple containing a boolean
	///   indicating
	///  whether the unregistration is allowed and the weight of the operation.
	pub fn on_unregister_hook(
		blueprint: &ServiceBlueprint<T::Constraints>,
		blueprint_id: u64,
		prefrences: &OperatorPreferences,
	) -> Result<(bool, Weight), DispatchErrorWithPostInfo> {
		#[allow(deprecated)]
		Self::dispatch_hook(
			blueprint,
			Function {
				name: String::from("onUnregister"),
				inputs: vec![
					ethabi::Param {
						name: String::from("blueprintId"),
						kind: ethabi::ParamType::Uint(64),
						internal_type: None,
					},
					OperatorPreferences::to_ethabi_param(),
				],
				outputs: Default::default(),
				constant: None,
				state_mutability: StateMutability::NonPayable,
			},
			&[Token::Uint(ethabi::Uint::from(blueprint_id)), prefrences.to_ethabi()],
			Zero::zero(),
		)
	}

	/// Hook to be called upon a new price targets update on a blueprint.
	/// This function is called when the price targets are updated. It performs an EVM call
	/// to the `onUpdatePriceTargets` function of the service blueprint's manager contract.
	///
	/// # Parameters
	/// * `blueprint` - The service blueprint.
	/// * `blueprint_id` - The blueprint ID.
	/// * `prefrences` - The operator preferences.
	///
	/// # Returns
	///
	/// * `Result<(bool, Weight), DispatchErrorWithPostInfo>` - A tuple containing a boolean
	///   indicating
	///  whether the price targets update is allowed and the weight of the operation.
	pub fn on_update_price_targets(
		blueprint: &ServiceBlueprint<T::Constraints>,
		blueprint_id: u64,
		prefrences: &OperatorPreferences,
	) -> Result<(bool, Weight), DispatchErrorWithPostInfo> {
		#[allow(deprecated)]
		Self::dispatch_hook(
			blueprint,
			Function {
				name: String::from("onUpdatePriceTargets"),
				inputs: vec![
					ethabi::Param {
						name: String::from("blueprintId"),
						kind: ethabi::ParamType::Uint(64),
						internal_type: None,
					},
					OperatorPreferences::to_ethabi_param(),
				],
				outputs: Default::default(),
				constant: None,
				state_mutability: StateMutability::Payable,
			},
			&[Token::Uint(ethabi::Uint::from(blueprint_id)), prefrences.to_ethabi()],
			Zero::zero(),
		)
	}

	/// Hook to be called upon an operator approve a service request on a blueprint.
	///
	/// This function is called when an operator approve a service request. It performs an EVM call
	/// to the `onApprove` function of the service blueprint's manager contract.
	///
	/// # Parameters
	/// * `blueprint` - The service blueprint.
	/// * `blueprint_id` - The blueprint ID.
	/// * `prefrences` - The operator preferences.
	/// * `request_id` - The request id.
	/// * `restaking_percent` - The restaking percent.
	///
	/// # Returns
	/// * `Result<(bool, Weight), DispatchErrorWithPostInfo>` - A tuple containing a boolean
	///   indicating
	/// whether the approve is allowed and the weight of the operation.
	pub fn on_approve_hook(
		blueprint: &ServiceBlueprint<T::Constraints>,
		blueprint_id: u64,
		prefrences: &OperatorPreferences,
		request_id: u64,
		restaking_percent: u8,
	) -> Result<(bool, Weight), DispatchErrorWithPostInfo> {
		#[allow(deprecated)]
		Self::dispatch_hook(
			blueprint,
			Function {
				name: String::from("onApprove"),
				inputs: vec![
					ethabi::Param {
						name: String::from("blueprintId"),
						kind: ethabi::ParamType::Uint(64),
						internal_type: None,
					},
					OperatorPreferences::to_ethabi_param(),
					ethabi::Param {
						name: String::from("requestId"),
						kind: ethabi::ParamType::Uint(64),
						internal_type: None,
					},
					ethabi::Param {
						name: String::from("restakingPercent"),
						kind: ethabi::ParamType::Uint(8),
						internal_type: None,
					},
				],
				outputs: Default::default(),
				constant: None,
				state_mutability: StateMutability::NonPayable,
			},
			&[
				Token::Uint(ethabi::Uint::from(blueprint_id)),
				prefrences.to_ethabi(),
				Token::Uint(ethabi::Uint::from(request_id)),
				Token::Uint(ethabi::Uint::from(restaking_percent)),
			],
			Zero::zero(),
		)
	}

	/// Hook to be called upon an operator reject a service request on a blueprint.
	/// This function is called when an operator reject a service request. It performs an EVM call
	/// to the `onReject` function of the service blueprint's manager contract.
	///
	/// # Parameters
	/// * `blueprint` - The service blueprint.
	/// * `blueprint_id` - The blueprint ID.
	/// * `prefrences` - The operator preferences.
	/// * `request_id` - The request id.
	///
	/// # Returns
	/// * `Result<(bool, Weight), DispatchErrorWithPostInfo>` - A tuple containing a boolean
	///   indicating
	/// whether the reject is allowed and the weight of the operation.
	pub fn on_reject_hook(
		blueprint: &ServiceBlueprint<T::Constraints>,
		blueprint_id: u64,
		prefrences: &OperatorPreferences,
		request_id: u64,
	) -> Result<(bool, Weight), DispatchErrorWithPostInfo> {
		#[allow(deprecated)]
		Self::dispatch_hook(
			blueprint,
			Function {
				name: String::from("onReject"),
				inputs: vec![
					ethabi::Param {
						name: String::from("blueprintId"),
						kind: ethabi::ParamType::Uint(64),
						internal_type: None,
					},
					OperatorPreferences::to_ethabi_param(),
					ethabi::Param {
						name: String::from("requestId"),
						kind: ethabi::ParamType::Uint(64),
						internal_type: None,
					},
				],
				outputs: Default::default(),
				constant: None,
				state_mutability: StateMutability::NonPayable,
			},
			&[
				Token::Uint(ethabi::Uint::from(blueprint_id)),
				prefrences.to_ethabi(),
				Token::Uint(ethabi::Uint::from(request_id)),
			],
			Zero::zero(),
		)
	}

	/// Hook to be called upon new service request.
	///
	/// This function is called when a service request is made. It performs an EVM call
	/// to the `onRequest` function of the service blueprint's manager contract.
	///
	/// # Parameters
	/// * `blueprint` - The service blueprint.
	/// * `blueprint_id` - The blueprint ID.
	/// * `requester` - The requester of the service.
	/// * `request_id` - The service request ID.
	/// * `operators` - The operator preferences.
	/// * `request_args` - The request arguments.
	/// * `permitted_callers` - The permitted callers.
	/// * `assets` - The assets to be used.
	/// * `ttl` - The time to live.
	/// * `value` - The value to be sent with the call.
	///
	/// # Returns
	/// * `Result<(bool, Weight), DispatchErrorWithPostInfo>` - A tuple containing a boolean
	///   indicating whether the request is allowed and the weight of the operation.
	pub fn on_request_hook(
		blueprint: &ServiceBlueprint<T::Constraints>,
		blueprint_id: u64,
		requester: &T::AccountId,
		request_id: u64,
		operators: &[OperatorPreferences],
		request_args: &[Field<T::Constraints, T::AccountId>],
		permitted_callers: &[T::AccountId],
		_assets: &[T::AssetId],
		ttl: BlockNumberFor<T>,
		paymet_asset: Asset<T::AssetId>,
		value: BalanceOf<T>,
		native_value: BalanceOf<T>,
	) -> Result<(bool, Weight), DispatchErrorWithPostInfo> {
		#[allow(deprecated)]
		Self::dispatch_hook(
			blueprint,
			Function {
				name: String::from("onRequest"),
				inputs: vec![
					ethabi::Param {
						name: String::from("blueprintId"),
						kind: ethabi::ParamType::Uint(64),
						internal_type: None,
					},
					ethabi::Param {
						name: String::from("params"),
						kind: ethabi::ParamType::Tuple(vec![
							// requestId
							ethabi::ParamType::Uint(64),
							// requester
							ethabi::ParamType::Address,
							// operatorsWithPreferences
							ethabi::ParamType::Array(Box::new(
								OperatorPreferences::to_ethabi_param_type(),
							)),
							// requestInputs
							ethabi::ParamType::Bytes,
							// permittedCallers
							ethabi::ParamType::Array(Box::new(ethabi::ParamType::Address)),
							// ttl
							ethabi::ParamType::Uint(64),
							// payment asset
							Asset::<T::AssetId>::to_ethabi_param_type(),
							// value
							ethabi::ParamType::Uint(256),
						]),
						internal_type: Some(String::from("struct ServiceOperators.RequestParams")),
					},
				],
				outputs: Default::default(),
				constant: None,
				state_mutability: StateMutability::Payable,
			},
			&[
				Token::Uint(ethabi::Uint::from(blueprint_id)),
				Token::Tuple(vec![
					Token::Uint(ethabi::Uint::from(request_id)),
					Token::Address(T::EvmAddressMapping::into_address(requester.clone())),
					Token::Array(operators.iter().map(OperatorPreferences::to_ethabi).collect()),
					Token::Bytes(Field::encode_to_ethabi(request_args)),
					Token::Array(
						permitted_callers
							.iter()
							.map(|caller| {
								Token::Address(T::EvmAddressMapping::into_address(caller.clone()))
									.clone()
							})
							.collect(),
					),
					// Token::Array(vec![]),
					Token::Uint(ethabi::Uint::from(ttl.into())),
					paymet_asset.to_ethabi(),
					Token::Uint(ethabi::Uint::from(value.using_encoded(U256::from_little_endian))),
				]),
			],
			native_value,
		)
	}

	/// Hook to be called when a service is initialized. This function will call the
	/// `onServiceInitialized` function of the service blueprint manager contract.
	///
	/// # Arguments
	/// * `blueprint` - The service blueprint.
	/// * `blueprint_id` - The blueprint ID.
	/// * `request_id` - The service request ID.
	/// * `service_id` - The service ID.
	/// * `owner` - The owner of the service.
	/// * `permitted_callers` - The permitted callers.
	/// * `assets` - The assets to be used.
	/// * `ttl` - The time to live.
	///
	/// # Returns
	/// * `Result<(bool, Weight), DispatchErrorWithPostInfo>` - A tuple containing a boolean
	///   indicating
	///  whether the request is allowed and the weight of the operation.
	pub fn on_service_init_hook(
		blueprint: &ServiceBlueprint<T::Constraints>,
		blueprint_id: u64,
		request_id: u64,
		service_id: u64,
		owner: &T::AccountId,
		permitted_callers: &[T::AccountId],
		_assets: &[T::AssetId],
		ttl: BlockNumberFor<T>,
	) -> Result<(bool, Weight), DispatchErrorWithPostInfo> {
		#[allow(deprecated)]
		Self::dispatch_hook(
			blueprint,
			Function {
				name: String::from("onServiceInitialized"),
				inputs: vec![
					ethabi::Param {
						name: String::from("blueprintId"),
						kind: ethabi::ParamType::Uint(64),
						internal_type: None,
					},
					ethabi::Param {
						name: String::from("requestId"),
						kind: ethabi::ParamType::Uint(64),
						internal_type: None,
					},
					ethabi::Param {
						name: String::from("serviceId"),
						kind: ethabi::ParamType::Uint(64),
						internal_type: None,
					},
					ethabi::Param {
						name: String::from("owner"),
						kind: ethabi::ParamType::Address,
						internal_type: None,
					},
					ethabi::Param {
						name: String::from("permittedCallers"),
						kind: ethabi::ParamType::Array(Box::new(ethabi::ParamType::Address)),
						internal_type: Some(String::from("address[]")),
					},
					// ethabi::Param {
					// 	name: String::from("assets"),
					// 	kind: ethabi::ParamType::Array(Box::new(ethabi::ParamType::Address)),
					// 	internal_type: Some(String::from("address[]")),
					// },
					ethabi::Param {
						name: String::from("ttl"),
						kind: ethabi::ParamType::Uint(64),
						internal_type: None,
					},
				],
				outputs: Default::default(),
				constant: None,
				state_mutability: StateMutability::Payable,
			},
			&[
				Token::Uint(ethabi::Uint::from(blueprint_id)),
				Token::Uint(ethabi::Uint::from(request_id)),
				Token::Uint(ethabi::Uint::from(service_id)),
				Token::Address(T::EvmAddressMapping::into_address(owner.clone())),
				Token::Array(
					permitted_callers
						.iter()
						.map(|caller| {
							Token::Address(T::EvmAddressMapping::into_address(caller.clone()))
						})
						.collect(),
				),
				// Token::Array(vec![]),
				Token::Uint(ethabi::Uint::from(ttl.into())),
			],
			Zero::zero(),
		)
	}

	/// Hook to be called when a service is terminated. This function will call the
	/// `onServiceTermination` function of the service blueprint manager contract.
	///
	/// # Arguments
	/// * `blueprint` - The service blueprint.
	/// * `blueprint_id` - The blueprint ID.
	/// * `service_id` - The service ID.
	/// * `owner` - The owner of the service.
	///
	/// # Returns
	/// * `Result<(bool, Weight), DispatchErrorWithPostInfo>` - A tuple containing a boolean
	///   indicating
	/// whether the request is allowed and the weight of the operation.
	pub fn on_service_termination_hook(
		blueprint: &ServiceBlueprint<T::Constraints>,
		blueprint_id: u64,
		service_id: u64,
		owner: &T::AccountId,
	) -> Result<(bool, Weight), DispatchErrorWithPostInfo> {
		#[allow(deprecated)]
		Self::dispatch_hook(
			blueprint,
			Function {
				name: String::from("onServiceTermination"),
				inputs: vec![
					ethabi::Param {
						name: String::from("blueprintId"),
						kind: ethabi::ParamType::Uint(64),
						internal_type: None,
					},
					ethabi::Param {
						name: String::from("serviceId"),
						kind: ethabi::ParamType::Uint(64),
						internal_type: None,
					},
					ethabi::Param {
						name: String::from("owner"),
						kind: ethabi::ParamType::Address,
						internal_type: None,
					},
				],
				outputs: Default::default(),
				constant: None,
				state_mutability: StateMutability::NonPayable,
			},
			&[
				Token::Uint(ethabi::Uint::from(blueprint_id)),
				Token::Uint(ethabi::Uint::from(service_id)),
				Token::Address(T::EvmAddressMapping::into_address(owner.clone())),
			],
			Zero::zero(),
		)
	}

	/// Hook to be called upon job call.
	///
	/// This function is called when a job call is made. It performs an EVM call
	/// to the `onJobCall` function of the service blueprint's manager contract.
	///
	/// # Parameters
	/// * `blueprint` - The service blueprint.
	/// * `blueprint_id` - The blueprint ID.
	/// * `service_id` - The service ID.
	/// * `job` - The job index.
	/// * `job_call_id` - The job call ID.
	/// * `inputs` - The input fields.
	///
	/// # Returns
	/// * `Result<(bool, Weight), DispatchErrorWithPostInfo>` - A tuple containing a boolean
	///   indicating whether the job call is allowed and the weight of the operation.
	pub fn on_job_call_hook(
		blueprint: &ServiceBlueprint<T::Constraints>,
		blueprint_id: u64,
		service_id: u64,
		job: u8,
		job_call_id: u64,
		inputs: &[Field<T::Constraints, T::AccountId>],
	) -> Result<(bool, Weight), DispatchErrorWithPostInfo> {
		#[allow(deprecated)]
		Self::dispatch_hook(
			blueprint,
			Function {
				name: String::from("onJobCall"),
				inputs: vec![
					ethabi::Param {
						name: String::from("blueprintId"),
						kind: ethabi::ParamType::Uint(64),
						internal_type: None,
					},
					ethabi::Param {
						name: String::from("serviceId"),
						kind: ethabi::ParamType::Uint(64),
						internal_type: None,
					},
					ethabi::Param {
						name: String::from("job"),
						kind: ethabi::ParamType::Uint(8),
						internal_type: None,
					},
					ethabi::Param {
						name: String::from("jobCallId"),
						kind: ethabi::ParamType::Uint(64),
						internal_type: None,
					},
					ethabi::Param {
						name: String::from("inputs"),
						kind: ethabi::ParamType::Bytes,
						internal_type: None,
					},
				],
				outputs: Default::default(),
				constant: None,
				state_mutability: StateMutability::Payable,
			},
			&[
				Token::Uint(ethabi::Uint::from(blueprint_id)),
				Token::Uint(ethabi::Uint::from(service_id)),
				Token::Uint(ethabi::Uint::from(job)),
				Token::Uint(ethabi::Uint::from(job_call_id)),
				Token::Bytes(Field::encode_to_ethabi(inputs)),
			],
			Zero::zero(),
		)
	}

	/// Hook to be called upon job result.
	///
	/// This function is called when a job result is submitted. It performs an EVM call
	/// to the `onJobResult` function of the service blueprint's manager contract.
	///
	/// # Parameters
	/// * `blueprint` - The service blueprint.
	/// * `bluepint_id` - The blueprint ID.
	/// * `service_id` - The service ID.
	/// * `job` - The job index.
	/// * `job_call_id` - The job call ID.
	/// * `prefrences` - The operator preferences.
	/// * `inputs` - The input fields.
	/// * `outputs` - The output fields.
	///
	/// # Returns
	/// * `Result<(bool, Weight), DispatchErrorWithPostInfo>` - A tuple containing a boolean
	///   indicating whether the job result is allowed and the weight of the operation.
	pub fn on_job_result_hook(
		blueprint: &ServiceBlueprint<T::Constraints>,
		blueprint_id: u64,
		service_id: u64,
		job: u8,
		job_call_id: u64,
		prefrences: &OperatorPreferences,
		inputs: &[Field<T::Constraints, T::AccountId>],
		outputs: &[Field<T::Constraints, T::AccountId>],
	) -> Result<(bool, Weight), DispatchErrorWithPostInfo> {
		#[allow(deprecated)]
		Self::dispatch_hook(
			blueprint,
			Function {
				name: String::from("onJobResult"),
				inputs: vec![
					ethabi::Param {
						name: String::from("blueprintId"),
						kind: ethabi::ParamType::Uint(64),
						internal_type: None,
					},
					ethabi::Param {
						name: String::from("serviceId"),
						kind: ethabi::ParamType::Uint(64),
						internal_type: None,
					},
					ethabi::Param {
						name: String::from("jobIndex"),
						kind: ethabi::ParamType::Uint(8),
						internal_type: None,
					},
					ethabi::Param {
						name: String::from("jobCallId"),
						kind: ethabi::ParamType::Uint(64),
						internal_type: None,
					},
					OperatorPreferences::to_ethabi_param(),
					ethabi::Param {
						name: String::from("inputs"),
						kind: ethabi::ParamType::Bytes,
						internal_type: None,
					},
					ethabi::Param {
						name: String::from("outputs"),
						kind: ethabi::ParamType::Bytes,
						internal_type: None,
					},
				],
				outputs: Default::default(),
				constant: None,
				state_mutability: StateMutability::NonPayable,
			},
			&[
				Token::Uint(ethabi::Uint::from(blueprint_id)),
				Token::Uint(ethabi::Uint::from(service_id)),
				Token::Uint(ethabi::Uint::from(job)),
				Token::Uint(ethabi::Uint::from(job_call_id)),
				prefrences.to_ethabi(),
				Token::Bytes(Field::encode_to_ethabi(inputs)),
				Token::Bytes(Field::encode_to_ethabi(outputs)),
			],
			Zero::zero(),
		)
	}

	/// Queries the slashing origin of a service.
	///
	/// This function performs an EVM call to the `querySlashingOrigin` function of the
	/// service blueprint's manager contract to retrieve the slashing origin.
	///
	/// # Parameters
	/// * `service` - The service.
	///
	/// # Returns
	/// * `Result<(Option<T::AccountId>, Weight), DispatchErrorWithPostInfo>` - A tuple containing
	///   the slashing origin account id (if any) and the weight of the operation.
	pub fn query_slashing_origin(
		service: &Service<T::Constraints, T::AccountId, BlockNumberFor<T>, T::AssetId>,
	) -> Result<(Option<T::AccountId>, Weight), DispatchErrorWithPostInfo> {
		let (_, blueprint) = Self::blueprints(service.blueprint)?;
		#[allow(deprecated)]
		let query = Function {
			name: String::from("querySlashingOrigin"),
			inputs: vec![
				ethabi::Param {
					name: String::from("blueprintId"),
					kind: ethabi::ParamType::Uint(64),
					internal_type: None,
				},
				ethabi::Param {
					name: String::from("serviceId"),
					kind: ethabi::ParamType::Uint(64),
					internal_type: None,
				},
			],
			outputs: vec![ethabi::Param {
				name: String::from("slashingOrigin"),
				kind: ethabi::ParamType::Address,
				internal_type: None,
			}],
			constant: None,
			state_mutability: StateMutability::NonPayable,
		};
		let mbsm = Self::mbsm_address_of(&blueprint)?;
		let (info, weight) = Self::dispatch_evm_call(
			mbsm,
			query.clone(),
			&[
				Token::Uint(ethabi::Uint::from(service.blueprint)),
				Token::Uint(ethabi::Uint::from(service.id)),
			],
			Zero::zero(),
		)?;

		// decode the result and return it
		let maybe_value = info.exit_reason.is_succeed().then_some(&info.value);
		let slashing_origin = if let Some(data) = maybe_value {
			let result = query.decode_output(data).map_err(|_| Error::<T>::EVMAbiDecode)?;
			let slashing_origin = result.first().ok_or_else(|| Error::<T>::EVMAbiDecode)?;
			if let ethabi::Token::Address(who) = slashing_origin {
				Some(T::EvmAddressMapping::into_account_id(*who))
			} else {
				None
			}
		} else {
			None
		};

		Ok((slashing_origin, weight))
	}

	/// Queries the dispute origin of a service.
	///
	/// This function performs an EVM call to the `queryDisputeOrigin` function of the
	/// service blueprint's manager contract to retrieve the dispute origin.
	///
	/// # Parameters
	/// * `service` - The service.
	///
	/// # Returns
	/// * `Result<(Option<T::AccountId>, Weight), DispatchErrorWithPostInfo>` - A tuple containing
	///   the dispute origin account id (if any) and the weight of the operation.
	pub fn query_dispute_origin(
		service: &Service<T::Constraints, T::AccountId, BlockNumberFor<T>, T::AssetId>,
	) -> Result<(Option<T::AccountId>, Weight), DispatchErrorWithPostInfo> {
		let (_, blueprint) = Self::blueprints(service.blueprint)?;
		#[allow(deprecated)]
		let query = Function {
			name: String::from("queryDisputeOrigin"),
			inputs: vec![
				ethabi::Param {
					name: String::from("blueprintId"),
					kind: ethabi::ParamType::Uint(64),
					internal_type: None,
				},
				ethabi::Param {
					name: String::from("serviceId"),
					kind: ethabi::ParamType::Uint(64),
					internal_type: None,
				},
			],
			outputs: vec![ethabi::Param {
				name: String::from("disputeOrigin"),
				kind: ethabi::ParamType::Address,
				internal_type: None,
			}],
			constant: None,
			state_mutability: StateMutability::NonPayable,
		};
		let mbsm = Self::mbsm_address_of(&blueprint)?;
		let (info, weight) = Self::dispatch_evm_call(
			mbsm,
			query.clone(),
			&[
				Token::Uint(ethabi::Uint::from(service.blueprint)),
				Token::Uint(ethabi::Uint::from(service.id)),
			],
			Zero::zero(),
		)?;

		// decode the result and return it
		let maybe_value = info.exit_reason.is_succeed().then_some(&info.value);
		let dispute_origin = if let Some(data) = maybe_value {
			let result = query.decode_output(data).map_err(|_| Error::<T>::EVMAbiDecode)?;
			let slashing_origin = result.first().ok_or_else(|| Error::<T>::EVMAbiDecode)?;
			if let ethabi::Token::Address(who) = slashing_origin {
				Some(T::EvmAddressMapping::into_account_id(*who))
			} else {
				None
			}
		} else {
			None
		};

		Ok((dispute_origin, weight))
	}

	/// Moves a `value` amount of tokens from the caller's account to `to`.
	pub fn erc20_transfer(
		erc20: H160,
		caller: &T::AccountId,
		to: H160,
		value: BalanceOf<T>,
	) -> Result<(bool, Weight), DispatchErrorWithPostInfo> {
		let from = T::EvmAddressMapping::into_address(caller.clone());
		#[allow(deprecated)]
		let transfer_fn = Function {
			name: String::from("transfer"),
			inputs: vec![
				ethabi::Param {
					name: String::from("to"),
					kind: ethabi::ParamType::Address,
					internal_type: None,
				},
				ethabi::Param {
					name: String::from("value"),
					kind: ethabi::ParamType::Uint(256),
					internal_type: None,
				},
			],
			outputs: vec![ethabi::Param {
				name: String::from("success"),
				kind: ethabi::ParamType::Bool,
				internal_type: None,
			}],
			constant: None,
			state_mutability: StateMutability::NonPayable,
		};

		let args = [
			Token::Address(to),
			Token::Uint(ethabi::Uint::from(value.using_encoded(U256::from_little_endian))),
		];

		log::debug!(target: "evm", "Dispatching EVM call(0x{}): {}", hex::encode(transfer_fn.short_signature()), transfer_fn.signature());
		let data = transfer_fn.encode_input(&args).map_err(|_| Error::<T>::EVMAbiEncode)?;
		let gas_limit = 300_000;
		let info = Self::evm_call(from, erc20, U256::zero(), data, gas_limit)?;
		let weight = Self::weight_from_call_info(&info);

		// decode the result and return it
		let maybe_value = info.exit_reason.is_succeed().then_some(&info.value);
		let success = if let Some(data) = maybe_value {
			let result = transfer_fn.decode_output(data).map_err(|_| Error::<T>::EVMAbiDecode)?;
			let success = result.first().ok_or_else(|| Error::<T>::EVMAbiDecode)?;
			if let ethabi::Token::Bool(val) = success {
				*val
			} else {
				false
			}
		} else {
			false
		};

		Ok((success, weight))
	}

	/// Get the balance of an ERC20 token for an account.
	pub fn query_erc20_balance_of(
		erc20: H160,
		who: H160,
	) -> Result<(U256, Weight), DispatchErrorWithPostInfo> {
		#[allow(deprecated)]
		let transfer_fn = Function {
			name: String::from("balanceOf"),
			inputs: vec![ethabi::Param {
				name: String::from("who"),
				kind: ethabi::ParamType::Address,
				internal_type: None,
			}],
			outputs: vec![ethabi::Param {
				name: String::from("balance"),
				kind: ethabi::ParamType::Uint(256),
				internal_type: None,
			}],
			constant: None,
			state_mutability: StateMutability::NonPayable,
		};

		let args = [Token::Address(who)];

		log::debug!(target: "evm", "Dispatching EVM call(0x{}): {}", hex::encode(transfer_fn.short_signature()), transfer_fn.signature());
		let data = transfer_fn.encode_input(&args).map_err(|_| Error::<T>::EVMAbiEncode)?;
		let gas_limit = 300_000;
		let info = Self::evm_call(Self::address(), erc20, U256::zero(), data, gas_limit)?;
		let weight = Self::weight_from_call_info(&info);

		// decode the result and return it
		let maybe_value = info.exit_reason.is_succeed().then_some(&info.value);
		let balance = if let Some(data) = maybe_value {
			let result = transfer_fn.decode_output(data).map_err(|_| Error::<T>::EVMAbiDecode)?;
			let success = result.first().ok_or_else(|| Error::<T>::EVMAbiDecode)?;
			if let ethabi::Token::Uint(val) = success {
				*val
			} else {
				U256::zero()
			}
		} else {
			U256::zero()
		};

		Ok((balance, weight))
	}

	/// Dispatches a hook to the EVM and returns if the call was successful with the used weight.
	fn dispatch_hook(
		blueprint: &ServiceBlueprint<T::Constraints>,
		f: Function,
		args: &[ethabi::Token],
		value: BalanceOf<T>,
	) -> Result<(bool, Weight), DispatchErrorWithPostInfo> {
		let mbsm = Self::mbsm_address_of(blueprint)?;
		Self::dispatch_evm_call(mbsm, f, args, value)
			.map(|(info, weight)| (info.exit_reason.is_succeed(), weight))
	}

	/// Dispatches a hook to the EVM and returns if the result with the used weight.
	fn dispatch_evm_call(
		contract: H160,
		f: Function,
		args: &[ethabi::Token],
		value: BalanceOf<T>,
	) -> Result<(fp_evm::CallInfo, Weight), DispatchErrorWithPostInfo> {
		log::debug!(target: "evm", "Dispatching EVM call(0x{}): {}", hex::encode(f.short_signature()), f.signature());
		let data = f.encode_input(args).map_err(|_| Error::<T>::EVMAbiEncode)?;
		let gas_limit = 300_000;
		let value = value.using_encoded(U256::from_little_endian);
		let info = Self::evm_call(Self::address(), contract, value, data, gas_limit)?;
		let weight = Self::weight_from_call_info(&info);
		Ok((info, weight))
	}

	/// Dispatches a call to the EVM and returns the result.
	fn evm_call(
		from: H160,
		to: H160,
		value: U256,
		data: Vec<u8>,
		gas_limit: u64,
	) -> Result<fp_evm::CallInfo, DispatchErrorWithPostInfo> {
		let transactional = true;
		let validate = false;
		let result =
			T::EvmRunner::call(from, to, data.clone(), value, gas_limit, transactional, validate);
		match result {
			Ok(info) => {
				log::debug!(
					target: "evm",
					"Call from: {:?}, to: {:?}, data: 0x{}, gas_limit: {:?}, result: {:?}",
					from,
					to,
					hex::encode(&data),
					gas_limit,
					info,
				);
				// if we have a revert reason, emit an event
				if info.exit_reason.is_revert() {
					log::debug!(
						target: "evm",
						"Call to: {:?} with data: 0x{} Reverted with reason: (0x{})",
						to,
						hex::encode(&data),
						hex::encode(&info.value),
					);
					#[cfg(test)]
					eprintln!(
						"Call to: {:?} with data: 0x{} Reverted with reason: (0x{})",
						to,
						hex::encode(&data),
						hex::encode(&info.value),
					);
					Self::deposit_event(Event::<T>::EvmReverted {
						from,
						to,
						data,
						reason: info.value.clone(),
					});
				}
				Ok(info)
			},
			Err(e) => Err(DispatchErrorWithPostInfo {
				post_info: PostDispatchInfo { actual_weight: Some(e.weight), pays_fee: Pays::Yes },
				error: e.error.into(),
			}),
		}
	}

	/// Convert the gas used in the call info to weight.
	pub fn weight_from_call_info(info: &fp_evm::CallInfo) -> Weight {
		let mut gas_to_weight = T::EvmGasWeightMapping::gas_to_weight(
			info.used_gas.standard.unique_saturated_into(),
			true,
		);
		if let Some(weight_info) = info.weight_info {
			if let Some(proof_size_usage) = weight_info.proof_size_usage {
				*gas_to_weight.proof_size_mut() = proof_size_usage;
			}
		}
		gas_to_weight
	}
}
