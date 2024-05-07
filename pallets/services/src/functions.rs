use core::iter;

#[cfg(not(feature = "std"))]
use alloc::{string::String, vec, vec::Vec};

use ethabi::Token;
use sp_core::{H160, U256};
use sp_runtime::DispatchResultWithInfo;
use tangle_primitives::jobs::v2::{
	Field, OperatorPreferences, ServiceBlueprint, ServiceRegistrationHook, ServiceRequestHook,
};

use super::*;

impl<T: Config> Pallet<T> {
	pub fn check_registeration_hook(
		blueprint: &ServiceBlueprint,
		prefrences: &OperatorPreferences,
		registration_args: &[Field<T::AccountId>],
	) -> DispatchResultWithInfo<bool> {
		let allowed = match blueprint.registration_hook {
			ServiceRegistrationHook::None => true,
			ServiceRegistrationHook::Evm(contract) => {
				#[allow(deprecated)]
				let call = ethabi::Function {
					name: String::from("onRegister"),
					inputs: vec![
						ethabi::Param {
							name: String::from("participant"),
							kind: ethabi::ParamType::Bytes,
							internal_type: None,
						},
						ethabi::Param {
							name: String::from("registrationInputs"),
							kind: ethabi::ParamType::Bytes,
							internal_type: None,
						},
					],
					outputs: Default::default(),
					constant: false,
					state_mutability: ethabi::StateMutability::Payable,
				};
				let args = prefrences
					.to_ethabi()
					.into_iter()
					.chain(iter::once(Token::Bytes(Field::encode_to_ethabi(registration_args))))
					.collect::<Vec<_>>();

				let data = call.encode_input(&args).map_err(|_| Error::<T>::EVMAbiEncode)?;
				let gas_limit = 300_000;

				let call_info = Self::evm_call(
					T::RuntimeEvmAddress::get(),
					contract,
					U256::from(0),
					data,
					gas_limit,
				)
				.map_err(|r| r.error.into())?;
				call_info.exit_reason.is_succeed()
			},
		};
		Ok(allowed)
	}

	pub fn check_request_hook(
		blueprint: &ServiceBlueprint,
		service_id: u64,
		participants: &[OperatorPreferences],
		request_args: &[Field<T::AccountId>],
	) -> DispatchResultWithInfo<bool> {
		let allowed = match blueprint.request_hook {
			ServiceRequestHook::None => true,
			ServiceRequestHook::Evm(contract) => {
				#[allow(deprecated)]
				let call = ethabi::Function {
					name: String::from("onRequest"),
					inputs: vec![
						ethabi::Param {
							name: String::from("serviceId"),
							kind: ethabi::ParamType::Uint(64),
							internal_type: None,
						},
						ethabi::Param {
							name: String::from("participants"),
							kind: ethabi::ParamType::Array(Box::new(ethabi::ParamType::Bytes)),
							internal_type: None,
						},
						ethabi::Param {
							name: String::from("requestInputs"),
							kind: ethabi::ParamType::Bytes,
							internal_type: None,
						},
					],
					outputs: Default::default(),
					constant: false,
					state_mutability: ethabi::StateMutability::Payable,
				};
				let service_id = Token::Uint(ethabi::Uint::from(service_id));
				let participants = Token::Array(
					participants.iter().flat_map(OperatorPreferences::to_ethabi).collect(),
				);
				let request_args = Token::Bytes(Field::encode_to_ethabi(request_args));
				let data = call
					.encode_input(&[service_id, participants, request_args])
					.map_err(|_| Error::<T>::EVMAbiEncode)?;
				let gas_limit = 300_000;

				let call_info = Self::evm_call(
					T::RuntimeEvmAddress::get(),
					contract,
					U256::from(0),
					data,
					gas_limit,
				)
				.map_err(|r| r.error.into())?;
				call_info.exit_reason.is_succeed()
			},
		};
		Ok(allowed)
	}

	pub fn check_job_call_hook(
		blueprint: &ServiceBlueprint,
		service_id: u64,
		job: u8,
		job_call_id: u64,
		inputs: &[Field<T::AccountId>],
	) -> DispatchResultWithInfo<bool> {
		let allowed = match blueprint.request_hook {
			ServiceRequestHook::None => true,
			ServiceRequestHook::Evm(contract) => {
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
					constant: false,
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

				let call_info = Self::evm_call(
					T::RuntimeEvmAddress::get(),
					contract,
					U256::from(0),
					data,
					gas_limit,
				)
				.map_err(|r| r.error.into())?;
				call_info.exit_reason.is_succeed()
			},
		};
		Ok(allowed)
	}

	pub fn evm_call(
		from: H160,
		to: H160,
		value: U256,
		data: Vec<u8>,
		gas_limit: u64,
	) -> Result<fp_evm::CallInfo, RunnerError<<T::EvmRunner as EvmRunner<T>>::Error>> {
		let transactional = true;
		let validate = false;
		T::EvmRunner::call(from, to, data, value, gas_limit, transactional, validate)
	}
}
