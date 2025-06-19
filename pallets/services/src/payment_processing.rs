use crate::{
	BalanceOf, BlockNumberFor, Config, Error, JobPayments, JobSubscriptionBillings, Pallet,
	UserSubscriptionCount,
};
use frame_support::{
	dispatch::DispatchResult,
	ensure,
	pallet_prelude::*,
	traits::{Currency, ReservableCurrency, fungibles::Mutate},
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
		caller: &T::AccountId,
		payer: &T::AccountId,
		amount: BalanceOf<T>,
	) -> DispatchResult {
		// Charge the payment from the payer with authorization check
		Self::charge_payment(caller, payer, amount)?;

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
					caller, // caller is both authorizer and payer for job calls
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
					caller, // caller is both authorizer and payer for subscriptions
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
					caller, // caller is both authorizer and payer for events
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
		caller: &T::AccountId,
		payer: &T::AccountId,
		amount: BalanceOf<T>,
	) -> DispatchResult {
		// Check if payment has already been processed for this call
		if JobPayments::<T>::contains_key(service_id, call_id) {
			return Err(Error::<T>::PaymentAlreadyProcessed.into());
		}

		// Charge the payment from the payer with authorization check
		Self::charge_payment(caller, payer, amount)?;

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
	#[allow(clippy::too_many_arguments)]
	pub fn process_job_subscription_payment(
		service_id: u64,
		job_index: u8,
		_call_id: u64,
		caller: &T::AccountId,
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

		// Check subscription limits for new subscriptions
		let billing_key = (service_id, job_index, payer.clone());
		let is_new_subscription = !JobSubscriptionBillings::<T>::contains_key(&billing_key);

		if is_new_subscription {
			let current_count = UserSubscriptionCount::<T>::get(payer);
			ensure!(current_count < 100, Error::<T>::TooManySubscriptions); // Max 100 subscriptions per user
			UserSubscriptionCount::<T>::insert(payer, current_count + 1);
		}

		// Get or create billing information for this subscription
		let mut billing = JobSubscriptionBillings::<T>::get(&billing_key).unwrap_or_else(|| {
			// Set last_billed to a past block so the first payment is due immediately
			let initial_last_billed = if current_block >= interval {
				current_block.saturating_sub(interval)
			} else {
				// If current_block < interval, start from block 0 to ensure immediate payment
				BlockNumberFor::<T>::zero()
			};

			log::debug!(
				"Creating new subscription billing for service {} job {} subscriber {:?}: last_billed set to {:?} (current: {:?}, interval: {:?})",
				service_id,
				job_index,
				payer,
				initial_last_billed,
				current_block,
				interval
			);

			JobSubscriptionBilling {
				service_id,
				job_index,
				subscriber: payer.clone(),
				last_billed: initial_last_billed, // Fixed: Now ensures first payment is due
				end_block: maybe_end,
			}
		});

		// Determine if payment is due
		let blocks_since_last = current_block.saturating_sub(billing.last_billed);
		let payment_due = blocks_since_last >= interval;

		log::debug!(
			"Subscription billing check for service {} job {}: blocks_since_last={:?}, interval={:?}, payment_due={}",
			service_id,
			job_index,
			blocks_since_last,
			interval,
			payment_due
		);

		if payment_due {
			// Process the subscription payment with authorization check
			Self::charge_payment(caller, payer, rate_per_interval)?;

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
		} else {
			log::debug!(
				"Subscription payment not due for service {} job {}: {:?} blocks since last < {:?} interval",
				service_id,
				job_index,
				blocks_since_last,
				interval
			);
		}

		Ok(())
	}

	/// Process event-driven payment when an event is reported
	pub fn process_job_event_driven_payment(
		service_id: u64,
		job_index: u8,
		_call_id: u64,
		caller: &T::AccountId,
		payer: &T::AccountId,
		reward_per_event: BalanceOf<T>,
		event_count: u32,
	) -> DispatchResult {
		let service = Self::services(service_id)?;
		let (_, blueprint) = Self::blueprints(service.blueprint)?;
		let _job_def = blueprint.jobs.get(job_index as usize).ok_or(Error::<T>::InvalidJobId)?;

		let total_reward = reward_per_event
			.checked_mul(&event_count.into())
			.ok_or(Error::<T>::PaymentCalculationOverflow)?;

		// Charge the payment with authorization check
		Self::charge_payment(caller, payer, total_reward)?;

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

	/// Charge payment from a user account with proper authorization checks
	fn charge_payment_with_asset(
		caller: &T::AccountId,
		payer: &T::AccountId,
		amount: BalanceOf<T>,
		asset: &Asset<T::AssetId>,
	) -> DispatchResult {
		// SECURITY CHECK: Ensure the caller has authorization to charge the payer
		ensure!(caller == payer, Error::<T>::InvalidRequestInput);

		match asset {
			Asset::Custom(asset_id) => {
				if *asset_id == T::AssetId::default() {
					// Native currency
					let free_balance = T::Currency::free_balance(payer);
					ensure!(free_balance >= amount, Error::<T>::InvalidRequestInput);
					T::Currency::reserve(payer, amount)?;
				} else {
					// Custom asset
					T::Fungibles::transfer(
						asset_id.clone(),
						payer,
						&Self::pallet_account(),
						amount,
						frame_support::traits::tokens::Preservation::Expendable,
					)
					.map_err(|_| Error::<T>::CustomAssetTransferFailed)?;
				}
			},
			Asset::Erc20(_) => {
				// ERC20 handled separately
				let free_balance = T::Currency::free_balance(payer);
				ensure!(free_balance >= amount, Error::<T>::InvalidRequestInput);
				T::Currency::reserve(payer, amount)?;
			},
		}

		Ok(())
	}

	/// Charge payment from a user account with proper authorization checks (native currency)
	fn charge_payment(
		caller: &T::AccountId,
		payer: &T::AccountId,
		amount: BalanceOf<T>,
	) -> DispatchResult {
		Self::charge_payment_with_asset(
			caller,
			payer,
			amount,
			&Asset::Custom(T::AssetId::default()),
		)
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
					// Custom asset - properly handle via fungibles trait
					let account_id = staging_payment
						.refund_to
						.clone()
						.try_into_account_id()
						.map_err(|_| Error::<T>::ExpectedAccountId)?;
					T::Fungibles::transfer(
						asset_id.clone(),
						&account_id,
						&Self::pallet_account(),
						staging_payment.amount,
						frame_support::traits::tokens::Preservation::Expendable,
					)
					.map_err(|_| Error::<T>::CustomAssetTransferFailed)?;
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
	///
	/// # Security Note
	/// This function processes automatic subscription payments. Since these are
	/// pre-authorized through the service registration process, we use the
	/// subscriber as both caller and payer for automated billing.
	pub fn process_subscription_payments_on_block(current_block: BlockNumberFor<T>) -> Weight {
		let mut total_weight = Weight::zero();
		let mut processed_count = 0u32;
		const MAX_SUBSCRIPTIONS_PER_BLOCK: u32 = 50; // Limit processing to prevent spam attacks

		// Iterate through subscription billings directly to avoid nested loops
		for ((service_id, job_index, subscriber), billing) in JobSubscriptionBillings::<T>::iter() {
			if processed_count >= MAX_SUBSCRIPTIONS_PER_BLOCK {
				break; // Early termination to prevent excessive processing
			}

			// Get service and blueprint info
			if let Ok(service_instance) = Self::services(service_id) {
				if let Ok((_, blueprint)) = Self::blueprints(service_instance.blueprint) {
					if let Some(job_def) = blueprint.jobs.get(job_index as usize) {
						if let PricingModel::Subscription {
							rate_per_interval,
							interval,
							maybe_end,
						} = &job_def.pricing_model
						{
							// Convert types for runtime compatibility
							let rate_converted: BalanceOf<T> =
								(*rate_per_interval).saturated_into();
							let interval_converted: BlockNumberFor<T> =
								(*interval).saturated_into();
							let maybe_end_converted: Option<BlockNumberFor<T>> =
								maybe_end.map(|end| end.saturated_into());

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

								// Process payment with early termination on failure
								if Self::process_job_subscription_payment(
									service_id,
									job_index,
									0,           /* call_id not relevant for subscription
									              * processing */
									&subscriber, /* subscriber authorizes their own automated
									              * payment */
									&subscriber,
									rate_converted,
									interval_converted,
									maybe_end_converted,
									current_block,
								)
								.is_err()
								{
									// Early termination on payment failure to prevent further
									// processing issues
									break;
								}

								processed_count += 1;
							}
						}
					}
				}
			}

			// Add weight for processing
			total_weight = total_weight.saturating_add(T::DbWeight::get().reads_writes(3, 1));
		}

		total_weight
	}

	/// Validate payment amount against pricing model
	pub fn validate_payment_amount(
		blueprint: &ServiceBlueprint<T::Constraints>,
		provided_amount: BalanceOf<T>,
	) -> DispatchResult {
		// Allow zero payments (no upfront payment, payments will be handled at job level)
		if provided_amount.is_zero() {
			return Ok(());
		}

		// If payment is provided, validate it makes sense for the blueprint's jobs
		let mut has_pay_once_jobs = false;
		let mut has_subscription_jobs = false;
		let mut min_pay_once_amount: Option<BalanceOf<T>> = None;
		let mut min_subscription_rate: Option<BalanceOf<T>> = None;

		for job_def in &blueprint.jobs {
			match &job_def.pricing_model {
				PricingModel::PayOnce { amount } => {
					has_pay_once_jobs = true;
					let amount_converted: BalanceOf<T> = (*amount).saturated_into();
					match min_pay_once_amount {
						Some(current_min) =>
							if amount_converted < current_min {
								min_pay_once_amount = Some(amount_converted);
							},
						None => {
							min_pay_once_amount = Some(amount_converted);
						},
					}
				},
				PricingModel::Subscription { rate_per_interval, .. } => {
					has_subscription_jobs = true;
					let rate_converted: BalanceOf<T> = (*rate_per_interval).saturated_into();
					match min_subscription_rate {
						Some(current_min) =>
							if rate_converted < current_min {
								min_subscription_rate = Some(rate_converted);
							},
						None => {
							min_subscription_rate = Some(rate_converted);
						},
					}
				},
				PricingModel::EventDriven { .. } => {
					// Event-driven jobs don't require upfront payment validation
				},
			}
		}

		// Validate based on the job types present
		if has_pay_once_jobs {
			// For pay-once jobs, the upfront payment should be at least the minimum required
			if let Some(min_amount) = min_pay_once_amount {
				ensure!(provided_amount >= min_amount, Error::<T>::InvalidRequestInput);
			}
		} else if has_subscription_jobs {
			// For subscription-only services, payment should cover at least one interval
			if let Some(min_rate) = min_subscription_rate {
				ensure!(provided_amount >= min_rate, Error::<T>::InvalidRequestInput);
			}
		}
		// If only event-driven jobs exist, any amount is acceptable

		Ok(())
	}
}
