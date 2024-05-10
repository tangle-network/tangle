use core::iter;

#[cfg(not(feature = "std"))]
use alloc::{string::String, vec, vec::Vec};

use ethabi::Token;
use frame_support::dispatch::{DispatchErrorWithPostInfo, PostDispatchInfo};
use sp_core::{H160, U256};
use sp_runtime::{traits::AccountIdConversion, traits::UniqueSaturatedInto};
use tangle_primitives::jobs::v2::{
	Field, JobDefinition, JobResultVerifier, OperatorPreferences, ServiceBlueprint,
	ServiceRegistrationHook, ServiceRequestHook,
};

use super::*;

impl<T: Config> Pallet<T> {
	/// Returns the account id of the pallet.
	pub fn account_id() -> T::AccountId {
		T::PalletId::get().into_account_truncating()
	}

	/// Returns the address of the pallet.
	pub fn address() -> H160 {
		// Convert the account id to bytes.
		let account_id = Self::account_id().encode();
		// Convert the first 20 bytes to an H160.
		H160::from_slice(&account_id[0..20])
	}

	pub fn check_registeration_hook(
		blueprint: &ServiceBlueprint,
		prefrences: &OperatorPreferences,
		registration_args: &[Field<T::AccountId>],
	) -> Result<(bool, Weight), DispatchErrorWithPostInfo> {
		let (allowed, weight) = match blueprint.registration_hook {
			ServiceRegistrationHook::None => (true, Weight::zero()),
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

				let info =
					Self::evm_call(Self::address(), contract, U256::from(0), data, gas_limit)?;
				(info.exit_reason.is_succeed(), Self::weight_from_call_info(&info))
			},
		};
		Ok((allowed, weight))
	}

	pub fn check_request_hook(
		blueprint: &ServiceBlueprint,
		service_id: u64,
		participants: &[OperatorPreferences],
		request_args: &[Field<T::AccountId>],
	) -> Result<(bool, Weight), DispatchErrorWithPostInfo> {
		let (allowed, weight) = match blueprint.request_hook {
			ServiceRequestHook::None => (true, Weight::zero()),
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

				let info =
					Self::evm_call(Self::address(), contract, U256::from(0), data, gas_limit)?;
				(info.exit_reason.is_succeed(), Self::weight_from_call_info(&info))
			},
		};

		Ok((allowed, weight))
	}

	pub fn check_job_call_hook(
		blueprint: &ServiceBlueprint,
		service_id: u64,
		job: u8,
		job_call_id: u64,
		inputs: &[Field<T::AccountId>],
	) -> Result<(bool, Weight), DispatchErrorWithPostInfo> {
		let (allowed, weight) = match blueprint.request_hook {
			ServiceRequestHook::None => (true, Weight::zero()),
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

				let info =
					Self::evm_call(Self::address(), contract, U256::from(0), data, gas_limit)?;
				(info.exit_reason.is_succeed(), Self::weight_from_call_info(&info))
			},
		};
		Ok((allowed, weight))
	}

	pub fn check_job_call_result_hook(
		job_def: &JobDefinition,
		service_id: u64,
		job: u8,
		job_call_id: u64,
		prefrences: &OperatorPreferences,
		inputs: &[Field<T::AccountId>],
		outputs: &[Field<T::AccountId>],
	) -> Result<(bool, Weight), DispatchErrorWithPostInfo> {
		let (allowed, weight) = match job_def.verifier {
			JobResultVerifier::None => (true, Weight::zero()),
			JobResultVerifier::Evm(contract) => {
				#[allow(deprecated)]
				let call = ethabi::Function {
					name: String::from("verify"),
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
							name: String::from("participant"),
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
					constant: false,
					state_mutability: ethabi::StateMutability::NonPayable,
				};
				let service_id = Token::Uint(ethabi::Uint::from(service_id));
				let job = Token::Uint(ethabi::Uint::from(job));
				let job_call_id = Token::Uint(ethabi::Uint::from(job_call_id));
				let participant = prefrences.to_ethabi().first().unwrap().clone();
				let inputs = Token::Bytes(Field::encode_to_ethabi(inputs));
				let outputs = Token::Bytes(Field::encode_to_ethabi(outputs));
				eprintln!("inputs: {}", inputs);
				eprintln!("outputs: {}", outputs);
				let data = call
					.encode_input(&[service_id, job, job_call_id, participant, inputs, outputs])
					.map_err(|_| Error::<T>::EVMAbiEncode)?;
				let gas_limit = 300_000;

				let info =
					Self::evm_call(Self::address(), contract, U256::from(0), data, gas_limit)?;
				eprintln!("info: {:?}", info);
				(info.exit_reason.is_succeed(), Self::weight_from_call_info(&info))
			},
		};
		Ok((allowed, weight))
	}

	pub fn evm_call(
		from: H160,
		to: H160,
		value: U256,
		data: Vec<u8>,
		gas_limit: u64,
	) -> Result<fp_evm::CallInfo, DispatchErrorWithPostInfo> {
		let transactional = true;
		let validate = false;
		let result = T::EvmRunner::call(from, to, data, value, gas_limit, transactional, validate);
		match result {
			Ok(info) => Ok(info),
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
