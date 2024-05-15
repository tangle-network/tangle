use super::*;

impl<T: Config> traits::EvmRunner<T> for () {
	type Error = crate::Error<T>;

	fn call(
		_source: sp_core::H160,
		_target: sp_core::H160,
		_input: Vec<u8>,
		_value: sp_core::U256,
		_gas_limit: u64,
		_is_transactional: bool,
		_validate: bool,
	) -> Result<fp_evm::CallInfo, traits::RunnerError<Self::Error>> {
		Ok(fp_evm::CallInfo {
			exit_reason: fp_evm::ExitReason::Succeed(fp_evm::ExitSucceed::Stopped),
			value: Default::default(),
			used_gas: fp_evm::UsedGas {
				standard: Default::default(),
				effective: Default::default(),
			},
			weight_info: Default::default(),
			logs: Default::default(),
		})
	}
}
