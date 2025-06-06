use crate::{BalanceOf, BlockNumberFor, Config, Error, JobPayments, JobSubscriptionBillings, Pallet};
use frame_support::{
	dispatch::DispatchResult,
	ensure,
	pallet_prelude::*,
	traits::{Currency, ReservableCurrency},
};
use sp_runtime::traits::{CheckedMul, Saturating};
use tangle_primitives::{
	services::{Asset, JobPayment, JobSubscriptionBilling, PricingModel, ServiceBlueprint},
	traits::RewardRecorder as RewardRecorderTrait,
};

impl<T: Config> Pallet<T> {
	/// Process payment for a specific job call
	pub fn process_job_payment(
		service_id: u64,
		job_id: u8,
		call_id: u64,
		caller: &T::AccountId,
		current_block: BlockNumberFor<T>,
	) -> DispatchResult {
		let service = Self::services(service_id)?;
		let (_, blueprint) = Self::blueprints(service.blueprint)?;
		
		// Find the job definition
		let job_def = blueprint.jobs.get(job_id as usize)
			.ok_or(Error::<T>::InvalidJobId)?;

		match &job_def.pricing_model {
			PricingModel::PayOnce { amount } => {
				Self::process_job_pay_once_payment(service_id, job_id, call_id, caller, *amount)?;
			},
			PricingModel::Subscription { rate_per_interval, interval, maybe_end } => {
				Self::process_job_subscription_payment(
					service_id,
					job_id,
					call_id,
					caller,
					*rate_per_interval,
					*interval,
					*maybe_end,
					current_block,
				)?;
			},
			PricingModel::EventDriven { reward_per_event } => {
				Self::process_job_event_driven_payment(
					service_id,
					job_id,
					call_id,
					caller,
					*reward_per_event,
					1, // Default to 1 event for this job call
				)?;
			},
		}

		Ok(())
	}

	/// Process a one-time payment for a job call
	pub fn process_job_pay_once_payment(
		service_id: u64,
		job_id: u8,
		call_id: u64,
		payer: &T::AccountId,
		amount: BalanceOf<T>,
	) -> DispatchResult {
		// Check if payment has already been processed for this call
		if JobPayments::<T>::contains_key(call_id) {
			return Err(Error::<T>::PaymentAlreadyProcessed.into());
		}

		// Charge the payment
		Self::charge_job_payment(payer, &amount)?;

		// Record the payment
		let payment = JobPayment {
			service_id,
			job_id,
			call_id,
			payer: payer.clone(),
			amount,
			processed_at: frame_system::Pallet::<T>::block_number(),
		};

		JobPayments::<T>::insert(call_id, &payment);

		// Record the reward with the rewards pallet
		T::RewardRecorder::record_job_reward(payer, service_id, job_id, call_id, amount)?;

		log::debug!(
			"Processed pay-once payment for job call {}-{}-{}: {:?}",
			service_id,
			job_id,
			call_id,
			amount
		);

		Ok(())
	}

	/// Process subscription payment for a service
	fn process_subscription_payment(
		service_id: u64,
		payer: &T::AccountId,
		rate_per_interval: BalanceOf<T>,
		interval: BlockNumberFor<T>,
		maybe_end: Option<BlockNumberFor<T>>,
		current_block: BlockNumberFor<T>,
	) -> DispatchResult {
		// Check if subscription has ended
		if let Some(end_block) = maybe_end {
			if current_block > end_block {
				log::debug!(
					"Subscription for service {} has ended at block {:?}",
					service_id,
					end_block
				);
				return Ok(());
			}
		}

		// Get the service to check last billing block
		let mut service = Self::services(service_id)?;

		// Determine if payment is due
		let payment_due = match service.last_billed {
			Some(last_billed) => {
				let blocks_since_last = current_block.saturating_sub(last_billed);
				blocks_since_last >= interval
			},
			None => true, // First payment
		};

		if payment_due {
			// Process the subscription payment
			Self::charge_subscription_payment(service_id, payer, &rate_per_interval)?;

			// Update last billed block
			service.last_billed = Some(current_block);
			Instances::<T>::insert(service_id, &service);

			log::debug!(
				"Processed subscription payment for service {}: {:?} at block {:?}",
				service_id,
				rate_per_interval,
				current_block
			);
		}

		Ok(())
	}

	/// Process event-driven payment when an event is reported
	pub fn process_event_driven_payment(service_id: u64, event_count: u32) -> DispatchResult {
		let service = Self::services(service_id)?;
		let (_, blueprint) = Self::blueprints(service.blueprint)?;

		if let PricingModel::EventDriven { reward_per_event } = &blueprint.pricing_model {
			let total_reward = reward_per_event
				.checked_mul(&event_count.into())
				.ok_or(Error::<T>::InvalidRequestInput)?;

			// Record the reward with the rewards pallet
			T::RewardRecorder::record_reward(
				&service.owner,
				service_id,
				total_reward,
				&blueprint.pricing_model,
			)?;

			log::debug!(
				"Processed event-driven payment for service {}: {} events, total reward: {:?}",
				service_id,
				event_count,
				total_reward
			);
		}

		Ok(())
	}

	/// Charge subscription payment from the service owner
	fn charge_subscription_payment(
		service_id: u64,
		payer: &T::AccountId,
		amount: &BalanceOf<T>,
	) -> DispatchResult {
		// For now, we'll use the native currency
		// In a full implementation, this would support multiple assets
		let free_balance = T::Currency::free_balance(payer);
		ensure!(free_balance >= *amount, Error::<T>::InvalidRequestInput);

		// Reserve the payment amount
		T::Currency::reserve(payer, *amount)?;

		// Record the reward with the rewards pallet
		let service = Self::services(service_id)?;
		let (_, blueprint) = Self::blueprints(service.blueprint)?;

		T::RewardRecorder::record_reward(payer, service_id, *amount, &blueprint.pricing_model)?;

		Ok(())
	}

	/// Transfer staged payment to the rewards pallet
	fn transfer_payment_to_rewards(
		service_id: u64,
		staging_payment: &StagingServicePayment<T::AccountId, T::AssetId, BalanceOf<T>>,
	) -> DispatchResult {
		match &staging_payment.asset {
			Asset::Custom(asset_id) => {
				if *asset_id == T::AssetId::default() {
					// Native currency - unreserve and record reward
					let account_id = staging_payment
						.refund_to
						.clone()
						.try_into_account_id()
						.map_err(|_| Error::<T>::ExpectedAccountId)?;
					T::Currency::unreserve(&account_id, staging_payment.amount);
				} else {
					// Custom asset - transfer to rewards pallet
					// This would need proper implementation based on your fungibles trait
					log::debug!("Processing custom asset payment for service {}", service_id);
				}
			},
			Asset::Erc20(_contract_address) => {
				// ERC20 payment - already handled during service request
				log::debug!("Processing ERC20 payment for service {}", service_id);
			},
		}

		// Record the reward
		let service = Self::services(service_id)?;
		let (_, blueprint) = Self::blueprints(service.blueprint)?;

		let account_id = staging_payment
			.refund_to
			.clone()
			.try_into_account_id()
			.map_err(|_| Error::<T>::ExpectedAccountId)?;

		T::RewardRecorder::record_reward(
			&account_id,
			service_id,
			staging_payment.amount,
			&blueprint.pricing_model,
		)?;

		Ok(())
	}

	/// Hook called on every block to process subscription payments
	pub fn process_subscription_payments_on_block(current_block: BlockNumberFor<T>) -> Weight {
		let mut total_weight = Weight::zero();

		// Iterate through all active services and check for subscription payments
		for (service_id, _) in Instances::<T>::iter() {
			if let Ok(service) = Self::services(service_id) {
				if let Ok((_, blueprint)) = Self::blueprints(service.blueprint) {
					if let PricingModel::Subscription { rate_per_interval, interval, maybe_end } =
						&blueprint.pricing_model
					{
						// Process subscription payment
						let _ = Self::process_subscription_payment(
							service_id,
							&service.owner,
							*rate_per_interval,
							*interval,
							*maybe_end,
							current_block,
						);

						// Add weight for processing
						total_weight =
							total_weight.saturating_add(T::DbWeight::get().reads_writes(2, 1));
					}
				}
			}
		}

		total_weight
	}

	/// Validate payment amount against pricing model
	pub fn validate_payment_amount(
		        blueprint: &ServiceBlueprint<T::Constraints>,
		provided_amount: BalanceOf<T>,
	) -> DispatchResult {
		match &blueprint.pricing_model {
			PricingModel::PayOnce { amount } => {
				ensure!(provided_amount >= *amount, Error::<T>::InvalidRequestInput);
			},
			PricingModel::Subscription { rate_per_interval, .. } => {
				// For subscriptions, the initial payment should cover at least one interval
				ensure!(provided_amount >= *rate_per_interval, Error::<T>::InvalidRequestInput);
			},
			PricingModel::EventDriven { reward_per_event } => {
				// For event-driven, any amount is acceptable as it's paid per event
				// The provided amount could be a deposit or initial payment
				log::debug!(
					"Event-driven service, reward per event: {:?}, provided: {:?}",
					reward_per_event,
					provided_amount
				);
			},
		}

		Ok(())
	}
}
