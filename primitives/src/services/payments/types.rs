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

//! Core payment types and enums.

use frame_support::pallet_prelude::*;

/// Represents different billing triggers for services
#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub enum BillingTrigger {
	/// Billing triggered on service activation
	Activation,
	/// Billing triggered on service completion
	Completion,
	/// Billing triggered on block interval
	BlockInterval,
	/// Billing triggered by event submission
	EventSubmission,
}

/// Represents the billing status of a service
#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub enum BillingStatus {
	/// Service has not been billed yet
	NotBilled,
	/// Service has been partially billed
	PartiallyBilled,
	/// Service has been fully billed
	FullyBilled,
	/// Service billing has expired or ended
	Expired,
}

/// Reasons why billing might be skipped
#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub enum BillingSkipReason {
	/// Already billed for this period
	AlreadyBilled,
	/// Subscription has ended
	SubscriptionEnded,
	/// No events to bill
	NoEvents,
	/// Service not yet activated
	NotActivated,
}

/// Comprehensive billing calculation result
#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub struct BillingCalculation<Balance> {
	/// The calculated amount to be billed
	pub amount: Balance,
	/// The billing trigger that caused this calculation
	pub trigger: BillingTrigger,
	/// Whether billing should proceed
	pub should_bill: bool,
	/// Optional reason why billing should not proceed
	pub skip_reason: Option<BillingSkipReason>,
}

/// Event log for tracking events in event-driven services
#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen, Default)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub struct EventLog {
	/// Number of events handled since last billing
	pub events_handled: u64,
	/// Total events handled over service lifetime
	pub total_events: u64,
}

impl EventLog {
	/// Add events to the log
	pub fn add_events(&mut self, count: u64) {
		self.events_handled = self.events_handled.saturating_add(count);
		self.total_events = self.total_events.saturating_add(count);
	}

	/// Clear the pending events (after billing)
	pub fn clear_pending(&mut self) {
		self.events_handled = 0;
	}

	/// Get the number of events pending billing
	pub fn pending_events(&self) -> u64 {
		self.events_handled
	}
}

/// Service billing state that tracks all billing-related information
#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub struct ServiceBillingState<BlockNumber> {
	/// When the service was last billed
	pub last_billed: Option<BlockNumber>,
	/// Current billing status
	pub status: BillingStatus,
	/// Event log for event-driven billing
	pub event_log: EventLog,
}

impl<BlockNumber> Default for ServiceBillingState<BlockNumber> {
	fn default() -> Self {
		Self { last_billed: None, status: BillingStatus::NotBilled, event_log: EventLog::default() }
	}
}

impl<BlockNumber: Copy> ServiceBillingState<BlockNumber> {
	/// Create a new billing state
	pub fn new() -> Self {
		Self::default()
	}

	/// Mark the service as billed at the given block
	pub fn mark_billed(&mut self, block: BlockNumber) {
		self.last_billed = Some(block);
		self.status = BillingStatus::FullyBilled;
	}

	/// Clear event log after billing
	pub fn clear_events(&mut self) {
		self.event_log.clear_pending();
	}

	/// Add events for billing calculation
	pub fn add_events(&mut self, count: u64) {
		self.event_log.add_events(count);
	}
}
