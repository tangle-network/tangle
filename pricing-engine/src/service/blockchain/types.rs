//! Type definitions for blockchain integration
//!
//! This module provides type definitions for working with the Tangle Network
//! blockchain using the tangle-subxt crate.

use std::sync::Arc;

// Import tangle-subxt directly
use tangle_subxt as tangle;

/// Re-export subxt types for convenience
pub use tangle::subxt::{
	backend::rpc::{RpcClient, RpcClientT},
	config::DefaultExtrinsicParamsBuilder,
	tx::Signer,
	Error as SubxtError, OnlineClient, PolkadotConfig,
};

/// Tangle Client type for interacting with the blockchain
pub type TangleClient = Arc<OnlineClient<PolkadotConfig>>;

// Note: In a real implementation, you would directly use the appropriate events and calls
// from the tangle-subxt crate rather than creating wrapper types for everything.
// For example, to access services pallet functionality:
//
// ```
// use tangle_subxt::api::services::calls::*;
// use tangle_subxt::api::services::events::*;
// ```
