use crate::{
	BalanceOf, Config, Error, Event, MaxAssetsPerServiceOf, MaxFieldsOf, MaxOperatorsPerServiceOf,
	MaxPermittedCallersOf, NextServiceRequestId, Pallet, ServiceRequests, StagingServicePayments,
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
	/// * `assets` - Required assets for the service
	/// * `ttl` - Time-to-live in blocks for the service request
	/// * `payment_asset` - Asset used for payment (native, custom or ERC20)
	/// * `value` - Payment amount for the service
	pub(crate) fn do_request(
		caller: T::AccountId,
		evm_origin: Option<H160>,
		blueprint_id: u64,
		permitted_callers: Vec<T::AccountId>,
		operators: Vec<T::AccountId>,
		request_args: Vec<Field<T::Constraints, T::AccountId>>,
		asset_security_requirements: Vec<AssetSecurityRequirement<T::AssetId>>,
		ttl: BlockNumberFor<T>,
		payment_asset: Asset<T::AssetId>,
		value: BalanceOf<T>,
		membership_model: MembershipModel,
	) -> Result<u64, DispatchError> {
		let (_, blueprint) = Self::blueprints(blueprint_id)?;

		blueprint.type_check_request(&request_args).map_err(Error::<T>::TypeCheck)?;
		// ensure we at least have one asset and all assets are unique
		ensure!(!asset_security_requirements.is_empty(), Error::<T>::NoAssetsProvided);
		ensure!(
			asset_security_requirements
				.iter()
				.map(|req| &req.asset)
				.collect::<sp_std::collections::btree_set::BTreeSet<_>>()
				.len() == asset_security_requirements.len(),
			Error::<T>::DuplicateAsset
		);

		let assets = asset_security_requirements
			.clone()
			.into_iter()
			.map(|req| req.asset)
			.collect::<Vec<_>>();
		let bounded_requirements = BoundedVec::try_from(asset_security_requirements)
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
						&Self::account_id(),
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
						&Self::account_id(),
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
						Self::erc20_transfer(token, evm_origin, Self::address(), value)
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
		let asset_security = BoundedVec::<_, MaxAssetsPerServiceOf<T>>::try_from(assets.clone())
			.map_err(|_| Error::<T>::MaxAssetsPerServiceExceeded)?;
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

		let service_request = ServiceRequest {
			blueprint: blueprint_id,
			owner: caller.clone(),
			non_native_asset_security: bounded_requirements,
			ttl,
			args,
			permitted_callers,
			operators_with_approval_state,
			membership_model,
		};

		ensure!(allowed, Error::<T>::InvalidRequestInput);
		ServiceRequests::<T>::insert(request_id, service_request);
		NextServiceRequestId::<T>::set(request_id.saturating_add(1));

		Self::deposit_event(Event::ServiceRequested {
			owner: caller,
			request_id,
			blueprint_id,
			pending_approvals,
			approved: Default::default(),
			asset_security: asset_security.into_iter().map(|asset| (asset, Vec::new())).collect(),
		});

		Ok(request_id)
	}
}
