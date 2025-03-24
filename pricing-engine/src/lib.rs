//! Tangle Cloud Pricing Engine
//!
//! A flexible pricing system for the Tangle Cloud service platform.
//! The pricing engine calculates costs for service deployments based on
//! resource requirements and provider pricing models, supporting
//! competitive bidding in a decentralized marketplace.

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

pub mod calculation;
pub mod error;
pub mod models;
pub mod types;

#[cfg(feature = "std")]
pub mod service;

// Re-exports
pub use calculation::{calculate_service_price, PricingContext};
pub use error::{Error, PricingError, Result};
pub use models::PricingModel;
#[cfg(feature = "std")]
pub use service::{Service, ServiceConfig, ServiceState};
pub use types::{Price, ResourceRequirement, ServiceCategory, TimePeriod};

// Make tangle-subxt available in our crate for blockchain integration
#[cfg(feature = "std")]
pub use tangle_subxt;
