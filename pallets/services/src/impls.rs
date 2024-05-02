use super::*;

impl<T: Config> traits::EvmRunner<T> for () {
	type Error = crate::Error<T>;

	type Config = ();

	fn validate(
		source: sp_core::H160,
		target: Option<sp_core::H160>,
		input: Vec<u8>,
		value: sp_core::U256,
		gas_limit: u64,
		max_fee_per_gas: Option<sp_core::U256>,
		max_priority_fee_per_gas: Option<sp_core::U256>,
		nonce: Option<sp_core::U256>,
		access_list: Vec<(sp_core::H160, Vec<sp_core::H256>)>,
		is_transactional: bool,
		weight_limit: Option<Weight>,
		proof_size_base_cost: Option<u64>,
		evm_config: &Self::Config,
	) -> Result<(), traits::RunnerError<Self::Error>> {
		Ok(())
	}

	fn call(
		source: sp_core::H160,
		target: sp_core::H160,
		input: Vec<u8>,
		value: sp_core::U256,
		gas_limit: u64,
		max_fee_per_gas: Option<sp_core::U256>,
		max_priority_fee_per_gas: Option<sp_core::U256>,
		nonce: Option<sp_core::U256>,
		access_list: Vec<(sp_core::H160, Vec<sp_core::H256>)>,
		is_transactional: bool,
		validate: bool,
		weight_limit: Option<Weight>,
		proof_size_base_cost: Option<u64>,
		config: &Self::Config,
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
