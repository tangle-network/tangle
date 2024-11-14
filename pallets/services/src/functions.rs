use core::iter;

#[cfg(not(feature = "std"))]
use alloc::{boxed::Box, string::String, vec, vec::Vec};

#[cfg(feature = "std")]
use std::{boxed::Box, string::String, vec::Vec};

use ethabi::Token;
use frame_support::dispatch::{DispatchErrorWithPostInfo, PostDispatchInfo};
use sp_core::{H160, U256};
use sp_runtime::traits::UniqueSaturatedInto;
use tangle_primitives::services::{
	BlueprintManager, Field, OperatorPreferences, Service, ServiceBlueprint,
};

use super::*;
use crate::types::BalanceOf;

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

	/// Hook to be called upon a new operator registration on a blueprint.
	///
	/// This function is called when a service is registered. It performs an EVM call
	/// to the `onRegister` function of the service blueprint's manager contract.
	///
	/// # Parameters
	/// * `blueprint` - The service blueprint.
	/// * `prefrences` - The operator preferences.
	/// * `registration_args` - The registration arguments.
	/// * `value` - The value to be sent with the call.
	///
	/// # Returns
	/// * `Result<(bool, Weight), DispatchErrorWithPostInfo>` - A tuple containing a boolean indicating
	///   whether the registration is allowed and the weight of the operation.
	pub fn on_register_hook(
		blueprint: &ServiceBlueprint<T::Constraints>,
		prefrences: &OperatorPreferences,
		registration_args: &[Field<T::Constraints, T::AccountId>],
		value: BalanceOf<T>,
	) -> Result<(bool, Weight), DispatchErrorWithPostInfo> {
		let (allowed, weight) = match blueprint.manager {
			BlueprintManager::Evm(contract) => {
				#[allow(deprecated)]
				let call = ethabi::Function {
					name: String::from("onRegister"),
					inputs: vec![
						OperatorPreferences::to_ethabi_param(),
						ethabi::Param {
							name: String::from("registrationInputs"),
							kind: ethabi::ParamType::Bytes,
							internal_type: None,
						},
					],
					outputs: Default::default(),
					constant: None,
					state_mutability: ethabi::StateMutability::Payable,
				};

				let args = prefrences
					.to_ethabi()
					.into_iter()
					.chain(iter::once(Token::Bytes(Field::encode_to_ethabi(registration_args))))
					.collect::<Vec<_>>();

				let value = value.using_encoded(|bytes| U256::from_little_endian(&bytes));
				let data = call.encode_input(&args).map_err(|_| Error::<T>::EVMAbiEncode)?;
				let gas_limit = 300_000;

				let info = Self::evm_call(Self::address(), contract, value, data, gas_limit)?;
				(info.exit_reason.is_succeed(), Self::weight_from_call_info(&info))
			},
			_ => (true, Weight::zero()),
		};
		Ok((allowed, weight))
	}

	/// Hook to be called upon an operator unregistration on a blueprint.
	///
	/// This function is called when an operator is unregistered. It performs an EVM call
	/// to the `onUnregister` function of the service blueprint's manager contract.
	///
	/// # Parameters
	/// * `blueprint` - The service blueprint.
	/// * `prefrences` - The operator preferences.
	///
	/// # Returns
	/// * `Result<(bool, Weight), DispatchErrorWithPostInfo>` - A tuple containing a boolean indicating
	///  whether the unregistration is allowed and the weight of the operation.
	pub fn on_unregister_hook(
		blueprint: &ServiceBlueprint<T::Constraints>,
		prefrences: &OperatorPreferences,
	) -> Result<(bool, Weight), DispatchErrorWithPostInfo> {
		match blueprint.manager {
			BlueprintManager::Evm(contract) => {
				#[allow(deprecated)]
				let call = ethabi::Function {
					name: String::from("onUnregister"),
					inputs: vec![OperatorPreferences::to_ethabi_param()],
					outputs: Default::default(),
					constant: None,
					state_mutability: ethabi::StateMutability::NonPayable,
				};

				let args = prefrences.to_ethabi();
				let data = call.encode_input(&args).map_err(|_| Error::<T>::EVMAbiEncode)?;
				let gas_limit = 300_000;

				let info = Self::evm_call(Self::address(), contract, 0.into(), data, gas_limit)?;
				Ok((info.exit_reason.is_succeed(), Self::weight_from_call_info(&info)))
			},
			_ => Ok((true, Weight::zero())),
		}
	}

	/// Hook to be called upon a new price targets update on a blueprint.
	/// This function is called when the price targets are updated. It performs an EVM call
	/// to the `onUpdatePriceTargets` function of the service blueprint's manager contract.
	///
	/// # Parameters
	/// * `blueprint` - The service blueprint.
	/// * `prefrences` - The operator preferences.
	///
	/// # Returns
	///
	/// * `Result<(bool, Weight), DispatchErrorWithPostInfo>` - A tuple containing a boolean indicating
	///  whether the price targets update is allowed and the weight of the operation.
	pub fn on_update_price_targets(
		blueprint: &ServiceBlueprint<T::Constraints>,
		prefrences: &OperatorPreferences,
	) -> Result<(bool, Weight), DispatchErrorWithPostInfo> {
		match blueprint.manager {
			BlueprintManager::Evm(contract) => {
				#[allow(deprecated)]
				let call = ethabi::Function {
					name: String::from("onUpdatePriceTargets"),
					inputs: vec![OperatorPreferences::to_ethabi_param()],
					outputs: Default::default(),
					constant: None,
					state_mutability: ethabi::StateMutability::Payable,
				};

				let args = prefrences.to_ethabi();
				let data = call.encode_input(&args).map_err(|_| Error::<T>::EVMAbiEncode)?;
				let gas_limit = 300_000;

				let info = Self::evm_call(Self::address(), contract, 0.into(), data, gas_limit)?;
				Ok((info.exit_reason.is_succeed(), Self::weight_from_call_info(&info)))
			},
			_ => Ok((true, Weight::zero())),
		}
	}

	/// Hook to be called upon an operator approve a service request on a blueprint.
	///
	/// This function is called when an operator approve a service request. It performs an EVM call
	/// to the `onApprove` function of the service blueprint's manager contract.
	///
	/// # Parameters
	/// * `blueprint` - The service blueprint.
	/// * `prefrences` - The operator preferences.
	/// * `request_id` - The request id.
	/// * `restaking_percent` - The restaking percent.
	///
	/// # Returns
	/// * `Result<(bool, Weight), DispatchErrorWithPostInfo>` - A tuple containing a boolean indicating
	/// whether the approve is allowed and the weight of the operation.
	pub fn on_approve_hook(
		blueprint: &ServiceBlueprint<T::Constraints>,
		prefrences: &OperatorPreferences,
		request_id: u64,
		restaking_percent: u8,
	) -> Result<(bool, Weight), DispatchErrorWithPostInfo> {
		match blueprint.manager {
			BlueprintManager::Evm(contract) => {
				#[allow(deprecated)]
				let call = ethabi::Function {
					name: String::from("onApprove"),
					inputs: vec![
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
					state_mutability: ethabi::StateMutability::NonPayable,
				};

				let args = prefrences
					.to_ethabi()
					.into_iter()
					.chain(iter::once(Token::Uint(U256::from(request_id))))
					.chain(iter::once(Token::Uint(U256::from(restaking_percent))))
					.collect::<Vec<_>>();

				let data = call.encode_input(&args).map_err(|_| Error::<T>::EVMAbiEncode)?;
				let gas_limit = 300_000;

				let info = Self::evm_call(Self::address(), contract, 0.into(), data, gas_limit)?;
				Ok((info.exit_reason.is_succeed(), Self::weight_from_call_info(&info)))
			},
			_ => Ok((true, Weight::zero())),
		}
	}

	/// Hook to be called upon an operator reject a service request on a blueprint.
	/// This function is called when an operator reject a service request. It performs an EVM call
	/// to the `onReject` function of the service blueprint's manager contract.
	///
	/// # Parameters
	/// * `blueprint` - The service blueprint.
	/// * `prefrences` - The operator preferences.
	/// * `request_id` - The request id.
	///
	/// # Returns
	/// * `Result<(bool, Weight), DispatchErrorWithPostInfo>` - A tuple containing a boolean indicating
	/// whether the reject is allowed and the weight of the operation.
	pub fn on_reject_hook(
		blueprint: &ServiceBlueprint<T::Constraints>,
		prefrences: &OperatorPreferences,
		request_id: u64,
	) -> Result<(bool, Weight), DispatchErrorWithPostInfo> {
		match blueprint.manager {
			BlueprintManager::Evm(contract) => {
				#[allow(deprecated)]
				let call = ethabi::Function {
					name: String::from("onReject"),
					inputs: vec![
						OperatorPreferences::to_ethabi_param(),
						ethabi::Param {
							name: String::from("requestId"),
							kind: ethabi::ParamType::Uint(64),
							internal_type: None,
						},
					],
					outputs: Default::default(),
					constant: None,
					state_mutability: ethabi::StateMutability::NonPayable,
				};

				let args = prefrences
					.to_ethabi()
					.into_iter()
					.chain(iter::once(Token::Uint(U256::from(request_id))))
					.collect::<Vec<_>>();

				let data = call.encode_input(&args).map_err(|_| Error::<T>::EVMAbiEncode)?;
				let gas_limit = 300_000;

				let info = Self::evm_call(Self::address(), contract, 0.into(), data, gas_limit)?;
				Ok((info.exit_reason.is_succeed(), Self::weight_from_call_info(&info)))
			},
			_ => Ok((true, Weight::zero())),
		}
	}

	/// Hook to be called upon new service request.
	///
	/// This function is called when a service request is made. It performs an EVM call
	/// to the `onRequest` function of the service blueprint's manager contract.
	///
	/// # Parameters
	/// * `requester` - The requester of the service.
	/// * `blueprint` - The service blueprint.
	/// * `request_id` - The service request ID.
	/// * `operators` - The operator preferences.
	/// * `request_args` - The request arguments.
	/// * `permitted_callers` - The permitted callers.
	/// * `assets` - The assets to be used.
	/// * `ttl` - The time to live.
	/// * `value` - The value to be sent with the call.
	///
	/// # Returns
	/// * `Result<(bool, Weight), DispatchErrorWithPostInfo>` - A tuple containing a boolean indicating
	///   whether the request is allowed and the weight of the operation.
	pub fn on_request_hook(
		requester: &T::AccountId,
		blueprint: &ServiceBlueprint<T::Constraints>,
		request_id: u64,
		operators: &[OperatorPreferences],
		request_args: &[Field<T::Constraints, T::AccountId>],
		permitted_callers: &[T::AccountId],
		_assets: &[T::AssetId],
		ttl: BlockNumberFor<T>,
		value: BalanceOf<T>,
	) -> Result<(bool, Weight), DispatchErrorWithPostInfo> {
		let (allowed, weight) = match blueprint.manager {
			BlueprintManager::Evm(contract) => {
				#[allow(deprecated)]
				let call = ethabi::Function {
					name: String::from("onRequest"),
					inputs: vec![
						ethabi::Param {
							name: String::from("requestId"),
							kind: ethabi::ParamType::Uint(64),
							internal_type: None,
						},
						ethabi::Param {
							name: String::from("requester"),
							kind: ethabi::ParamType::Address,
							internal_type: None,
						},
						ethabi::Param {
							name: String::from("operatorsWithPreferences"),
							kind: ethabi::ParamType::Array(Box::new(
								OperatorPreferences::to_ethabi_param_type(),
							)),
							internal_type: Some(String::from("OperatorPreferences[]")),
						},
						ethabi::Param {
							name: String::from("requestInputs"),
							kind: ethabi::ParamType::Bytes,
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
					state_mutability: ethabi::StateMutability::Payable,
				};
				let request_id = Token::Uint(ethabi::Uint::from(request_id));
				let requester =
					Token::Address(T::EvmAddressMapping::into_address(requester.clone()));
				let operators = Token::Array(
					operators.iter().flat_map(OperatorPreferences::to_ethabi).collect(),
				);
				let request_args = Token::Bytes(Field::encode_to_ethabi(request_args));
				let permitted_callers = Token::Array(
					permitted_callers
						.iter()
						.map(|caller| {
							Token::Address(T::EvmAddressMapping::into_address(caller.clone()))
						})
						.collect(),
				);
				// TODO: Implement asset mapping or when we have AssetIds as an enum.
				// let assets = Token::Array(vec![]);
				let ttl = Token::Uint(ethabi::Uint::from(ttl.into()));
				let data = call
					.encode_input(&[
						request_id,
						requester,
						operators,
						request_args,
						permitted_callers,
						// assets,
						ttl,
					])
					.map_err(|_| Error::<T>::EVMAbiEncode)?;
				let value = value.using_encoded(|bytes| U256::from_little_endian(&bytes));
				let gas_limit = 300_000;

				let info = Self::evm_call(Self::address(), contract, value, data, gas_limit)?;
				(info.exit_reason.is_succeed(), Self::weight_from_call_info(&info))
			},
			_ => (true, Weight::zero()),
		};

		Ok((allowed, weight))
	}

	/// Hook to be called when a service is initialized. This function will call the `onServiceInit`
	/// function of the service blueprint manager contract.
	///
	/// # Arguments
	/// * `blueprint` - The service blueprint.
	/// * `request_id` - The service request ID.
	/// * `service_id` - The service ID.
	/// * `owner` - The owner of the service.
	/// * `permitted_callers` - The permitted callers.
	/// * `assets` - The assets to be used.
	/// * `ttl` - The time to live.
	///
	/// # Returns
	/// * `Result<(bool, Weight), DispatchErrorWithPostInfo>` - A tuple containing a boolean indicating
	///  whether the request is allowed and the weight of the operation.
	pub fn on_service_init_hook(
		blueprint: &ServiceBlueprint<T::Constraints>,
		request_id: u64,
		service_id: u64,
		owner: &T::AccountId,
		permitted_callers: &[T::AccountId],
		_assets: &[T::AssetId],
		ttl: BlockNumberFor<T>,
	) -> Result<(bool, Weight), DispatchErrorWithPostInfo> {
		let (allowed, weight) = match blueprint.manager {
			BlueprintManager::Evm(contract) => {
				#[allow(deprecated)]
				let call = ethabi::Function {
					name: String::from("onServiceInit"),
					inputs: vec![
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
					state_mutability: ethabi::StateMutability::Payable,
				};
				let request_id = Token::Uint(ethabi::Uint::from(request_id));
				let service_id = Token::Uint(ethabi::Uint::from(service_id));
				let owner = Token::Address(T::EvmAddressMapping::into_address(owner.clone()));
				let permitted_callers = Token::Array(
					permitted_callers
						.iter()
						.map(|caller| {
							Token::Address(T::EvmAddressMapping::into_address(caller.clone()))
						})
						.collect(),
				);
				let ttl = Token::Uint(ethabi::Uint::from(ttl.into()));
				let data = call
					.encode_input(&[
						request_id,
						service_id,
						owner,
						permitted_callers,
						// assets,
						ttl,
					])
					.map_err(|_| Error::<T>::EVMAbiEncode)?;
				let value = U256::zero();
				let gas_limit = 300_000;
				let info = Self::evm_call(Self::address(), contract, value, data, gas_limit)?;
				(info.exit_reason.is_succeed(), Self::weight_from_call_info(&info))
			},
			_ => (true, Weight::zero()),
		};
		Ok((allowed, weight))
	}

	/// Hook to be called when a service is terminated. This function will call the `onServiceTermination`
	/// function of the service blueprint manager contract.
	///
	/// # Arguments
	/// * `blueprint` - The service blueprint.
	/// * `service_id` - The service ID.
	/// * `owner` - The owner of the service.
	///
	/// # Returns
	/// * `Result<(bool, Weight), DispatchErrorWithPostInfo>` - A tuple containing a boolean indicating
	/// whether the request is allowed and the weight of the operation.
	pub fn on_service_termination_hook(
		blueprint: &ServiceBlueprint<T::Constraints>,
		service_id: u64,
		owner: &T::AccountId,
	) -> Result<(bool, Weight), DispatchErrorWithPostInfo> {
		let (allowed, weight) = match blueprint.manager {
			BlueprintManager::Evm(contract) => {
				#[allow(deprecated)]
				let call = ethabi::Function {
					name: String::from("onServiceTermination"),
					inputs: vec![
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
					state_mutability: ethabi::StateMutability::NonPayable,
				};
				let service_id = Token::Uint(ethabi::Uint::from(service_id));
				let owner = Token::Address(T::EvmAddressMapping::into_address(owner.clone()));
				let data = call
					.encode_input(&[service_id, owner])
					.map_err(|_| Error::<T>::EVMAbiEncode)?;
				let value = U256::zero();
				let gas_limit = 300_000;
				let info = Self::evm_call(Self::address(), contract, value, data, gas_limit)?;
				(info.exit_reason.is_succeed(), Self::weight_from_call_info(&info))
			},
			_ => (true, Weight::zero()),
		};
		Ok((allowed, weight))
	}

	/// Hook to be called upon job call.
	///
	/// This function is called when a job call is made. It performs an EVM call
	/// to the `onJobCall` function of the service blueprint's manager contract.
	///
	/// # Parameters
	/// * `blueprint` - The service blueprint.
	/// * `service_id` - The service ID.
	/// * `job` - The job index.
	/// * `job_call_id` - The job call ID.
	/// * `inputs` - The input fields.
	///
	/// # Returns
	/// * `Result<(bool, Weight), DispatchErrorWithPostInfo>` - A tuple containing a boolean indicating
	///   whether the job call is allowed and the weight of the operation.
	pub fn on_job_call_hook(
		blueprint: &ServiceBlueprint<T::Constraints>,
		service_id: u64,
		job: u8,
		job_call_id: u64,
		inputs: &[Field<T::Constraints, T::AccountId>],
	) -> Result<(bool, Weight), DispatchErrorWithPostInfo> {
		let (allowed, weight) = match blueprint.manager {
			BlueprintManager::Evm(contract) => {
				#[allow(deprecated)]
				let call = ethabi::Function {
					name: String::from("onJobCall"),
					inputs: vec![
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
					state_mutability: ethabi::StateMutability::Payable,
				};
				let service_id = Token::Uint(ethabi::Uint::from(service_id));
				let job = Token::Uint(ethabi::Uint::from(job));
				let job_call_id = Token::Uint(ethabi::Uint::from(job_call_id));
				let inputs = Token::Bytes(Field::encode_to_ethabi(inputs));
				let data = call
					.encode_input(&[service_id, job, job_call_id, inputs])
					.map_err(|_| Error::<T>::EVMAbiEncode)?;
				let gas_limit = 300_000;

				let info =
					Self::evm_call(Self::address(), contract, U256::from(0), data, gas_limit)?;
				(info.exit_reason.is_succeed(), Self::weight_from_call_info(&info))
			},
			_ => (true, Weight::zero()),
		};
		Ok((allowed, weight))
	}

	/// Hook to be called upon job result.
	///
	/// This function is called when a job result is submitted. It performs an EVM call
	/// to the `onJobResult` function of the service blueprint's manager contract.
	///
	/// # Parameters
	/// * `blueprint` - The service blueprint.
	/// * `service_id` - The service ID.
	/// * `job` - The job index.
	/// * `job_call_id` - The job call ID.
	/// * `prefrences` - The operator preferences.
	/// * `inputs` - The input fields.
	/// * `outputs` - The output fields.
	///
	/// # Returns
	/// * `Result<(bool, Weight), DispatchErrorWithPostInfo>` - A tuple containing a boolean indicating
	///   whether the job result is allowed and the weight of the operation.
	pub fn on_job_result_hook(
		blueprint: &ServiceBlueprint<T::Constraints>,
		service_id: u64,
		job: u8,
		job_call_id: u64,
		prefrences: &OperatorPreferences,
		inputs: &[Field<T::Constraints, T::AccountId>],
		outputs: &[Field<T::Constraints, T::AccountId>],
	) -> Result<(bool, Weight), DispatchErrorWithPostInfo> {
		let (allowed, weight) = match blueprint.manager {
			BlueprintManager::Evm(contract) => {
				#[allow(deprecated)]
				let call = ethabi::Function {
					name: String::from("onJobResult"),
					inputs: vec![
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
						ethabi::Param {
							name: String::from("operator"),
							kind: ethabi::ParamType::Bytes,
							internal_type: None,
						},
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
					state_mutability: ethabi::StateMutability::NonPayable,
				};
				let service_id = Token::Uint(ethabi::Uint::from(service_id));
				let job = Token::Uint(ethabi::Uint::from(job));
				let job_call_id = Token::Uint(ethabi::Uint::from(job_call_id));
				let operator = prefrences.to_ethabi().first().unwrap().clone();
				let inputs = Token::Bytes(Field::encode_to_ethabi(inputs));
				let outputs = Token::Bytes(Field::encode_to_ethabi(outputs));
				let data = call
					.encode_input(&[service_id, job, job_call_id, operator, inputs, outputs])
					.map_err(|_| Error::<T>::EVMAbiEncode)?;
				let gas_limit = 300_000;

				let info =
					Self::evm_call(Self::address(), contract, U256::from(0), data, gas_limit)?;
				(info.exit_reason.is_succeed(), Self::weight_from_call_info(&info))
			},
			_ => (true, Weight::zero()),
		};
		Ok((allowed, weight))
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
	/// * `Result<(Option<T::AccountId>, Weight), DispatchErrorWithPostInfo>` - A tuple containing the
	///   slashing origin account id (if any) and the weight of the operation.
	pub fn query_slashing_origin(
		service: &Service<T::Constraints, T::AccountId, BlockNumberFor<T>, T::AssetId>,
	) -> Result<(Option<T::AccountId>, Weight), DispatchErrorWithPostInfo> {
		let (_, blueprint) = Self::blueprints(service.blueprint)?;
		#[allow(deprecated)]
		let query_call = ethabi::Function {
			name: String::from("querySlashingOrigin"),
			inputs: vec![ethabi::Param {
				name: String::from("serviceId"),
				kind: ethabi::ParamType::Uint(64),
				internal_type: None,
			}],
			outputs: vec![ethabi::Param {
				name: String::from("slashingOrigin"),
				kind: ethabi::ParamType::Address,
				internal_type: None,
			}],
			constant: None,
			state_mutability: ethabi::StateMutability::NonPayable,
		};
		let service_id_tok = ethabi::Token::Uint(ethabi::Uint::from(service.id));
		let blueprint_manager =
			blueprint.manager.try_into_evm().map_err(|_| Error::<T>::EVMAbiEncode)?;
		let info = Self::evm_call(
			Self::address(),
			blueprint_manager,
			U256::from(0),
			query_call
				.encode_input(&[service_id_tok])
				.map_err(|_| Error::<T>::EVMAbiEncode)?,
			300_000,
		)?;

		// decode the result and return it
		let maybe_value = info.exit_reason.is_succeed().then_some(&info.value);
		let slashing_origin = if let Some(data) = maybe_value {
			let result = query_call.decode_output(data).map_err(|_| Error::<T>::EVMAbiDecode)?;
			let slashing_origin = result.first().ok_or_else(|| Error::<T>::EVMAbiDecode)?;
			if let ethabi::Token::Address(who) = slashing_origin {
				Some(T::EvmAddressMapping::into_account_id(*who))
			} else {
				None
			}
		} else {
			None
		};

		Ok((slashing_origin, Self::weight_from_call_info(&info)))
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
	/// * `Result<(Option<T::AccountId>, Weight), DispatchErrorWithPostInfo>` - A tuple containing the
	///   dispute origin account id (if any) and the weight of the operation.
	pub fn query_dispute_origin(
		service: &Service<T::Constraints, T::AccountId, BlockNumberFor<T>, T::AssetId>,
	) -> Result<(Option<T::AccountId>, Weight), DispatchErrorWithPostInfo> {
		let (_, blueprint) = Self::blueprints(service.blueprint)?;
		#[allow(deprecated)]
		let query_call = ethabi::Function {
			name: String::from("queryDisputeOrigin"),
			inputs: vec![ethabi::Param {
				name: String::from("serviceId"),
				kind: ethabi::ParamType::Uint(64),
				internal_type: None,
			}],
			outputs: vec![ethabi::Param {
				name: String::from("disputeOrigin"),
				kind: ethabi::ParamType::Address,
				internal_type: None,
			}],
			constant: None,
			state_mutability: ethabi::StateMutability::NonPayable,
		};
		let service_id_tok = ethabi::Token::Uint(ethabi::Uint::from(service.id));
		let blueprint_manager =
			blueprint.manager.try_into_evm().map_err(|_| Error::<T>::EVMAbiEncode)?;
		let info = Self::evm_call(
			Self::address(),
			blueprint_manager,
			U256::from(0),
			query_call
				.encode_input(&[service_id_tok])
				.map_err(|_| Error::<T>::EVMAbiEncode)?,
			300_000,
		)?;

		// decode the result and return it
		let maybe_value = info.exit_reason.is_succeed().then_some(&info.value);
		let dispute_origin = if let Some(data) = maybe_value {
			let result = query_call.decode_output(data).map_err(|_| Error::<T>::EVMAbiDecode)?;
			let slashing_origin = result.first().ok_or_else(|| Error::<T>::EVMAbiDecode)?;
			if let ethabi::Token::Address(who) = slashing_origin {
				Some(T::EvmAddressMapping::into_account_id(*who))
			} else {
				None
			}
		} else {
			None
		};

		Ok((dispute_origin, Self::weight_from_call_info(&info)))
	}

	/// Dispatches a call to the EVM and returns the result.
	pub fn evm_call(
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
					Self::deposit_event(Event::<T>::EvmReverted {
						from,
						to,
						data,
						reason: info.value.clone(),
					});
				}
				// Emit logs from the EVM call.
				for log in &info.logs {
					Self::deposit_event(Event::<T>::EvmLog {
						address: log.address,
						topics: log.topics.clone(),
						data: log.data.clone(),
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
