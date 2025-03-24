//! Pricing models for cloud services
//!
//! This module defines the pricing models for Tangle cloud services.

use core::fmt;
use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

use crate::error::PricingError;
use crate::types::{Price, ResourceRequirement, ResourceUnit, ServiceCategory, TimePeriod};

/// Pricing tier for tiered pricing models
#[derive(Debug, Clone, Encode, Decode, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct PricingTier {
	/// Start value for this tier (inclusive)
	pub start: u128,
	/// End value for this tier (exclusive)
	pub end: Option<u128>,
	/// Price per unit in this tier
	pub price_per_unit: Price,
}

/// Resource-based pricing configuration for a specific resource
#[derive(Debug, Clone, Encode, Decode, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct ResourcePricing {
	/// Resource unit being priced
	pub unit: ResourceUnit,
	/// Price per unit of resource
	pub price_per_unit: Price,
	/// Optional minimum quantity for pricing
	pub min_quantity: Option<u128>,
	/// Optional maximum quantity supported
	pub max_quantity: Option<u128>,
	/// Optional time period for recurring pricing
	pub time_period: Option<TimePeriod>,
	/// Optional tiers for tiered pricing
	pub tiers: Option<Vec<PricingTier>>,
}

impl ResourcePricing {
	/// Create a new simple resource pricing with price per unit
	pub fn new(unit: ResourceUnit, price_per_unit: Price) -> Self {
		Self {
			unit,
			price_per_unit,
			min_quantity: None,
			max_quantity: None,
			time_period: None,
			tiers: None,
		}
	}

	/// Set the time period for recurring pricing
	pub fn with_time_period(mut self, period: TimePeriod) -> Self {
		self.time_period = Some(period);
		self
	}

	/// Set minimum quantity
	pub fn with_min_quantity(mut self, min: u128) -> Self {
		self.min_quantity = Some(min);
		self
	}

	/// Set maximum quantity
	pub fn with_max_quantity(mut self, max: u128) -> Self {
		self.max_quantity = Some(max);
		self
	}

	/// Set pricing tiers
	pub fn with_tiers(mut self, tiers: Vec<PricingTier>) -> Self {
		self.tiers = Some(tiers);
		self
	}

	/// Calculate price for a given quantity of this resource
	pub fn calculate_price(&self, quantity: u128) -> Result<Price, PricingError> {
		// Check quantity against min/max
		if let Some(min) = self.min_quantity {
			if quantity < min {
				return Err(PricingError::QuantityBelowMinimum(quantity));
			}
		}

		if let Some(max) = self.max_quantity {
			if quantity > max {
				return Err(PricingError::QuantityAboveMaximum(quantity));
			}
		}

		// Calculate price based on tiered or flat pricing
		if let Some(tiers) = &self.tiers {
			// Find the applicable tier(s)
			let mut applicable_tiers = Vec::new();
			for tier in tiers {
				if quantity >= tier.start && (tier.end.is_none() || quantity < tier.end.unwrap()) {
					applicable_tiers.push(tier);
				}
			}

			if applicable_tiers.is_empty() {
				return Err(PricingError::CalculationError(
					"No applicable pricing tier found".to_string(),
				));
			}

			// In tiered pricing, we might need to calculate prices for different tiers
			// For simplicity, we just use the first applicable tier in this implementation
			let tier = &applicable_tiers[0];

			// Price = quantity * price_per_unit
			let price = Price {
				value: quantity.saturating_mul(tier.price_per_unit.value) / 1_000_000,
				token: tier.price_per_unit.token.clone(),
			};

			Ok(price)
		} else {
			// For flat pricing, we simply multiply quantity by price per unit
			let price = Price {
				value: quantity.saturating_mul(self.price_per_unit.value) / 1_000_000,
				token: self.price_per_unit.token.clone(),
			};

			Ok(price)
		}
	}
}

/// Pricing model types supported by the system
#[derive(Debug, Clone, PartialEq, Encode, Decode, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum PricingModelType {
	/// Fixed price regardless of usage
	Fixed,
	/// Price based on resource usage
	Usage,
	/// Tiered pricing with different rates at different usage levels
	Tiered,
}

#[cfg(feature = "std")]
impl fmt::Display for PricingModelType {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			PricingModelType::Fixed => write!(f, "Fixed"),
			PricingModelType::Usage => write!(f, "Usage-based"),
			PricingModelType::Tiered => write!(f, "Tiered"),
		}
	}
}

/// Main pricing model that combines different pricing strategies
#[derive(Debug, Clone, Encode, Decode, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct PricingModel {
	/// Type of pricing model
	pub model_type: PricingModelType,
	/// Name of the pricing model
	pub name: String,
	/// Description of the pricing model
	pub description: Option<String>,
	/// Target service category
	pub category: ServiceCategory,
	/// Base price (for fixed pricing or minimum charge)
	pub base_price: Option<Price>,
	/// Resource-specific pricing
	pub resource_pricing: Vec<ResourcePricing>,
	/// Time period for recurring charges
	pub billing_period: Option<TimePeriod>,
}

impl PricingModel {
	/// Create a new pricing model
	pub fn new(
		model_type: PricingModelType,
		name: impl Into<String>,
		category: ServiceCategory,
	) -> Self {
		Self {
			model_type,
			name: name.into(),
			description: None,
			category,
			base_price: None,
			resource_pricing: Vec::new(),
			billing_period: None,
		}
	}

	/// Add description
	pub fn with_description(mut self, description: impl Into<String>) -> Self {
		self.description = Some(description.into());
		self
	}

	/// Add base price
	pub fn with_base_price(mut self, price: Price) -> Self {
		self.base_price = Some(price);
		self
	}

	/// Add resource pricing
	pub fn with_resource_pricing(mut self, pricing: ResourcePricing) -> Self {
		self.resource_pricing.push(pricing);
		self
	}

	/// Add billing period
	pub fn with_billing_period(mut self, period: TimePeriod) -> Self {
		self.billing_period = Some(period);
		self
	}
}

/// Pricing strategy trait for implementing different pricing algorithms
pub trait PricingStrategy {
	/// Calculate the price for a service based on resource requirements
	fn calculate_price(
		&self,
		requirements: &[ResourceRequirement],
		model: &PricingModel,
	) -> Result<Price, PricingError>;
}

/// Create a standard fixed-price model
pub fn create_fixed_price_model(
	name: impl Into<String>,
	category: ServiceCategory,
	price: Price,
	period: TimePeriod,
) -> PricingModel {
	PricingModel::new(PricingModelType::Fixed, name, category)
		.with_base_price(price)
		.with_billing_period(period)
}

/// Create a standard usage-based pricing model
pub fn create_usage_model(
	name: impl Into<String>,
	category: ServiceCategory,
	resources: Vec<ResourcePricing>,
	period: TimePeriod,
) -> PricingModel {
	let mut model =
		PricingModel::new(PricingModelType::Usage, name, category).with_billing_period(period);

	for resource in resources {
		model = model.with_resource_pricing(resource);
	}

	model
}

/// Create a standard tiered pricing model
pub fn create_tiered_model(
	name: impl Into<String>,
	category: ServiceCategory,
	resources: Vec<ResourcePricing>,
	period: TimePeriod,
) -> PricingModel {
	let mut model =
		PricingModel::new(PricingModelType::Tiered, name, category).with_billing_period(period);

	for resource in resources {
		model = model.with_resource_pricing(resource);
	}

	model
}

/// Recommends a basic pricing model based on the service category
pub fn recommend_model(
	resources: &[ResourceRequirement],
	category: ServiceCategory,
) -> PricingModel {
	match category {
		ServiceCategory::Compute => {
			let cpu_price = Price::new(0.03, "TGL");
			let memory_price = Price::new(0.005, "TGL");

			let mut model =
				PricingModel::new(PricingModelType::Usage, "Standard Compute Pricing", category)
					.with_description("Pay only for what you use with our standard compute pricing")
					.with_billing_period(TimePeriod::Hour);

			model = model.with_resource_pricing(
				ResourcePricing::new(ResourceUnit::CPU, cpu_price)
					.with_time_period(TimePeriod::Hour),
			);

			model = model.with_resource_pricing(
				ResourcePricing::new(ResourceUnit::MemoryMB, memory_price)
					.with_time_period(TimePeriod::Hour),
			);

			model
		},
		ServiceCategory::Storage => {
			let storage_price = Price::new(0.00002, "TGL");

			PricingModel::new(PricingModelType::Usage, "Standard Storage Pricing", category)
				.with_description("Pay only for the storage you use")
				.with_billing_period(TimePeriod::Month)
				.with_resource_pricing(
					ResourcePricing::new(ResourceUnit::StorageMB, storage_price)
						.with_time_period(TimePeriod::Month),
				)
		},
		_ => {
			// Default model for other categories
			PricingModel::new(
				PricingModelType::Fixed,
				format!("Standard {} Pricing", category),
				category,
			)
			.with_description("Standard pricing model")
			.with_base_price(Price::new(10.0, "TGL"))
			.with_billing_period(TimePeriod::Month)
		},
	}
}
