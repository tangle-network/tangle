//! Core types for the pricing engine
//!
//! This module defines the fundamental data types used throughout the pricing engine.

use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;

use core::{fmt, str::FromStr};
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

/// Resource units for various types of cloud resources
#[derive(Debug, Clone, PartialEq, Encode, Decode, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum ResourceUnit {
	/// CPU cores or vCPUs
	CPU,
	/// Memory in megabytes
	MemoryMB,
	/// Storage in megabytes
	StorageMB,
	/// Network egress in megabytes
	NetworkEgressMB,
	/// Network ingress in megabytes
	NetworkIngressMB,
	/// GPU units
	GPU,
	/// Request count (for FaaS/API services)
	Request,
	/// Invocation count (for FaaS)
	Invocation,
	/// Execution time in milliseconds
	ExecutionTimeMS,
	/// Custom unit with a name
	Custom(String),
}

#[cfg(feature = "std")]
impl fmt::Display for ResourceUnit {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			ResourceUnit::CPU => write!(f, "CPU"),
			ResourceUnit::MemoryMB => write!(f, "MemoryMB"),
			ResourceUnit::StorageMB => write!(f, "StorageMB"),
			ResourceUnit::NetworkEgressMB => write!(f, "NetworkEgressMB"),
			ResourceUnit::NetworkIngressMB => write!(f, "NetworkIngressMB"),
			ResourceUnit::GPU => write!(f, "GPU"),
			ResourceUnit::Request => write!(f, "Request"),
			ResourceUnit::Invocation => write!(f, "Invocation"),
			ResourceUnit::ExecutionTimeMS => write!(f, "ExecutionTimeMS"),
			ResourceUnit::Custom(name) => write!(f, "{}", name),
		}
	}
}

/// Error type for parsing resource units
#[derive(Debug)]
#[cfg_attr(feature = "std", derive(thiserror::Error))]
#[cfg_attr(feature = "std", error("Failed to parse resource unit"))]
pub struct ParseResourceUnitError;

#[cfg(feature = "std")]
impl FromStr for ResourceUnit {
	type Err = ParseResourceUnitError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s.to_uppercase().as_str() {
			"CPU" => Ok(ResourceUnit::CPU),
			"MEMORYMB" => Ok(ResourceUnit::MemoryMB),
			"STORAGEMB" => Ok(ResourceUnit::StorageMB),
			"NETWORKEGRESSMB" => Ok(ResourceUnit::NetworkEgressMB),
			"NETWORKINGRESSMB" => Ok(ResourceUnit::NetworkIngressMB),
			"GPU" => Ok(ResourceUnit::GPU),
			"REQUEST" => Ok(ResourceUnit::Request),
			"INVOCATION" => Ok(ResourceUnit::Invocation),
			"EXECUTIONTIMEMS" => Ok(ResourceUnit::ExecutionTimeMS),
			_ => Ok(ResourceUnit::Custom(s.to_string())),
		}
	}
}

/// Represents a price with a value and currency/token
#[derive(Debug, Clone, PartialEq, Encode, Decode, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct Price {
	/// Numerical value of the price (in the smallest unit of the token, e.g., microtoken)
	pub value: u128,
	/// Token or currency used for pricing (e.g., "TGL")
	pub token: String,
}

impl Price {
	/// Create a new price
	pub fn new(value: u128, token: impl Into<String>) -> Self {
		Self { value, token: token.into() }
	}

	/// Add another price to this one, assuming same token
	pub fn add(&self, other: &Price) -> Result<Price, &'static str> {
		if self.token != other.token {
			return Err("Cannot add prices with different tokens");
		}

		Ok(Price { value: self.value.saturating_add(other.value), token: self.token.clone() })
	}

	/// Scale this price by a factor
	pub fn scale(&self, factor: u128) -> Price {
		Price { value: self.value.saturating_mul(factor), token: self.token.clone() }
	}
}

#[cfg(feature = "std")]
impl fmt::Display for Price {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		// Format to show as a decimal number with 6 decimal places (assuming microtoken)
		let whole = self.value / 1_000_000;
		let fractional = self.value % 1_000_000;
		write!(f, "{}.{:06} {}", whole, fractional, self.token)
	}
}

/// Resource requirement for a service
#[derive(Debug, Clone, Encode, Decode, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct ResourceRequirement {
	/// Type of resource
	pub unit: ResourceUnit,
	/// Quantity of the resource (in the smallest measurable unit)
	pub quantity: u128,
}

impl ResourceRequirement {
	/// Create a new resource requirement
	pub fn new(unit: ResourceUnit, quantity: u128) -> Self {
		Self { unit, quantity }
	}
}

/// Service category to help classify different types of services
#[derive(Debug, Clone, PartialEq, Encode, Decode, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum ServiceCategory {
	/// General compute resources (VMs, containers)
	Compute,
	/// Storage services
	Storage,
	/// Network services
	Network,
	/// Database services
	Database,
	/// Function as a Service
	FaaS,
	/// API-based services
	API,
	/// AI/ML services
	AI,
	/// Custom category with a name
	Custom(String),
}

#[cfg(feature = "std")]
impl fmt::Display for ServiceCategory {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			ServiceCategory::Compute => write!(f, "Compute"),
			ServiceCategory::Storage => write!(f, "Storage"),
			ServiceCategory::Network => write!(f, "Network"),
			ServiceCategory::Database => write!(f, "Database"),
			ServiceCategory::FaaS => write!(f, "FaaS"),
			ServiceCategory::API => write!(f, "API"),
			ServiceCategory::AI => write!(f, "AI"),
			ServiceCategory::Custom(name) => write!(f, "{}", name),
		}
	}
}

/// Time period for recurring pricing
#[derive(Debug, Clone, PartialEq, Encode, Decode, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum TimePeriod {
	/// Second
	Second,
	/// Minute
	Minute,
	/// Hour
	Hour,
	/// Day
	Day,
	/// Week
	Week,
	/// Month (30 days)
	Month,
	/// Year (365 days)
	Year,
}

impl TimePeriod {
	/// Convert to seconds for calculation purposes
	pub fn to_seconds(&self) -> u64 {
		match self {
			TimePeriod::Second => 1,
			TimePeriod::Minute => 60,
			TimePeriod::Hour => 3600,
			TimePeriod::Day => 86400,
			TimePeriod::Week => 604800,
			TimePeriod::Month => 2592000, // 30 days
			TimePeriod::Year => 31536000, // 365 days
		}
	}

	/// Get the display name
	pub fn name(&self) -> &'static str {
		match self {
			TimePeriod::Second => "second",
			TimePeriod::Minute => "minute",
			TimePeriod::Hour => "hour",
			TimePeriod::Day => "day",
			TimePeriod::Week => "week",
			TimePeriod::Month => "month",
			TimePeriod::Year => "year",
		}
	}
}
