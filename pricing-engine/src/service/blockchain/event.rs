//! Blockchain event definitions for the Tangle Cloud Pricing Engine
//!
//! This module defines the events that the pricing engine processes from the blockchain.
//! For direct access to blockchain events, we should use tangle-subxt's event definitions.

use tangle_subxt as tangle;

/// Provider metadata from blockchain events
#[derive(Debug, Clone)]
pub struct ProviderMetadata {
	/// Provider account ID
	pub account_id: String,
	/// Provider name
	pub name: String,
	/// Provider description
	pub description: Option<String>,
	/// Provider reputation score
	pub reputation: u32,
}

/// Pricing targets from blockchain
#[derive(Debug, Clone)]
pub struct PriceTargets {
	/// Base price
	pub base_price: u128,
	/// Resource multipliers
	pub resource_multipliers: Vec<(String, u128)>,
}

/// Operator preferences
#[derive(Debug, Clone)]
pub struct OperatorPreferences {
	/// Auto approve threshold
	pub auto_approve_threshold: Option<u128>,
	/// Max concurrent services
	pub max_concurrent_services: u32,
}

/// Events from the blockchain that are relevant to the pricing engine
///
/// Note: In production, you would directly use the events from tangle-subxt instead of
/// creating this intermediate representation. This is here to simplify the example.
#[derive(Debug, Clone)]
pub enum BlockchainEvent {
	/// An operator has been registered for a service blueprint
	Registered {
		/// Provider account ID
		provider: String,
		/// Blueprint ID
		blueprint_id: u64,
		/// Operator preferences
		preferences: OperatorPreferences,
		/// Registration arguments
		registration_args: Vec<String>,
	},

	/// An operator has been unregistered
	Unregistered {
		/// Operator account ID
		operator: String,
		/// Blueprint ID
		blueprint_id: u64,
	},

	/// Price targets for an operator have been updated
	PriceTargetsUpdated {
		/// Operator account ID
		operator: String,
		/// Blueprint ID
		blueprint_id: u64,
		/// New price targets
		price_targets: PriceTargets,
	},

	/// A new service has been requested
	ServiceRequested {
		/// Account that requested the service
		owner: String,
		/// Service request ID
		request_id: u64,
		/// Blueprint ID
		blueprint_id: u64,
		/// Operators that need to approve
		pending_approvals: Vec<String>,
		/// Operators that automatically approved
		approved: Vec<String>,
	},

	/// A service request has been approved
	ServiceRequestApproved {
		/// Operator that approved the service
		operator: String,
		/// Service request ID
		request_id: u64,
		/// Blueprint ID
		blueprint_id: u64,
		/// Remaining operators that need to approve
		pending_approvals: Vec<String>,
		/// Operators that have approved
		approved: Vec<String>,
	},

	/// A service request has been rejected
	ServiceRequestRejected {
		/// Operator that rejected the service
		operator: String,
		/// Service request ID
		request_id: u64,
		/// Blueprint ID
		blueprint_id: u64,
	},

	/// A service has been initiated
	ServiceInitiated {
		/// Owner of the service
		owner: String,
		/// Request ID that was approved
		request_id: u64,
		/// Service ID
		service_id: u64,
		/// Blueprint ID
		blueprint_id: u64,
	},

	/// A service has been terminated
	ServiceTerminated {
		/// Owner of the service
		owner: String,
		/// Service ID
		service_id: u64,
		/// Blueprint ID
		blueprint_id: u64,
	},

	/// The chain has been reorganized, potentially invalidating previous events
	ChainReorg {
		/// New best block number
		new_best_block: u32,
		/// Number of blocks that were reorganized
		depth: u32,
	},
}

// Note: In a real implementation, you should directly use the events from tangle-subxt
// instead of manually mapping them to internal types. This would look something like:
//
// ```
// use tangle_subxt::api::services::events::*;
//
// async fn handle_service_registered(event: EventRegistered) {
//     // Process the actual event from tangle-subxt
//     let provider = event.provider.to_string();
//     let blueprint_id = event.blueprint_id;
//     // ... handle the event
// }
// ```
