// This file is part of Tangle.
// Copyright (C) 2022-2024 Tangle Foundation.
//
// Tangle is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// Tangle is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Tangle.  If not, see <http://www.gnu.org/licenses/>.

//! Billing calculation logic and helper methods.

use sp_runtime::traits::{CheckedAdd, CheckedMul, Saturating, Zero};
use sp_std::ops::Div;

use super::{
	super::types::PricingModel,
	types::{BillingCalculation, BillingSkipReason, BillingTrigger, EventLog, ServiceBillingState},
};

/// Payment calculation methods for different pricing models
impl<BlockNumber, Balance> PricingModel<BlockNumber, Balance>
where
	BlockNumber: Copy + PartialOrd + Saturating + Zero + Div<Output = BlockNumber>,
	Balance: Copy + Zero + Saturating + CheckedMul + CheckedAdd + PartialOrd,
{
	/// Calculate billing amount for PayOnce model
	pub fn calculate_pay_once_billing(
		&self,
		last_billed: Option<BlockNumber>,
	) -> Option<BillingCalculation<Balance>> {
		match self {
			PricingModel::PayOnce { amount } =>
				if last_billed.is_none() {
					Some(BillingCalculation {
						amount: *amount,
						trigger: BillingTrigger::Activation,
						should_bill: true,
						skip_reason: None,
					})
				} else {
					Some(BillingCalculation {
						amount: Balance::zero(),
						trigger: BillingTrigger::Activation,
						should_bill: false,
						skip_reason: Some(BillingSkipReason::AlreadyBilled),
					})
				},
			_ => None,
		}
	}

	/// Calculate billing amount for Subscription model
	pub fn calculate_subscription_billing(
		&self,
		current_block: BlockNumber,
		last_billed: Option<BlockNumber>,
	) -> Option<BillingCalculation<Balance>> {
		match self {
			PricingModel::Subscription { rate_per_interval, interval, maybe_end } => {
				// Check if subscription has ended
				if let Some(end_block) = maybe_end {
					if current_block > *end_block {
						return Some(BillingCalculation {
							amount: Balance::zero(),
							trigger: BillingTrigger::BlockInterval,
							should_bill: false,
							skip_reason: Some(BillingSkipReason::SubscriptionEnded),
						});
					}
				}

				let last_billed_block = last_billed.unwrap_or_else(|| BlockNumber::zero());
				let blocks_since_last_billing = current_block.saturating_sub(last_billed_block);

				if blocks_since_last_billing >= *interval {
					// For simplicity, we'll bill for one interval at a time
					Some(BillingCalculation {
						amount: *rate_per_interval,
						trigger: BillingTrigger::BlockInterval,
						should_bill: true,
						skip_reason: None,
					})
				} else {
					Some(BillingCalculation {
						amount: Balance::zero(),
						trigger: BillingTrigger::BlockInterval,
						should_bill: false,
						skip_reason: Some(BillingSkipReason::AlreadyBilled),
					})
				}
			},
			_ => None,
		}
	}

	/// Calculate billing amount for EventDriven model
	pub fn calculate_event_driven_billing(
		&self,
		event_log: &EventLog,
	) -> Option<BillingCalculation<Balance>>
	where
		Balance: From<u64>,
	{
		match self {
			PricingModel::EventDriven { reward_per_event } => {
				let pending_events = event_log.pending_events();

				if pending_events > 0 {
					let events_balance = Balance::from(pending_events);
					if let Some(total_amount) = reward_per_event.checked_mul(&events_balance) {
						Some(BillingCalculation {
							amount: total_amount,
							trigger: BillingTrigger::EventSubmission,
							should_bill: true,
							skip_reason: None,
						})
					} else {
						// Overflow protection
						Some(BillingCalculation {
							amount: Balance::zero(),
							trigger: BillingTrigger::EventSubmission,
							should_bill: false,
							skip_reason: Some(BillingSkipReason::NoEvents),
						})
					}
				} else {
					Some(BillingCalculation {
						amount: Balance::zero(),
						trigger: BillingTrigger::EventSubmission,
						should_bill: false,
						skip_reason: Some(BillingSkipReason::NoEvents),
					})
				}
			},
			_ => None,
		}
	}

	/// Get the billing trigger for this pricing model
	pub fn get_billing_trigger(&self) -> BillingTrigger {
		match self {
			PricingModel::PayOnce { .. } => BillingTrigger::Activation,
			PricingModel::Subscription { .. } => BillingTrigger::BlockInterval,
			PricingModel::EventDriven { .. } => BillingTrigger::EventSubmission,
		}
	}

	/// Check if this pricing model requires periodic billing
	pub fn requires_periodic_billing(&self) -> bool {
		matches!(self, PricingModel::Subscription { .. })
	}

	/// Check if this pricing model is event-based
	pub fn is_event_based(&self) -> bool {
		matches!(self, PricingModel::EventDriven { .. })
	}

	/// Get the next billing block for subscription models
	pub fn next_billing_block(&self, last_billed: Option<BlockNumber>) -> Option<BlockNumber> {
		match self {
			PricingModel::Subscription { interval, maybe_end, .. } => {
				let last_billed_block = last_billed.unwrap_or_else(|| BlockNumber::zero());
				let next_block = last_billed_block.saturating_add(*interval);

				// Check if next billing would be after subscription end
				if let Some(end_block) = maybe_end {
					if next_block > *end_block {
						return None;
					}
				}

				Some(next_block)
			},
			_ => None,
		}
	}

	/// Check if billing is due for this pricing model
	pub fn is_billing_due(
		&self,
		current_block: BlockNumber,
		billing_state: &ServiceBillingState<BlockNumber>,
	) -> bool
	where
		Balance: From<u64>,
	{
		match self {
			PricingModel::PayOnce { .. } => billing_state.last_billed.is_none(),
			PricingModel::Subscription { .. } => {
				if let Some(calculation) =
					self.calculate_subscription_billing(current_block, billing_state.last_billed)
				{
					calculation.should_bill
				} else {
					false
				}
			},
			PricingModel::EventDriven { .. } => billing_state.event_log.pending_events() > 0,
		}
	}

	/// Get a human-readable description of the pricing model
	pub fn description(&self) -> &'static str {
		match self {
			PricingModel::PayOnce { .. } => "Pay-once service with upfront payment",
			PricingModel::Subscription { .. } => "Subscription service with recurring payments",
			PricingModel::EventDriven { .. } => "Event-driven service with per-event billing",
		}
	}
}
