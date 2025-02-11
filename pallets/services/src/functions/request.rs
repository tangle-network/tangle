use crate::{
	BalanceOf, Config, Error, Event, MaxFieldsOf, MaxOperatorsPerServiceOf, MaxPermittedCallersOf,
	NextServiceRequestId, Pallet, ServiceRequests, StagingServicePayments,
};
use frame_support::{
	pallet_prelude::*,
	traits::{fungibles::Mutate, tokens::Preservation, Currency, ExistenceRequirement},
	BoundedVec,
};
use frame_system::pallet_prelude::*;
use sp_core::H160;
use sp_runtime::traits::Zero;
use sp_std::vec::Vec;
use tangle_primitives::{
	services::{
		ApprovalState, Asset, AssetSecurityRequirement, EvmAddressMapping, Field, MembershipModel,
		ServiceRequest, StagingServicePayment,
	},
	Account,
};

impl<T: Config> Pallet<T> {
	/// Request a new service using a blueprint and specified operators.
	///
	/// # Arguments
	///
	/// * `caller` - The account requesting the service
	/// * `evm_origin` - Optional EVM address for ERC20 payments
	/// * `blueprint_id` - The identifier of the blueprint to use
	/// * `permitted_callers` - Accounts allowed to call the service
	/// * `operators` - List of operators that will run the service
	/// * `request_args` - Blueprint initialization arguments
	/// * `native_asset_requirement` - Native asset requirement for the service
	/// * `security_requirements` - Non-native asset requirements for the service
	/// * `ttl` - Time-to-live in blocks for the service request
	/// * `payment_asset` - Asset used for payment (native, custom or ERC20)
	/// * `value` - Payment amount for the service
	#[allow(clippy::too_many_arguments)]
	pub(crate) fn do_request(
		caller: T::AccountId,
		evm_origin: Option<H160>,
		blueprint_id: u64,
		permitted_callers: Vec<T::AccountId>,
		operators: Vec<T::AccountId>,
		request_args: Vec<Field<T::Constraints, T::AccountId>>,
		mut security_requirements: Vec<AssetSecurityRequirement<T::AssetId>>,
		ttl: BlockNumberFor<T>,
		payment_asset: Asset<T::AssetId>,
		value: BalanceOf<T>,
		membership_model: MembershipModel,
	) -> Result<u64, DispatchError> {
		let (_, blueprint) = Self::blueprints(blueprint_id)?;

		blueprint.type_check_request(&request_args).map_err(Error::<T>::TypeCheck)?;
		// ensure we at least have one asset and all assets are unique
		ensure!(!security_requirements.is_empty(), Error::<T>::NoAssetsProvided);
		ensure!(
			security_requirements
				.iter()
				.map(|req| &req.asset)
				.collect::<sp_std::collections::btree_set::BTreeSet<_>>()
				.len() == security_requirements.len(),
			Error::<T>::DuplicateAsset
		);

		// Check if native asset exists in requirements
		let has_native_asset =
			security_requirements.iter().any(|req| req.asset == Asset::Custom(Zero::zero()));

		// If native asset not found, append it with minimum requirements
		if !has_native_asset {
			security_requirements.push(AssetSecurityRequirement {
				asset: Asset::Custom(Zero::zero()),
				min_exposure_percent: T::MinimumNativeSecurityRequirement::get(),
				max_exposure_percent: T::MinimumNativeSecurityRequirement::get(),
			});
		}

		// Get native asset requirement for validation
		let native_asset_requirement = security_requirements
			.iter()
			.find(|req| req.asset == Asset::Custom(Zero::zero()))
			.ok_or(Error::<T>::NoNativeAsset)?;

		ensure!(
			native_asset_requirement.min_exposure_percent
				>= T::MinimumNativeSecurityRequirement::get(),
			Error::<T>::NativeAssetExposureTooLow
		);

		let security_requirements = BoundedVec::try_from(security_requirements)
			.map_err(|_| Error::<T>::MaxAssetsPerServiceExceeded)?;

		let mut preferences = Vec::new();
		let mut pending_approvals = Vec::new();
		for provider in &operators {
			let prefs = Self::operators(blueprint_id, provider)?;
			pending_approvals.push(provider.clone());
			preferences.push(prefs);
		}

		let mut native_value = Zero::zero();
		let request_id = Self::next_service_request_id();

		if value != Zero::zero() {
			// Payment transfer
			let refund_to = match payment_asset.clone() {
				// Handle the case of native currency.
				Asset::Custom(asset_id) if asset_id == Zero::zero() => {
					T::Currency::transfer(
						&caller,
						&Self::pallet_account(),
						value,
						ExistenceRequirement::KeepAlive,
					)?;
					native_value = value;
					Account::id(caller.clone())
				},
				Asset::Custom(asset_id) => {
					T::Fungibles::transfer(
						asset_id,
						&caller,
						&Self::pallet_account(),
						value,
						Preservation::Preserve,
					)?;
					Account::id(caller.clone())
				},
				Asset::Erc20(token) => {
					// origin check.
					let evm_origin = evm_origin.ok_or(Error::<T>::MissingEVMOrigin)?;
					let mapped_origin = T::EvmAddressMapping::into_account_id(evm_origin);
					ensure!(mapped_origin == caller, DispatchError::BadOrigin);
					let (success, _weight) =
						Self::erc20_transfer(token, evm_origin, Self::pallet_evm_account(), value)
							.map_err(|_| Error::<T>::OnErc20TransferFailure)?;
					ensure!(success, Error::<T>::ERC20TransferFailed);
					Account::from(evm_origin)
				},
			};

			// Save the payment information for the service request.
			let payment = StagingServicePayment {
				request_id,
				refund_to,
				asset: payment_asset.clone(),
				amount: value,
			};

			StagingServicePayments::<T>::insert(request_id, payment);
		}

		let (allowed, _weight) = Self::on_request_hook(
			&blueprint,
			blueprint_id,
			&caller,
			request_id,
			&preferences,
			&request_args,
			&permitted_callers,
			ttl,
			payment_asset,
			value,
			native_value,
		)
		.map_err(|_| Error::<T>::OnRequestFailure)?;

		ensure!(allowed, Error::<T>::InvalidRequestInput);

		let permitted_callers =
			BoundedVec::<_, MaxPermittedCallersOf<T>>::try_from(permitted_callers)
				.map_err(|_| Error::<T>::MaxPermittedCallersExceeded)?;
		let operators = pending_approvals
			.iter()
			.cloned()
			.map(|v| (v, ApprovalState::Pending))
			.collect::<Vec<_>>();

		let args = BoundedVec::<_, MaxFieldsOf<T>>::try_from(request_args)
			.map_err(|_| Error::<T>::MaxFieldsExceeded)?;

		let operators_with_approval_state =
			BoundedVec::<_, MaxOperatorsPerServiceOf<T>>::try_from(operators)
				.map_err(|_| Error::<T>::MaxServiceProvidersExceeded)?;

		ServiceRequests::<T>::insert(
			request_id,
			ServiceRequest {
				blueprint: blueprint_id,
				owner: caller.clone(),
				security_requirements: security_requirements.clone(),
				ttl,
				args,
				permitted_callers,
				operators_with_approval_state,
				membership_model,
			},
		);
		NextServiceRequestId::<T>::set(request_id.saturating_add(1));

		Self::deposit_event(Event::ServiceRequested {
			owner: caller,
			request_id,
			blueprint_id,
			pending_approvals,
			approved: Default::default(),
			security_requirements,
		});

		Ok(request_id)
	}
}
