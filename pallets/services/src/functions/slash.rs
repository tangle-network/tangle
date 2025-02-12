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

		// Find operator's exposure percentage for this service and slash it
		let offender_security_commitments = service
			.operator_security_commitments
			.iter()
			.find(|(op, _)| op == offender)
			.map(|(_, commitments)| commitments)
			.ok_or(Error::<T>::OffenderNotOperator)?;
		let native_security_commitment = offender_security_commitments
			.iter()
			.find(|c| c.asset == Asset::Custom(Zero::zero()))
			.map(|c| c.exposure_percent)
			.ok_or(Error::<T>::NoNativeAsset)?;

		// Calculate operator's own slash in native currency
		let exposed_stake = native_security_commitment.mul_floor(total_stake);
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

		println!("eligible_delegators: {:#?}", eligible_delegators);

		// Calculate delegator slashes per asset
		let mut delegator_slashes = Vec::new();

		// Slash each delegator for each exposed asset
		for commitment in offender_security_commitments {
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

	/// Apply a slash to an operator's stake
	fn apply_operator_slash(
		operator: &T::AccountId,
		slash_amount: BalanceOf<T>,
		service_id: u64,
		blueprint_id: u64,
		era: u32,
	) -> Result<Weight, DispatchError> {
		let mut weight: Weight = Weight::zero();

		// Transfer native slashed amount to treasury
		T::Currency::unreserve(operator, slash_amount);
		weight += T::DbWeight::get().reads(1_u64);
		weight += T::DbWeight::get().writes(1_u64);

		T::Currency::transfer(
			operator,
			&T::SlashRecipient::get(),
			slash_amount,
			ExistenceRequirement::AllowDeath,
		)?;
		weight += T::DbWeight::get().reads(1_u64);
		weight += T::DbWeight::get().writes(1_u64);

		// Emit event for native token slash
		Self::deposit_event(Event::OperatorSlashed {
			operator: operator.clone(),
			amount: slash_amount,
			service_id,
			blueprint_id,
			era,
		});

		Ok(weight)
	}

	/// Apply a slash for native asset delegations (both nominated and non-nominated)
	fn apply_native_delegator_slash(
		delegator: &T::AccountId,
		operator: &T::AccountId,
		slash_amount: BalanceOf<T>,
		service_id: u64,
		blueprint_id: u64,
		era: u32,
	) -> Result<Weight, DispatchError> {
		let mut weight: Weight = Weight::zero();

		// Handle native asset slashing
		let (non_nominated_slash, nominated_slash) =
			Self::handle_native_asset_slash(delegator, operator, slash_amount)?;

		// Handle non-nominated slash first (direct transfer)
		if !non_nominated_slash.is_zero() {
			T::Currency::transfer(
				&Self::pallet_account(),
				&T::SlashRecipient::get(),
				non_nominated_slash,
				ExistenceRequirement::AllowDeath,
			)?;
			weight += T::DbWeight::get().reads(1_u64);
			weight += T::DbWeight::get().writes(1_u64);
		}

		// Handle nominated slash (requires special handling through staking system)
		if !nominated_slash.is_zero() {
			Self::deposit_event(Event::NominatedSlash {
				delegator: delegator.clone(),
				operator: operator.clone(),
				amount: nominated_slash,
				service_id,
				blueprint_id,
				era,
			});
		}

		Ok(weight)
	}

	/// Apply a slash for non-native asset delegations (custom assets and ERC20)
	fn apply_non_native_delegator_slash(
		delegator: &T::AccountId,
		asset: Asset<T::AssetId>,
		slash_amount: BalanceOf<T>,
		service_id: u64,
		blueprint_id: u64,
		era: u32,
	) -> Result<Weight, DispatchError> {
		let mut weight: Weight = Weight::zero();

		match asset.clone() {
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
			asset,
			service_id,
			blueprint_id,
			era,
		});

		Ok(weight)
	}

	/// Apply a slash to both operator and delegators
	pub fn apply_slash(
		unapplied_slash: UnappliedSlash<T::AccountId, BalanceOf<T>, T::AssetId>,
	) -> Result<Weight, DispatchError> {
		let mut weight: Weight = Weight::zero();

		// Notify the multi-asset delegation system about the operator slash
		// This call will also slash all delegators for the operator. Note, that
		// the `SlashManager` is solely responsible for updating the storage of
		// the operator and delegators. The asset transfers are handled by this function.
		let slash_operator_weight = T::SlashManager::slash_operator(&unapplied_slash)?;
		weight += slash_operator_weight;

		// Apply operator slash
		weight += Self::apply_operator_slash(
			&unapplied_slash.operator,
			unapplied_slash.own,
			unapplied_slash.service_id,
			unapplied_slash.blueprint_id,
			unapplied_slash.era,
		)?;

		// Process all delegator slashes
		for (delegator, asset, slash_amount) in unapplied_slash.clone().others {
			match asset {
				Asset::Custom(asset_id) if asset_id == Zero::zero() => {
					// Handle native asset slashing
					weight += Self::apply_native_delegator_slash(
						&delegator,
						&unapplied_slash.operator,
						slash_amount,
						unapplied_slash.service_id,
						unapplied_slash.blueprint_id,
						unapplied_slash.era,
					)?;
				},
				_ => {
					// Handle non-native asset slashing
					weight += Self::apply_non_native_delegator_slash(
						&delegator,
						asset,
						slash_amount,
						unapplied_slash.service_id,
						unapplied_slash.blueprint_id,
						unapplied_slash.era,
					)?;
				},
			}
		}

		Ok(weight)
	}

	/// Helper function to handle native asset slashing for a delegator.
	/// This function prioritizes slashing from non-nominated delegations before touching nominated ones.
	///
	/// # Arguments
	/// * `delegator` - The account ID of the delegator being slashed
	/// * `operator` - The operator associated with the slash
	/// * `total_slash_amount` - The total amount to be slashed
	///
	/// # Returns
	/// A tuple containing:
	/// - The amount to slash from non-nominated delegation
	/// - The amount to slash from nominated delegation
	/// - Any remaining unslashable amount
	fn handle_native_asset_slash(
		delegator: &T::AccountId,
		operator: &T::AccountId,
		total_slash_amount: BalanceOf<T>,
	) -> Result<(BalanceOf<T>, BalanceOf<T>), DispatchError> {
		// TODO: Implement native asset slashing logic for delegators
		//
		// This function should:
		// 1. Query the delegator's non-nominated stake from the delegation manager
		// 2. Calculate how much can be slashed from non-nominated stake
		// 3. If non-nominated stake is insufficient, calculate remaining amount to slash from nominated stake
		// 4. Return tuple of (non_nominated_slash, nominated_slash)
		//
		// The implementation should ensure:
		// - We slash non-nominated stake first before touching nominated stake
		// - We don't slash more than the total_slash_amount
		// - We handle zero balances and edge cases gracefully
		// - We respect any minimum stake requirements from the delegation system

		let non_nominated_slash = BalanceOf::<T>::zero();
		let nominated_slash = BalanceOf::<T>::zero();

		Ok((non_nominated_slash, nominated_slash))
	}
}
