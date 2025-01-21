use super::*;
use crate::types::BalanceOf;
use frame_system::pallet_prelude::BlockNumberFor;
use sp_runtime::Percent;
use sp_std::{vec, vec::Vec};
use tangle_primitives::{
	rewards::UserDepositWithLocks,
	services::{Asset, Constraints},
	traits::ServiceManager,
	BlueprintId,
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
pub struct BenchmarkingOperatorDelegationManager<T: crate::Config, Balance: Default>(
	core::marker::PhantomData<(T, Balance)>,
);

#[cfg(feature = "runtime-benchmarks")]
impl<T: crate::Config, Balance: Default>
	tangle_primitives::traits::MultiAssetDelegationInfo<
		T::AccountId,
		Balance,
		BlockNumberFor<T>,
		T::AssetId,
	> for BenchmarkingOperatorDelegationManager<T, Balance>
{
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

	fn get_total_delegation_by_asset_id(_operator: &T::AccountId, _asset_id: &AssetId) -> Balance {
		Default::default()
	}

	fn get_delegators_for_operator(
		_operator: &T::AccountId,
	) -> Vec<(T::AccountId, Balance, Asset<AssetId>)> {
		Vec::new()
	}

	fn has_delegator_selected_blueprint(
		_delegator: &T::AccountId,
		_operator: &T::AccountId,
		_blueprint_id: BlueprintId,
	) -> bool {
		true // For benchmarking, always return true
	}

	fn slash_operator(
		_operator: &T::AccountId,
		_blueprint_id: BlueprintId,
		_service_id: InstanceId,
		_percentage: Percent,
	) {
		// For benchmarking, do nothing
	}

	fn get_user_deposit_with_locks(
		_who: &T::AccountId,
		_asset_id: Asset<AssetId>,
	) -> Option<UserDepositWithLocks<Balance, BlockNumberFor<T>>> {
		None
	}
}
