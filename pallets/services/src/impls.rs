use super::*;
use crate::types::BalanceOf;
use frame_support::traits::OneSessionHandler;
use sp_std::{vec, vec::Vec};
use tangle_primitives::{BlueprintId, services::Constraints, traits::ServiceManager};

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
