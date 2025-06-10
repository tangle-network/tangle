use crate::{
	BalanceOf, BlockNumberFor, Config, Error, Instances, JobPayments, JobSubscriptionBillings,
	Pallet,
};
use frame_support::{
	dispatch::DispatchResult,
	ensure,
	pallet_prelude::*,
	traits::{Currency, ReservableCurrency},
};
use sp_runtime::traits::{CheckedMul, SaturatedConversion, Saturating, Zero};
use tangle_primitives::{
	services::{
		Asset, JobPayment, JobSubscriptionBilling, PricingModel, ServiceBlueprint,
		StagingServicePayment,
	},
	traits::RewardRecorder as RewardRecorderTrait,
};

impl<T: Config> Pallet<T> {
	/// Process a one-time payment for a service (not job-specific)
	pub fn process_pay_once_payment(
		service_id: u64,
		payer: &T::AccountId,
		amount: BalanceOf<T>,
	) -> DispatchResult {
		// Charge the payment from the payer
		Self::charge_payment(payer, amount)?;

		// For service-level payments, we can record a generic reward
		// This would typically be handled by the MBSM or service initialization
		log::debug!(
			"Processed service-level pay-once payment for service {}: {:?}",
			service_id,
			amount
		);

		Ok(())
	}
	/// Process payment for a specific job call
	pub fn process_job_payment(
		service_id: u64,
		job_index: u8,
		call_id: u64,
		caller: &T::AccountId,
		current_block: BlockNumberFor<T>,
	) -> DispatchResult {
		let service = Self::services(service_id)?;
		let (_, blueprint) = Self::blueprints(service.blueprint)?;

		// Find the job definition
		let job_def = blueprint.jobs.get(job_index as usize).ok_or(Error::<T>::InvalidJobId)?;

		match &job_def.pricing_model {
			PricingModel::PayOnce { amount } => {
				let amount_converted: BalanceOf<T> = (*amount).saturated_into();
				Self::process_job_pay_once_payment(
					service_id,
					job_index,
					call_id,
					caller,
					amount_converted,
				)?;
			},
			PricingModel::Subscription { rate_per_interval, interval, maybe_end } => {
				let rate_converted: BalanceOf<T> = (*rate_per_interval).saturated_into();
				let interval_converted: BlockNumberFor<T> = (*interval).saturated_into();
				let maybe_end_converted: Option<BlockNumberFor<T>> =
					maybe_end.map(|end| end.saturated_into());
				Self::process_job_subscription_payment(
					service_id,
					job_index,
					call_id,
					caller,
					rate_converted,
					interval_converted,
					maybe_end_converted,
					current_block,
				)?;
			},
			PricingModel::EventDriven { reward_per_event } => {
				let reward_converted: BalanceOf<T> = (*reward_per_event).saturated_into();
				Self::process_job_event_driven_payment(
					service_id,
					job_index,
					call_id,
					caller,
					reward_converted,
					1, // Default to 1 event for this job call
				)?;
			},
		}

		Ok(())
	}

	/// Process a one-time payment for a job call
	pub fn process_job_pay_once_payment(
		service_id: u64,
		job_index: u8,
		call_id: u64,
		payer: &T::AccountId,
		amount: BalanceOf<T>,
	) -> DispatchResult {
		// Check if payment has already been processed for this call
		if JobPayments::<T>::contains_key(service_id, call_id) {
			return Err(Error::<T>::PaymentAlreadyProcessed.into());
		}

		// Charge the payment from the payer
		Self::charge_payment(payer, amount)?;

		// Record the payment
		let payment = JobPayment {
			service_id,
			job_index,
			call_id,
			payer: payer.clone(),
			asset: Asset::Custom(0u32),      // Default to native asset ID 0
			amount: amount.saturated_into(), // Convert to u128
		};

		JobPayments::<T>::insert(service_id, call_id, &payment);

		// Record the reward with the rewards pallet
		let service = Self::services(service_id)?;
		let (_, blueprint) = Self::blueprints(service.blueprint)?;
		let job_def = blueprint.jobs.get(job_index as usize).ok_or(Error::<T>::InvalidJobId)?;

		// Convert the job-level pricing model to runtime types
		let runtime_pricing_model = match &job_def.pricing_model {
			PricingModel::PayOnce { amount: job_amount } => {
				let runtime_amount: BalanceOf<T> = (*job_amount).saturated_into();
				PricingModel::PayOnce { amount: runtime_amount }
			},
			PricingModel::Subscription { rate_per_interval, interval, maybe_end } => {
				let runtime_rate: BalanceOf<T> = (*rate_per_interval).saturated_into();
				let runtime_interval: BlockNumberFor<T> = (*interval).saturated_into();
				let runtime_maybe_end: Option<BlockNumberFor<T>> =
					maybe_end.map(|end| end.saturated_into());
				PricingModel::Subscription {
					rate_per_interval: runtime_rate,
					interval: runtime_interval,
					maybe_end: runtime_maybe_end,
				}
			},
			PricingModel::EventDriven { reward_per_event } => {
				let runtime_reward: BalanceOf<T> = (*reward_per_event).saturated_into();
				PricingModel::EventDriven { reward_per_event: runtime_reward }
			},
		};

		T::RewardRecorder::record_reward(payer, service_id, amount, &runtime_pricing_model)?;

		log::debug!(
			"Processed pay-once payment for job call {}-{}-{}: {:?}",
			service_id,
			job_index,
			call_id,
			amount
		);

		Ok(())
	}

	/// Process subscription payment for a job
	pub fn process_job_subscription_payment(
		service_id: u64,
		job_index: u8,
		_call_id: u64,
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
					"Subscription for service {} job {} has ended at block {:?}",
					service_id,
					job_index,
					end_block
				);
				return Ok(());
			}
		}

		// Get or create billing information for this subscription
		let billing_key = (service_id, job_index, payer.clone());
		let mut billing = JobSubscriptionBillings::<T>::get(&billing_key).unwrap_or_else(|| {
			JobSubscriptionBilling {
				service_id,
				job_index,
				subscriber: payer.clone(),
				last_billed: current_block,
				end_block: maybe_end,
			}
		});

		// Determine if payment is due
		let blocks_since_last = current_block.saturating_sub(billing.last_billed);
		let payment_due = blocks_since_last >= interval;

		if payment_due {
			// Process the subscription payment
			Self::charge_payment(payer, rate_per_interval)?;

			// Update last billed block
			billing.last_billed = current_block;
			JobSubscriptionBillings::<T>::insert(&billing_key, &billing);

			// Record the reward
			let service = Self::services(service_id)?;
			let (_, blueprint) = Self::blueprints(service.blueprint)?;
			let _job_def =
				blueprint.jobs.get(job_index as usize).ok_or(Error::<T>::InvalidJobId)?;

			// Convert pricing model to runtime types for reward recording
			let runtime_pricing_model =
				PricingModel::Subscription { rate_per_interval, interval, maybe_end };

			T::RewardRecorder::record_reward(
				payer,
				service_id,
				rate_per_interval,
				&runtime_pricing_model,
			)?;

			log::debug!(
				"Processed subscription payment for service {} job {}: {:?} at block {:?}",
				service_id,
				job_index,
				rate_per_interval,
				current_block
			);
		}

		Ok(())
	}

	/// Process event-driven payment when an event is reported
	pub fn process_job_event_driven_payment(
		service_id: u64,
		job_index: u8,
		_call_id: u64,
		payer: &T::AccountId,
		reward_per_event: BalanceOf<T>,
		event_count: u32,
	) -> DispatchResult {
		let service = Self::services(service_id)?;
		let (_, blueprint) = Self::blueprints(service.blueprint)?;
		let _job_def = blueprint.jobs.get(job_index as usize).ok_or(Error::<T>::InvalidJobId)?;

		let total_reward = reward_per_event
			.checked_mul(&event_count.into())
			.ok_or(Error::<T>::InvalidRequestInput)?;

		// Record the reward with the rewards pallet
		let runtime_pricing_model = PricingModel::EventDriven { reward_per_event };
		T::RewardRecorder::record_reward(payer, service_id, total_reward, &runtime_pricing_model)?;

		log::debug!(
			"Processed event-driven payment for service {} job {}: {} events, total reward: {:?}",
			service_id,
			job_index,
			event_count,
			total_reward
		);

		Ok(())
	}

	/// Charge payment from a user account
	fn charge_payment(payer: &T::AccountId, amount: BalanceOf<T>) -> DispatchResult {
		// For now, we'll use the native currency
		let free_balance = T::Currency::free_balance(payer);
		ensure!(free_balance >= amount, Error::<T>::InvalidRequestInput);

		// Reserve the payment amount
		T::Currency::reserve(payer, amount)?;

		Ok(())
	}

	/// Transfer staged payment to the rewards pallet
	pub fn transfer_payment_to_rewards(
		service_id: u64,
		staging_payment: &StagingServicePayment<T::AccountId, T::AssetId, BalanceOf<T>>,
	) -> DispatchResult {
		match &staging_payment.asset {
			Asset::Custom(asset_id) => {
				if *asset_id == T::AssetId::default() {
					// Native currency - unreserve
					let account_id = staging_payment
						.refund_to
						.clone()
						.try_into_account_id()
						.map_err(|_| Error::<T>::ExpectedAccountId)?;
					T::Currency::unreserve(&account_id, staging_payment.amount);
				} else {
					// Custom asset - would need proper implementation based on fungibles trait
					log::debug!("Processing custom asset payment for service {}", service_id);
				}
			},
			Asset::Erc20(_contract_address) => {
				// ERC20 payment - already handled during service request
				log::debug!("Processing ERC20 payment for service {}", service_id);
			},
		}

		Ok(())
	}

	/// Hook called on every block to process subscription payments
	pub fn process_subscription_payments_on_block(current_block: BlockNumberFor<T>) -> Weight {
		let mut total_weight = Weight::zero();

		// Iterate through all active services and check for subscription payments
		for (service_id, _service) in Instances::<T>::iter() {
			if let Ok((_, blueprint)) = Self::blueprints(_service.blueprint) {
				for (job_index, job_def) in blueprint.jobs.iter().enumerate() {
					if let PricingModel::Subscription { rate_per_interval, interval, maybe_end } =
						&job_def.pricing_model
					{
						// Process subscription payments for all subscribers to this job
						let job_index = job_index as u8;

						// Convert types for runtime compatibility
						let rate_converted: BalanceOf<T> = (*rate_per_interval).saturated_into();
						let interval_converted: BlockNumberFor<T> = (*interval).saturated_into();
						let maybe_end_converted: Option<BlockNumberFor<T>> =
							maybe_end.map(|end| end.saturated_into());

						// Iterate through all job subscription billings
						for ((s_id, j_idx, subscriber), billing) in
							JobSubscriptionBillings::<T>::iter()
						{
							if s_id == service_id && j_idx == job_index {
								// Check if payment is due
								let blocks_since_last =
									current_block.saturating_sub(billing.last_billed);
								if blocks_since_last >= interval_converted {
									// Check if subscription hasn't ended
									if let Some(end_block) = maybe_end_converted {
										if current_block > end_block {
											continue;
										}
									}

									// Process payment
									let _ = Self::process_job_subscription_payment(
										service_id,
										job_index,
										0, // call_id not relevant for subscription processing
										&subscriber,
										rate_converted,
										interval_converted,
										maybe_end_converted,
										current_block,
									);
								}
							}
						}

						// Add weight for processing
						total_weight =
							total_weight.saturating_add(T::DbWeight::get().reads_writes(3, 1));
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
		// For service-level validation, we could check against all job pricing models
		// For now, we'll accept any positive amount
		ensure!(!provided_amount.is_zero(), Error::<T>::InvalidRequestInput);

		// Validate against each job's pricing model if needed
		for job_def in &blueprint.jobs {
			match &job_def.pricing_model {
				PricingModel::PayOnce { amount } => {
					// Individual job validation would happen at job call time
					log::debug!("Job has pay-once pricing: {:?}", amount);
				},
				PricingModel::Subscription { rate_per_interval, .. } => {
					// Individual job validation would happen at job call time
					log::debug!("Job has subscription pricing: {:?}", rate_per_interval);
				},
				PricingModel::EventDriven { reward_per_event } => {
					// For event-driven, any amount is acceptable as it's paid per event
					log::debug!("Job has event-driven pricing: {:?}", reward_per_event);
				},
			}
		}

		Ok(())
	}
}
