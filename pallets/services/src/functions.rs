use core::iter;

#[cfg(not(feature = "std"))]
use alloc::{string::String, vec, vec::Vec};

use ethabi::Token;
use sp_core::{H160, U256};
use sp_runtime::DispatchResultWithInfo;
use tangle_primitives::jobs::v2::{
	Field, ServiceBlueprint, ServiceProviderPrefrences, ServiceRegistrationHook,
};

use super::*;

impl<T: Config> Pallet<T> {
	pub fn check_registeration_hook(
		blueprint: &ServiceBlueprint,
		prefrences: &ServiceProviderPrefrences,
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
