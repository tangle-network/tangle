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
		// keccak256("onRegister(bytes,bytes)")[0..4] = 0xa7c66f86
		const FUNCTION_SELECTOR: [u8; 4] = [0xa7, 0xc6, 0x6f, 0x86];

		let allowed = match blueprint.registration_hook {
			ServiceRegistrationHook::None => true,
			ServiceRegistrationHook::Evm(contract) => {
				let mut data = Vec::new();
				// write the function that will be called on the contract.
				data.extend_from_slice(&FUNCTION_SELECTOR);
				// write the `public_key` argument.
				data.extend_from_slice(&prefrences.encode_to_ethabi());
				// write the arguments that will be passed to the function.
				let args_encoded = Field::encode_to_ethabi(registration_args);
				data.extend_from_slice(&args_encoded);
				let gas_limit = 496;
				let call_info = Self::evm_call(contract, contract, U256::from(0), data, gas_limit)
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
