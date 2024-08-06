#[cfg(not(feature = "std"))]
use alloc::vec::Vec;
#[cfg(feature = "std")]
use std::vec::Vec;
use tangle_primitives::services::Constraints;

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

impl<T: Config> Constraints for types::ConstraintsOf<T> {
	type MaxFields = T::MaxFields;

	type MaxFieldsSize = T::MaxFieldsSize;

	type MaxMetadataLength = T::MaxMetadataLength;

	type MaxJobsPerService = T::MaxJobsPerService;

	type MaxOperatorsPerService = T::MaxOperatorsPerService;

	type MaxPermittedCallers = T::MaxPermittedCallers;

	type MaxServicesPerOperator = T::MaxServicesPerOperator;

	type MaxBlueprintsPerOperator = T::MaxBlueprintsPerOperator;

	type MaxServicesPerUser = T::MaxServicesPerUser;

	type MaxBinariesPerGadget = T::MaxBinariesPerGadget;

	type MaxSourcesPerGadget = T::MaxSourcesPerGadget;

	type MaxGitOwnerLength = T::MaxGitOwnerLength;

	type MaxGitRepoLength = T::MaxGitRepoLength;

	type MaxGitTagLength = T::MaxGitTagLength;

	type MaxBinaryNameLength = T::MaxBinaryNameLength;

	type MaxIpfsHashLength = T::MaxIpfsHashLength;

	type MaxContainerRegistryLength = T::MaxContainerRegistryLength;

	type MaxContainerImageNameLength = T::MaxContainerImageNameLength;

	type MaxContainerImageTagLength = T::MaxContainerImageTagLength;
}

impl traits::EvmGasWeightMapping for () {
	fn gas_to_weight(_gas: u64, _without_base_weight: bool) -> Weight {
		Default::default()
	}
	fn weight_to_gas(_weight: Weight) -> u64 {
		Default::default()
	}
}
