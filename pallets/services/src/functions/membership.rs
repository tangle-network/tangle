use crate::{Config, Error, Instances, Pallet};
use frame_support::pallet_prelude::*;
use sp_runtime::Percent;
use sp_std::vec::Vec;
use tangle_primitives::services::{
	AssetSecurityCommitment, MembershipModel, OperatorPreferences, ServiceBlueprint,
};

impl<T: Config> Pallet<T> {
	/// Implementation of join_service extrinsic
	pub(crate) fn do_join_service(
		blueprint: &ServiceBlueprint<T::Constraints>,
		blueprint_id: u64,
		instance_id: u64,
		operator: &T::AccountId,
		preferences: &OperatorPreferences,
		native_asset_exposure: Percent,
		non_native_asset_exposures: Vec<AssetSecurityCommitment<T::AssetId>>,
	) -> DispatchResult {
		// Get service instance
		let instance = Instances::<T>::get(instance_id)?;

		// Validate membership model
		match instance.membership_model {
			MembershipModel::Fixed { .. } => {
				return Err(Error::<T>::DynamicMembershipNotSupported.into())
			},
			MembershipModel::Dynamic { min_operators, max_operators } => {
				ensure!(min_operators > 0, Error::<T>::InvalidMinOperators);
				// Check max operators if set
				if let Some(max) = max_operators {
					ensure!(min_operators < max, Error::<T>::InvalidMinOperators);
					ensure!(
						instance.native_asset_security.len() < max as usize,
						Error::<T>::MaxOperatorsReached
					);
				}
			},
		}

		// Check if operator can join via blueprint contract
		let (can_join, _) =
			Self::can_join_hook(blueprint, blueprint_id, instance_id, operator, preferences)
				.map_err(|e| {
					log::error!("Can join hook failed: {:?}", e);
					Error::<T>::OnCanJoinFailure
				})?;
		ensure!(can_join, Error::<T>::JoinRejected);

		// Add operator to instance
		Instances::<T>::try_mutate(instance_id, |maybe_instance| -> DispatchResult {
			let instance = maybe_instance.as_mut().map_err(|e| {
				log::error!("Service not found: {:?}", e);
				Error::<T>::ServiceNotFound
			})?;
			instance
				.native_asset_security
				.try_push((operator.clone(), native_asset_exposure))
				.map_err(|e| {
					log::error!("Failed to push native asset security: {:?}", e);
					Error::<T>::MaxOperatorsReached
				})?;
			instance
				.non_native_asset_security
				.try_push((
					operator.clone(),
					BoundedVec::try_from(non_native_asset_exposures.clone()).map_err(|e| {
						log::error!("Failed to convert non-native asset exposures: {:?}", e);
						Error::<T>::MaxOperatorsReached
					})?,
				))
				.map_err(|e| {
					log::error!("Failed to push non-native asset security: {:?}", e);
					Error::<T>::MaxOperatorsReached
				})?;

			Ok(())
		})?;

		// Notify blueprint
		Self::on_operator_joined_hook(blueprint, blueprint_id, instance_id, operator, preferences)
			.map_err(|e| {
				log::error!("Operator joined hook failed: {:?}", e);
				Error::<T>::OnOperatorJoinFailure
			})?;

		Ok(())
	}

	/// Implementation of leave_service extrinsic
	pub(crate) fn do_leave_service(
		blueprint: &ServiceBlueprint<T::Constraints>,
		blueprint_id: u64,
		instance_id: u64,
		operator: &T::AccountId,
	) -> DispatchResult {
		// Get service instance
		let instance = Instances::<T>::get(instance_id)?;

		// Validate membership model
		match instance.membership_model {
			MembershipModel::Fixed { .. } => {
				return Err(Error::<T>::DynamicMembershipNotSupported.into())
			},
			MembershipModel::Dynamic { min_operators, .. } => {
				// Ensure minimum operators maintained
				ensure!(
					instance.native_asset_security.len() > min_operators as usize,
					Error::<T>::InsufficientOperators
				);
			},
		}

		// Check if operator can leave via blueprint contract
		let (can_leave, _) = Self::can_leave_hook(blueprint, blueprint_id, instance_id, operator)
			.map_err(|e| {
			log::error!("Can leave hook failed: {:?}", e);
			Error::<T>::OnCanLeaveFailure
		})?;
		ensure!(can_leave, Error::<T>::LeaveRejected);

		// Remove operator from instance
		Instances::<T>::try_mutate(instance_id, |maybe_instance| -> DispatchResult {
			let instance = maybe_instance.as_mut().map_err(|e| {
				log::error!("Service not found: {:?}", e);
				Error::<T>::ServiceNotFound
			})?;
			instance.native_asset_security.retain(|(op, _)| op != operator);
			instance.non_native_asset_security.retain(|(op, _)| op != operator);
			Ok(())
		})?;

		// Notify blueprint
		Self::on_operator_left_hook(blueprint, blueprint_id, instance_id, operator).map_err(
			|e| {
				log::error!("Operator left hook failed: {:?}", e);
				Error::<T>::OnOperatorLeaveFailure
			},
		)?;

		Ok(())
	}
}
