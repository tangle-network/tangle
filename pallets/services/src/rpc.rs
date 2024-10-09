use itertools::Itertools;
use sp_std::{collections::btree_map::BTreeMap, vec::Vec};

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
		let profile = Self::operator_profile(operator)?;
		let mut result = Vec::with_capacity(profile.services.len());
		let services = profile
			.services
			.into_iter()
			.flat_map(Self::services)
			.chunk_by(|service| service.blueprint);

		let blueprints = profile
			.blueprints
			.into_iter()
			.flat_map(|id| Self::blueprints(id).map(|(_, b)| (id, b)))
			.collect::<BTreeMap<_, _>>();

		for (blueprint_id, services) in services.into_iter() {
			match blueprints.get(&blueprint_id) {
				Some(blueprint) => {
					result.push(RpcServicesWithBlueprint {
						blueprint_id,
						blueprint: blueprint.clone(),
						services: services.collect(),
					});
				},
				None => return Err(Error::<T>::BlueprintNotFound),
			}
		}
		Ok(result)
	}
}
