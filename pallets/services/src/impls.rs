use super::*;
use crate::types::BalanceOf;
#[cfg(not(feature = "std"))]
use alloc::vec::Vec;
use sp_runtime::Percent;
use sp_std::vec;
#[cfg(feature = "std")]
use std::vec::Vec;
use tangle_primitives::{
	rewards::UserDepositWithLocks, services::Constraints, traits::ServiceManager, BlueprintId,
};

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
			.map_or(true, |profile| profile.services.is_empty() && profile.blueprints.is_empty())
	}

	fn get_blueprints_by_operator(operator: &T::AccountId) -> Vec<BlueprintId> {
		OperatorsProfile::<T>::get(operator)
			.map_or(vec![], |profile| profile.blueprints.into_iter().collect())
	}
}

#[cfg(feature = "runtime-benchmarks")]
pub struct BenchmarkingOperatorDelegationManager<T: crate::Config, Balance: Default, AssetId>(
	core::marker::PhantomData<(T, Balance, AssetId)>,
);

#[cfg(feature = "runtime-benchmarks")]
impl<T: crate::Config, Balance: Default, AssetId>
	tangle_primitives::traits::MultiAssetDelegationInfo<T::AccountId, Balance, BlockNumberFor<T>>
	for BenchmarkingOperatorDelegationManager<T, Balance, AssetId>
{
	fn get_delegators_for_operator(
		_operator: &T::AccountId,
	) -> Vec<(T::AccountId, Balance, tangle_primitives::services::Asset<AssetId>)> {
		Vec::new()
	}

	fn slash_operator(_operator: &T::AccountId, _amount: u64, _slash_rate: Percent) {
		// No-op for benchmarking
	}

	fn get_user_deposit_with_locks(
		_user: &T::AccountId,
		_asset: tangle_primitives::services::Asset<AssetId>,
	) -> Option<UserDepositWithLocks<Balance, BlockNumberFor<T>>> {
		None
	}

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
		_asset: &tangle_primitives::services::Asset<Self::AssetId>,
	) -> Balance {
		Default::default()
	}
}
