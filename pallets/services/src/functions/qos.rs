use crate::{
	BalanceOf, BlueprintId, Config, DefaultHeartbeatInterval, DefaultHeartbeatThreshold,
	DefaultSlashingWindow, Error, Event, InstanceId, Instances, NextUnappliedSlashIndex, Pallet,
	UnappliedSlash, UnappliedSlashes,
};
use frame_support::pallet_prelude::*;
use frame_system::pallet_prelude::BlockNumberFor;
use sp_runtime::Percent;
use tangle_primitives::{services::ServiceBlueprint, traits::MultiAssetDelegationInfo};

impl<T: Config> Pallet<T> {
	/// Gets the heartbeat interval for a service instance.
	///
	/// This function retrieves the heartbeat interval for a service instance by calling
	/// the EVM hook to check if a custom value is specified, or using the blueprint default.
	///
	/// # Parameters
	/// * `blueprint` - The service blueprint
	/// * `blueprint_id` - The ID of the service blueprint
	/// * `service_id` - The ID of the service instance
	///
	/// # Returns
	/// * `Result<u64, DispatchError>` - The heartbeat interval in blocks
	pub(crate) fn get_heartbeat_interval(
		blueprint: &ServiceBlueprint<T::Constraints, BlockNumberFor<T>, BalanceOf<T>>,
		blueprint_id: u64,
		service_id: u64,
	) -> Result<BlockNumberFor<T>, DispatchError> {
		// Check if service exists
		ensure!(Instances::<T>::contains_key(service_id), Error::<T>::ServiceNotFound);

		// Call EVM hook
		let (use_default, interval) =
			Self::get_heartbeat_interval_hook(blueprint, blueprint_id, service_id).map_err(
				|e| {
					log::error!("Get heartbeat interval hook failed: {:?}", e);
					Error::<T>::GetHeartbeatIntervalFailure
				},
			)?;

		// If use_default is true, return the default interval
		if use_default {
			Ok(DefaultHeartbeatInterval::<T>::get())
		} else {
			Ok(BlockNumberFor::<T>::from(interval))
		}
	}

	/// Gets the heartbeat threshold for a service instance.
	///
	/// This function retrieves the heartbeat threshold percentage for a service instance by calling
	/// the EVM hook to check if a custom value is specified, or using a default value.
	///
	/// # Parameters
	/// * `blueprint` - The service blueprint
	/// * `blueprint_id` - The ID of the service blueprint
	/// * `service_id` - The ID of the service instance
	///
	/// # Returns
	/// * `Result<u8, DispatchError>` - The heartbeat threshold percentage (0-100)
	pub(crate) fn get_heartbeat_threshold(
		blueprint: &ServiceBlueprint<T::Constraints, BlockNumberFor<T>, BalanceOf<T>>,
		blueprint_id: u64,
		service_id: u64,
	) -> Result<u8, DispatchError> {
		// Check if service exists
		ensure!(Instances::<T>::contains_key(service_id), Error::<T>::ServiceNotFound);

		// Call EVM hook
		let (use_default, threshold) =
			Self::get_heartbeat_threshold_hook(blueprint, blueprint_id, service_id).map_err(
				|e| {
					log::error!("Get heartbeat threshold hook failed: {:?}", e);
					Error::<T>::GetHeartbeatThresholdFailure
				},
			)?;

		// If use_default is true, return the default threshold
		if use_default { Ok(DefaultHeartbeatThreshold::<T>::get()) } else { Ok(threshold) }
	}

	/// Gets the slashing window for a service instance.
	///
	/// This function retrieves the slashing window in blocks for a service instance by calling
	/// the EVM hook to check if a custom value is specified, or using a default value.
	///
	/// # Parameters
	/// * `blueprint` - The service blueprint
	/// * `blueprint_id` - The ID of the service blueprint
	/// * `service_id` - The ID of the service instance
	///
	/// # Returns
	/// * `Result<BlockNumberFor<T>, DispatchError>` - The slashing window in blocks
	pub(crate) fn get_slashing_window(
		blueprint: &ServiceBlueprint<T::Constraints, BlockNumberFor<T>, BalanceOf<T>>,
		blueprint_id: u64,
		service_id: u64,
	) -> Result<BlockNumberFor<T>, DispatchError> {
		// Check if service exists
		ensure!(Instances::<T>::contains_key(service_id), Error::<T>::ServiceNotFound);

		// Call EVM hook
		let (use_default, window) =
			Self::get_slashing_window_hook(blueprint, blueprint_id, service_id).map_err(|e| {
				log::error!("Get slashing window hook failed: {:?}", e);
				Error::<T>::GetSlashingWindowFailure
			})?;

		// If use_default is true, return the default window
		if use_default {
			Ok(DefaultSlashingWindow::<T>::get())
		} else {
			Ok(BlockNumberFor::<T>::from(window))
		}
	}

	/// Helper function to create and store a heartbeat slash
	pub(crate) fn create_heartbeat_slash(
		blueprint_id: BlueprintId,
		service_id: InstanceId,
		operator: T::AccountId,
		slash_percent: Percent,
	) {
		// Create an unapplied slash
		let unapplied_slash = UnappliedSlash {
			era: T::OperatorDelegationManager::get_current_round(),
			blueprint_id,
			service_id,
			operator: operator.clone(),
			slash_percent,
		};

		// Store the slash for later processing
		let index = Self::next_unapplied_slash_index();
		UnappliedSlashes::<T>::insert(unapplied_slash.era, index, unapplied_slash.clone());
		NextUnappliedSlashIndex::<T>::set(index.saturating_add(1));

		// Emit an event for the unapplied slash
		Self::deposit_event(Event::<T>::UnappliedSlash {
			index,
			operator,
			blueprint_id,
			service_id,
			slash_percent,
			era: unapplied_slash.era,
		});
	}
}
