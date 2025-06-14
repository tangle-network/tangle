use frame_system::pallet_prelude::BlockNumberFor;
use sp_std::vec::Vec;
use tangle_primitives::services::*;

use super::*;

impl<T: Config> Pallet<T> {
	#[allow(clippy::type_complexity)]
	pub fn services_with_blueprints_by_operator(
		operator: T::AccountId,
	) -> Result<
		Vec<RpcServicesWithBlueprint<T::Constraints, T::AccountId, BlockNumberFor<T>, T::AssetId>>,
		Error<T>,
	> {
		// First get the operator's profile to know which blueprints they're registered for
		let profile = Self::operator_profile(&operator)?;

		// Get the operator's blueprints
		let blueprint_ids = profile.blueprints;

		// Create a map to group services by blueprint
		let mut blueprint_services_map: sp_std::collections::btree_map::BTreeMap<
			u64,
			Vec<Service<T::Constraints, T::AccountId, BlockNumberFor<T>, T::AssetId>>,
		> = sp_std::collections::btree_map::BTreeMap::new();

		// Iterate through all active service instances to find ones where the operator is
		// participating
		for (_service_id, service) in Instances::<T>::iter() {
			// Check if this service is for a blueprint the operator is registered for
			if blueprint_ids.contains(&service.blueprint) {
				// Check if the operator is one of the operators providing security for this service
				if service.operator_security_commitments.iter().any(|(op, _)| op == &operator) {
					// Add this service to the appropriate blueprint group
					blueprint_services_map
						.entry(service.blueprint)
						.or_insert_with(Vec::new)
						.push(service);
				}
			}
		}

		// Convert the map to the expected result format
		let mut result = Vec::new();
		for (blueprint_id, services) in blueprint_services_map {
			// Get the blueprint details
			let (_blueprint_id, blueprint) = Self::blueprints(blueprint_id)?;

			result.push(RpcServicesWithBlueprint { blueprint_id, blueprint, services });
		}

		Ok(result)
	}

	#[allow(clippy::type_complexity)]
	pub fn service_requests_with_blueprints_by_operator(
		operator: T::AccountId,
	) -> Result<
		Vec<(u64, ServiceRequest<T::Constraints, T::AccountId, BlockNumberFor<T>, T::AssetId>)>,
		Error<T>,
	> {
		// First we need to get the operator's profile to know which blueprints they're registered
		// for
		let profile = Self::operator_profile(&operator)?;

		// Get the operator's blueprints
		let blueprint_ids = profile.blueprints;

		// Iterate through all service requests to find the ones for the operator's blueprints
		// and where the operator is included in the requested operators
		let mut result = Vec::new();

		// We need to iterate through all service requests to check if operator is included
		for (request_id, request) in ServiceRequests::<T>::iter() {
			// Check if the service request is for a blueprint the operator is registered for
			if blueprint_ids.contains(&request.blueprint) {
				// Check if the operator is one of the requested operators with approval state
				if request.operators_with_approval_state.iter().any(|(op, _)| op == &operator) {
					// Only include pending requests (those that haven't been approved by all
					// operators yet)
					if !request.is_approved() {
						result.push((request_id, request));
					}
				}
			}
		}

		Ok(result)
	}
}
