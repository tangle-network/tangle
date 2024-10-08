use super::*;
use crate::types::BalanceOf;
#[cfg(not(feature = "std"))]
use alloc::vec::Vec;
#[cfg(feature = "std")]
use std::vec::Vec;
use tangle_primitives::{services::Constraints, traits::ServiceManager};

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

	type MaxAssetsPerService = T::MaxAssetsPerService;
}

impl traits::EvmGasWeightMapping for () {
	fn gas_to_weight(_gas: u64, _without_base_weight: bool) -> Weight {
		Default::default()
	}
	fn weight_to_gas(_weight: Weight) -> u64 {
		Default::default()
	}
}

impl<T: crate::Config> ServiceManager<T::AccountId, BalanceOf<T>> for crate::Pallet<T> {
	fn get_active_services_count(operator: &T::AccountId) -> usize {
		OperatorsProfile::<T>::get(operator)
			.map_or(Default::default(), |profile| profile.services.len())
	}

	fn get_active_blueprints_count(operator: &T::AccountId) -> usize {
		OperatorsProfile::<T>::get(operator)
			.map_or(Default::default(), |profile| profile.blueprints.len())
	}

	/// Operator can exit if no active services or blueprints
	fn can_exit(operator: &T::AccountId) -> bool {
		OperatorsProfile::<T>::get(operator)
			.map_or(false, |profile| profile.services.is_empty() && profile.blueprints.is_empty())
	}
}

#[cfg(feature = "runtime-benchmarks")]
pub struct BenchmarkingOperatorDelegationManager<T: crate::Config, Balance: Default, AssetId>(
	core::marker::PhantomData<(T, Balance, AssetId)>,
);

#[cfg(feature = "runtime-benchmarks")]
impl<T: crate::Config, Balance: Default, AssetId>
	tangle_primitives::traits::MultiAssetDelegationInfo<T::AccountId, Balance>
	for BenchmarkingOperatorDelegationManager<T, Balance, AssetId>
{
	type AssetId = AssetId;

	fn get_current_round() -> tangle_primitives::types::RoundIndex {
		Default::default()
	}

	fn is_operator(_operator: &T::AccountId) -> bool {
		true
	}

	fn is_operator_active(_operator: &T::AccountId) -> bool {
		true
	}

	fn get_operator_stake(_operator: &T::AccountId) -> Balance {
		Default::default()
	}

	fn get_total_delegation_by_asset_id(
		_operator: &T::AccountId,
		_asset_id: &Self::AssetId,
	) -> Balance {
		Default::default()
	}
}
