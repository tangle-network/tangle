use crate::Weight;
use fp_evm::CallInfo;
use sp_core::{H160, U256};
use sp_std::vec::Vec;

#[derive(Debug)]
pub struct RunnerError<E: Into<sp_runtime::DispatchError>> {
	pub error: E,
	pub weight: Weight,
}

#[allow(clippy::too_many_arguments)]
pub trait EvmRunner<T: crate::Config> {
	type Error: Into<sp_runtime::DispatchError>;

	fn call(
		source: H160,
		target: H160,
		input: Vec<u8>,
		value: U256,
		gas_limit: u64,
		is_transactional: bool,
		validate: bool,
	) -> Result<CallInfo, RunnerError<Self::Error>>;
}
