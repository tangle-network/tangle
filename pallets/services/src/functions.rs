use sp_core::{H160, U256};
use sp_runtime::DispatchResultWithInfo;
use tangle_primitives::jobs::v2::{Field, ServiceBlueprint, ServiceRegistrationHook};

use super::*;

impl<T: Config> Pallet<T> {
	pub fn check_registeration_hook(
		blueprint: &ServiceBlueprint,
		registration_args: &[Field<T::AccountId>],
	) -> DispatchResultWithInfo<bool> {
		// keccak256("onRegister(bytes)")[0..4]
		const FUNCTION_SELECTOR: [u8; 4] = [0x6c, 0xb9, 0xd7, 0xe1];
		let allowed = match blueprint.registration_hook {
			ServiceRegistrationHook::None => true,
			ServiceRegistrationHook::Evm(contract) => {
				let mut data = Vec::new();
				// write the function that will be called on the contract.
				data.extend_from_slice(&FUNCTION_SELECTOR);
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
