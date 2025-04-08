use crate::{Config, Error, Instances, Pallet};
use frame_support::pallet_prelude::*;
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
		security_commitments: Vec<AssetSecurityCommitment<T::AssetId>>,
	) -> DispatchResult {
		// Get service instance
		let instance = Instances::<T>::get(instance_id)?;

		// Validate membership model
		match instance.membership_model {
			MembershipModel::Fixed { .. } => {
				return Err(Error::<T>::DynamicMembershipNotSupported.into());
			},
			MembershipModel::Dynamic { max_operators, .. } => {
				// Check max operators if set
				if let Some(max) = max_operators {
					ensure!(
						instance.operator_security_commitments.len() < max as usize,
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
				.operator_security_commitments
				.try_push((
					operator.clone(),
					BoundedVec::try_from(security_commitments.clone()).map_err(|e| {
						log::error!("Failed to convert security commitments: {:?}", e);
						Error::<T>::MaxOperatorsReached
					})?,
				))
				.map_err(|e| {
					log::error!("Failed to push security commitments: {:?}", e);
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
				return Err(Error::<T>::DynamicMembershipNotSupported.into());
			},
			MembershipModel::Dynamic { min_operators, .. } => {
				// Ensure minimum operators maintained
				ensure!(
					instance.operator_security_commitments.len() > min_operators as usize,
					Error::<T>::TooFewOperators
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
			instance.operator_security_commitments.retain(|(op, _)| op != operator);
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
