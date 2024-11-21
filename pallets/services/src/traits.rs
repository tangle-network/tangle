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

/// A mapping function that converts EVM gas to Substrate weight and vice versa
pub trait EvmGasWeightMapping {
	/// Convert EVM gas to Substrate weight
	fn gas_to_weight(gas: u64, without_base_weight: bool) -> Weight;
	/// Convert Substrate weight to EVM gas
	fn weight_to_gas(weight: Weight) -> u64;
}

/// Trait to be implemented for evm address mapping.
pub trait EvmAddressMapping<A> {
	/// Convert an address to an account id.
	fn into_account_id(address: H160) -> A;

	/// Convert an account id to an address.
	fn into_address(account_id: A) -> H160;
}
