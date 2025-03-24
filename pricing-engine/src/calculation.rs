//! Price calculation logic for the pricing engine
//!
//! This module implements the core price calculation algorithms for various pricing models.

use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

use crate::error::{PricingError, Result};
use crate::models::{PricingModel, PricingModelType};
use crate::types::{Price, ResourceRequirement};

/// Context for price calculation
#[derive(Debug, Clone, Encode, Decode, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct PricingContext {
	/// Provider identifier
	pub provider_id: String,
}

/// Calculate service price based on resource requirements and pricing model
pub fn calculate_service_price(
	requirements: &ResourceRequirement,
	model: &PricingModel,
	context: &PricingContext,
) -> Result<u64> {
	// For a full implementation, we'd use a more sophisticated pricing algorithm
	// that considers all the resource requirements and pricing model details
	// But for this simplified example, we'll use a basic approach

	match model.model_type {
		PricingModelType::Fixed => {
			// For fixed pricing, simply return the base price
			if let Some(base_price) = &model.base_price {
				// Convert to u64, capping at u64::MAX
				let price_value = u64::try_from(base_price.value / 1_000_000).unwrap_or(u64::MAX);
				Ok(price_value)
			} else {
				Err(PricingError::InvalidModelConfiguration(
					"Fixed price model has no base price".to_string(),
				)
				.into())
			}
		},
		PricingModelType::Usage => {
			// For usage-based pricing, we need to calculate based on each resource
			let mut total_price: u128 = 0;

			// Add base price if present
			if let Some(base_price) = &model.base_price {
				total_price = total_price.saturating_add(base_price.value);
			}

			// Match resource requirement against pricing model
			let mut found_match = false;

			for resource_pricing in &model.resource_pricing {
				if resource_pricing.unit == requirements.unit {
					// Calculate price based on the quantity
					let resource_price =
						match resource_pricing.calculate_price(requirements.quantity) {
							Ok(price) => price.value,
							Err(e) => return Err(e.into()),
						};

					total_price = total_price.saturating_add(resource_price);
					found_match = true;
				}
			}

			if !found_match {
				return Err(PricingError::NoPricingModelForResource.into());
			}

			// Convert to u64, capping at u64::MAX if necessary
			let price_value = u64::try_from(total_price / 1_000_000).unwrap_or(u64::MAX);
			Ok(price_value)
		},
		PricingModelType::Tiered => {
			// Similar to usage, but with tier awareness
			// For this simplified example, we'll use the same approach as usage
			// but in a real implementation, we'd use a more sophisticated algorithm

			let mut total_price: u128 = 0;

			// Add base price if present
			if let Some(base_price) = &model.base_price {
				total_price = total_price.saturating_add(base_price.value);
			}

			// Match resource requirement against pricing model
			let mut found_match = false;

			for resource_pricing in &model.resource_pricing {
				if resource_pricing.unit == requirements.unit {
					// Calculate price based on the quantity and tiers
					let resource_price =
						match resource_pricing.calculate_price(requirements.quantity) {
							Ok(price) => price.value,
							Err(e) => return Err(e.into()),
						};

					total_price = total_price.saturating_add(resource_price);
					found_match = true;
				}
			}

			if !found_match {
				return Err(PricingError::NoPricingModelForResource.into());
			}

			// Convert to u64, capping at u64::MAX if necessary
			let price_value = u64::try_from(total_price / 1_000_000).unwrap_or(u64::MAX);
			Ok(price_value)
		},
	}
}
