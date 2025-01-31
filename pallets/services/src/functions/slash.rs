use crate::{types::BalanceOf, Config, Error, Event, Pallet};
use frame_support::{
	ensure,
	pallet_prelude::*,
	traits::{
		fungibles::Mutate, tokens::Preservation, Currency, ExistenceRequirement, Get,
		ReservableCurrency,
	},
};
use frame_system::pallet_prelude::BlockNumberFor;
use sp_runtime::{traits::Zero, Percent};
use sp_std::{vec, vec::Vec};
use tangle_primitives::{
	services::{Asset, EvmAddressMapping, Service, UnappliedSlash},
	traits::{MultiAssetDelegationInfo, SlashManager},
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
	/// * `reporter` - The account ID of the reporter
	/// * `service` - The service instance where the slash occurred
	/// * `offender` - The operator being slashed
	/// * `slash_percent` - The percentage of exposed stake to slash
	pub(crate) fn calculate_slash(
		reporter: &T::AccountId,
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

		// Get all delegators for this operator and filter by blueprint selection upfront
		let all_delegators = T::OperatorDelegationManager::get_delegators_for_operator(offender);
		let eligible_delegators: Vec<_> = all_delegators
			.into_iter()
			.filter(|(delegator, _, _)| {
				T::OperatorDelegationManager::has_delegator_selected_blueprint(
					delegator,
					offender,
					service.blueprint,
				)
			})
			.collect();

		// Get the asset commitments for the offending operator
		let offender_commitments = service
			.non_native_asset_security
			.iter()
			.find(|(op, _)| op == offender)
			.map(|(_, commitments)| commitments)
			.ok_or(Error::<T>::OffenderNotOperator)?;

		// Calculate delegator slashes per asset
		let mut delegator_slashes = Vec::new();

		// For each asset commitment of the offending operator
		for commitment in offender_commitments {
			let asset = &commitment.asset;
			let asset_exposure = commitment.exposure_percent;

			// Process only delegators who delegated this specific asset
			let asset_delegators = eligible_delegators.iter().filter(|(_, _, delegator_asset)| {
				match (delegator_asset, asset) {
					(Asset::Custom(d_asset), Asset::Custom(s_asset)) => d_asset == s_asset,
					(Asset::Erc20(d_asset), Asset::Erc20(s_asset)) => d_asset.0 == s_asset.0,
					_ => false,
				}
			});

			// Calculate slashes for eligible delegators
			for (delegator, stake, _) in asset_delegators {
				let exposed_delegation = asset_exposure.mul_floor(*stake);
				let slash_amount = slash_percent.mul_floor(exposed_delegation);

				if !slash_amount.is_zero() {
					delegator_slashes.push((delegator.clone(), asset.clone(), slash_amount));
				}
			}
		}

		Ok(UnappliedSlash {
			era: T::OperatorDelegationManager::get_current_round(),
			blueprint_id: service.blueprint,
			service_id: service.id,
			operator: offender.clone(),
			own: own_slash,
			others: delegator_slashes,
			reporters: vec![reporter.clone()],
		})
	}

	/// Slash an operator and their delegators for a specific service instance.
	/// The slashing is applied according to the operator's security commitments in the service.
	///
	/// # Arguments
	/// * `unapplied_slash` - The unapplied slash record to apply
	pub fn apply_slash(
		unapplied_slash: UnappliedSlash<T::AccountId, BalanceOf<T>, T::AssetId>,
	) -> Result<Weight, DispatchError> {
		let mut weight: Weight = Weight::zero();
		// Notify the multi-asset delegation system about the operator slash
		let slash_operator_weight = T::SlashManager::slash_operator(&unapplied_slash)?;
		weight += slash_operator_weight;

		// Transfer native slashed amount to treasury
		T::Currency::unreserve(&unapplied_slash.operator, unapplied_slash.own);
		weight += T::DbWeight::get().reads(1_u64);
		weight += T::DbWeight::get().writes(1_u64);

		T::Currency::transfer(
			&unapplied_slash.operator,
			&T::SlashRecipient::get(),
			unapplied_slash.own,
			ExistenceRequirement::AllowDeath,
		)?;
		weight += T::DbWeight::get().reads(1_u64);
		weight += T::DbWeight::get().writes(1_u64);

		// Emit event for native token slash
		Self::deposit_event(Event::OperatorSlashed {
			operator: unapplied_slash.operator.clone(),
			amount: unapplied_slash.own,
			service_id: unapplied_slash.service_id,
			blueprint_id: unapplied_slash.blueprint_id,
			era: unapplied_slash.era,
		});

		// Process all delegator slashes
		for (delegator, asset, slash_amount) in unapplied_slash.clone().others {
			// Notify multi-asset delegation system about the delegator slash
			let slash_delegator_weight =
				T::SlashManager::slash_delegator(&unapplied_slash, &delegator)?;
			weight += slash_delegator_weight;

			// Transfer slashed assets to treasury
			match asset {
				Asset::Custom(asset_id) => {
					T::Fungibles::transfer(
						asset_id,
						&Self::pallet_account(),
						&T::SlashRecipient::get(),
						slash_amount,
						Preservation::Expendable,
					)?;
					weight += T::DbWeight::get().reads(1_u64);
					weight += T::DbWeight::get().writes(1_u64);
				},
				Asset::Erc20(token) => {
					let treasury_evm = T::EvmAddressMapping::into_address(T::SlashRecipient::get());
					let (success, _) = Self::erc20_transfer(
						token,
						Self::pallet_evm_account(),
						treasury_evm,
						slash_amount,
					)
					.map_err(|_| Error::<T>::ERC20TransferFailed)?;
					ensure!(success, Error::<T>::ERC20TransferFailed);
					weight += T::DbWeight::get().reads(1_u64);
					weight += T::DbWeight::get().writes(1_u64);
				},
			}

			// Emit event for delegator slash
			Self::deposit_event(Event::DelegatorSlashed {
				delegator: delegator.clone(),
				amount: slash_amount,
				service_id: unapplied_slash.service_id,
				blueprint_id: unapplied_slash.blueprint_id,
				era: unapplied_slash.era,
			});
		}

		Ok(weight)
	}
}
