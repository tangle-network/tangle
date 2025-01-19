use crate::{
	types::{BalanceOf, UnappliedSlash},
	Config, Error, Pallet,
};
use frame_support::pallet_prelude::*;
use frame_system::pallet_prelude::BlockNumberFor;
use sp_runtime::{traits::Zero, Percent};
use sp_std::vec::Vec;
use tangle_primitives::{
	services::{Asset, Service},
	traits::MultiAssetDelegationInfo,
};

impl<T: Config> Pallet<T> {
	/// Calculates and creates an unapplied slash for an operator and their delegators.
	///
	/// This function:
	/// 1. Calculates the operator's native currency slash based on their exposure
	/// 2. For each asset required by the service:
	///    - Identifies delegators who selected this blueprint
	///    - Calculates slashes based on delegator exposure and asset requirements
	/// 3. Creates an UnappliedSlash record for later processing
	///
	/// # Arguments
	/// * `service` - The service instance where the slash occurred
	/// * `offender` - The operator being slashed
	/// * `slash_percent` - The percentage of exposed stake to slash
	pub(crate) fn calculate_slash(
		service: &Service<T::Constraints, T::AccountId, BlockNumberFor<T>, T::AssetId>,
		offender: &T::AccountId,
		slash_percent: Percent,
	) -> Result<UnappliedSlash<T::AccountId, BalanceOf<T>, T::AssetId>, DispatchError> {
		// Get operator's total stake and calculate their native currency slash
		let total_stake = T::OperatorDelegationManager::get_operator_stake(offender);

		// Find operator's exposure percentage for this service
		let operator_exposure = service
			.native_asset_security
			.iter()
			.find(|(op, _)| op == offender)
			.map(|(_, exposure)| *exposure)
			.ok_or(Error::<T>::OffenderNotOperator)?;

		// Calculate operator's own slash in native currency
		let exposed_stake = operator_exposure.mul_floor(total_stake);
		let own_slash = slash_percent.mul_floor(exposed_stake);

		// Calculate delegator slashes per asset
		let mut delegator_slashes = Vec::new();

		// Get all delegators for this operator
		let delegators = T::OperatorDelegationManager::get_delegators_for_operator(offender);

		// Get the asset commitments for the offending operator
		let offender_commitments = service
			.non_native_asset_security
			.iter()
			.find(|(op, _)| op == offender)
			.map(|(_, commitments)| commitments)
			.ok_or(Error::<T>::OffenderNotOperator)?;

		// For each asset commitment of the offending operator
		for commitment in offender_commitments {
			let asset = &commitment.asset;
			let asset_exposure = commitment.exposure_percent;

			// Calculate slashes for delegators who selected this blueprint
			for (delegator, stake, delegator_asset) in delegators.iter() {
				// Only slash if the delegator's asset matches and they selected this blueprint
				let should_slash = match (delegator_asset, asset) {
					(Asset::Custom(d_asset), Asset::Custom(s_asset)) => d_asset == s_asset,
					(Asset::Erc20(d_asset), Asset::Erc20(s_asset)) => d_asset.0 == s_asset.0,
					_ => false,
				}
					&& T::OperatorDelegationManager::has_delegator_selected_blueprint(
						delegator,
						offender,
						service.blueprint,
					);

				if should_slash {
					let exposed_delegation = asset_exposure.mul_floor(*stake);
					let slash_amount = slash_percent.mul_floor(exposed_delegation);

					if !slash_amount.is_zero() {
						delegator_slashes.push((delegator.clone(), asset.clone(), slash_amount));
					}
				}
			}
		}

		Ok(UnappliedSlash {
			service_id: service.id,
			operator: offender.clone(),
			own: own_slash,
			others: delegator_slashes,
			reporters: Vec::new(),
		})
	}
}
