use super::*;
use crate::types::BalanceOf;
use frame_support::traits::OneSessionHandler;
use sp_std::{vec, vec::Vec};
use tangle_primitives::{services::Constraints, traits::ServiceManager, BlueprintId};

#[cfg(feature = "runtime-benchmarks")]
use tangle_primitives::rewards::{AssetType, UserDepositWithLocks};
#[cfg(feature = "runtime-benchmarks")]
use tangle_primitives::services::Asset;

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

	type MaxRpcAddressLength = T::MaxRpcAddressLength;

	type MaxResourceNameLength = T::MaxResourceNameLength;
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

	fn has_active_services(operator: &T::AccountId) -> bool {
		OperatorsProfile::<T>::get(operator).is_ok_and(|profile| !profile.services.is_empty())
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
		AssetType<T::AssetId>,
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

	fn get_total_delegation_by_asset(
		_operator: &T::AccountId,
		_asset: &Asset<T::AssetId>,
	) -> Balance {
		Default::default()
	}

	fn get_delegators_for_operator(
		_operator: &T::AccountId,
	) -> Vec<(T::AccountId, Balance, Asset<T::AssetId>)> {
		Vec::new()
	}

	fn get_user_deposit_with_locks(
		_who: &T::AccountId,
		_asset: Asset<T::AssetId>,
	) -> Option<UserDepositWithLocks<Balance, BlockNumberFor<T>>> {
		None
	}

	fn get_user_deposit_by_asset_type(
		_who: &T::AccountId,
		_asset_type: tangle_primitives::rewards::AssetType<T::AssetId>,
	) -> Option<Balance> {
		None
	}
}

impl<T: Config> sp_runtime::BoundToRuntimeAppPublic for Pallet<T> {
	type Public = T::RoleKeyId;
}

impl<T: Config> OneSessionHandler<T::AccountId> for Pallet<T> {
	type Key = T::RoleKeyId;

	#[allow(clippy::multiple_bound_locations)]
	fn on_genesis_session<'a, I: 'a>(_validators: I)
	where
		I: Iterator<Item = (&'a T::AccountId, T::RoleKeyId)>,
	{
	}

	#[allow(clippy::multiple_bound_locations)]
	fn on_new_session<'a, I: 'a>(_changed: bool, _validators: I, _queued_validators: I)
	where
		I: Iterator<Item = (&'a T::AccountId, T::RoleKeyId)>,
	{
	}

	fn on_disabled(_i: u32) {
		// ignore
	}

	// Distribute the inflation rewards
	fn on_before_session_ending() {
		// ignore
	}
}
