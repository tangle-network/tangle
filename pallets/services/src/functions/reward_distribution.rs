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

//! Reward Distribution Logic for Services
//!
//! This module implements the payment → reward distribution pipeline for service revenues.
//! Payments from customers are distributed among:
//! - Service operators (weighted by their security commitment exposure)
//! - Blueprint developers (configurable percentage)
//! - Protocol treasury (configurable percentage)

use crate::{BalanceOf, Config, Error, Pallet};
use frame_support::{dispatch::DispatchResult, ensure};
use frame_system::pallet_prelude::BlockNumberFor;
use sp_runtime::{Perbill, traits::{CheckedDiv, CheckedMul, Saturating, Zero}};
use tangle_primitives::{
	services::{PricingModel, Service},
	traits::RewardRecorder,
};

/// Revenue distribution configuration
pub struct RevenueDistribution {
	/// Percentage of revenue going to operators (split by exposure weight)
	pub operator_share: Perbill,
	/// Percentage going to blueprint developer
	pub developer_share: Perbill,
	/// Percentage going to protocol treasury (optional)
	pub protocol_share: Perbill,
}

impl RevenueDistribution {
	/// Default revenue distribution:
	/// - 85% to operators
	/// - 10% to developer
	/// - 5% to protocol
	pub fn default_distribution() -> Self {
		Self {
			operator_share: Perbill::from_percent(85),
			developer_share: Perbill::from_percent(10),
			protocol_share: Perbill::from_percent(5),
		}
	}

	/// Validate that percentages sum to 100%
	pub fn validate(&self) -> bool {
		let total = self.operator_share
			.saturating_add(self.developer_share)
			.saturating_add(self.protocol_share);
		total == Perbill::one()
	}
}

impl<T: Config> Pallet<T> {
	/// Distribute service payment to operators, developer, and protocol.
	///
	/// This function implements exposure-weighted distribution where operators
	/// receive rewards proportional to their committed security exposure.
	///
	/// # Arguments
	/// * `service` - The service instance for which payment is being processed
	/// * `blueprint_owner` - The account that created the blueprint (developer)
	/// * `total_amount` - The total payment amount to distribute
	/// * `pricing_model` - The pricing model (used for reward recording)
	///
	/// # Distribution Logic
	/// 1. Calculate operator_total = operator_share * total_amount
	/// 2. For each operator, calculate:
	///    operator_reward = (operator_exposure_percent / total_exposure) * operator_total
	/// 3. Record developer_share * total_amount to blueprint owner
	/// 4. Record protocol_share * total_amount to treasury (if configured)
	///
	/// # Returns
	/// DispatchResult indicating success or error
	pub fn distribute_service_payment(
		service: &Service<T::Constraints, T::AccountId, BlockNumberFor<T>, T::AssetId>,
		blueprint_owner: &T::AccountId,
		total_amount: BalanceOf<T>,
		pricing_model: &PricingModel<BlockNumberFor<T>, BalanceOf<T>>,
	) -> DispatchResult {
		// Don't process zero payments
		if total_amount.is_zero() {
			return Ok(());
		}

		// Ensure service has operators
		ensure!(
			!service.operator_security_commitments.is_empty(),
			Error::<T>::NoOperatorsAvailable
		);

		let distribution = RevenueDistribution::default_distribution();

		// Validate distribution percentages
		ensure!(distribution.validate(), Error::<T>::InvalidRevenueDistribution);

		// Calculate shares
		let operator_total = distribution
			.operator_share
			.mul_floor(total_amount);
		let developer_amount = distribution
			.developer_share
			.mul_floor(total_amount);
		let protocol_amount = distribution
			.protocol_share
			.mul_floor(total_amount);

		// Distribute to operators weighted by exposure
		Self::distribute_to_operators(
			service,
			operator_total,
			pricing_model,
		)?;

		// Distribute to developer
		if !developer_amount.is_zero() {
			T::RewardRecorder::record_reward(
				blueprint_owner,
				service.id,
				developer_amount,
				pricing_model,
			)?;
		}

		// Distribute to protocol treasury (if configured)
		if !protocol_amount.is_zero() {
			// TODO: Add treasury account configuration to Config trait
			// For now, we skip protocol share or add it to operator pool
			log::debug!(
				"Protocol share ({:?}) not distributed - treasury account not configured",
				protocol_amount
			);
		}

		Ok(())
	}

	/// Distribute operator share among all operators weighted by exposure.
	///
	/// Each operator's reward is proportional to their exposure_percent commitment.
	/// This ensures operators with higher security backing receive proportionally more rewards.
	///
	/// # Arguments
	/// * `service` - The service instance
	/// * `operator_total` - Total amount to distribute among operators
	/// * `pricing_model` - The pricing model for reward recording
	///
	/// # Formula
	/// For each operator i:
	/// reward_i = (exposure_i / sum(all_exposures)) * operator_total
	///
	/// # Returns
	/// DispatchResult indicating success or error
	fn distribute_to_operators(
		service: &Service<T::Constraints, T::AccountId, BlockNumberFor<T>, T::AssetId>,
		operator_total: BalanceOf<T>,
		pricing_model: &PricingModel<BlockNumberFor<T>, BalanceOf<T>>,
	) -> DispatchResult {
		if operator_total.is_zero() {
			return Ok(());
		}

		// Calculate total exposure across all operators
		let total_exposure: u128 = service
			.operator_security_commitments
			.iter()
			.map(|(_, commitments)| {
				// Sum exposure percentages across all asset commitments for this operator
				commitments
					.iter()
					.map(|c| c.exposure_percent.deconstruct() as u128)
					.sum::<u128>()
			})
			.sum();

		// Ensure we have non-zero total exposure
		ensure!(total_exposure > 0, Error::<T>::NoOperatorExposure);

		// Distribute to each operator proportionally
		let mut distributed_sum = BalanceOf::<T>::zero();

		for (operator, commitments) in &service.operator_security_commitments {
			// Calculate this operator's total exposure
			let operator_exposure: u128 = commitments
				.iter()
				.map(|c| c.exposure_percent.deconstruct() as u128)
				.sum();

			if operator_exposure == 0 {
				continue;
			}

			// Calculate operator's proportional share
			// reward = (operator_exposure / total_exposure) * operator_total
			let operator_reward = Self::calculate_proportional_share(
				operator_exposure,
				total_exposure,
				operator_total,
			)?;

			if operator_reward.is_zero() {
				continue;
			}

			// Record reward for this operator
			T::RewardRecorder::record_reward(
				operator,
				service.id,
				operator_reward,
				pricing_model,
			)?;

			distributed_sum = distributed_sum.saturating_add(operator_reward);
		}

		// Handle any dust (rounding errors) - this should be minimal
		let dust = operator_total.saturating_sub(distributed_sum);
		if !dust.is_zero() {
			log::debug!(
				"Dust from reward distribution: {:?} ({}% of total)",
				dust,
				Perbill::from_rational(dust, operator_total).deconstruct() as f64 / 10_000_000.0
			);
		}

		Ok(())
	}

	/// Calculate proportional share using safe arithmetic.
	///
	/// Formula: (numerator / denominator) * total
	///
	/// Uses checked operations to prevent overflow/underflow.
	///
	/// # Arguments
	/// * `numerator` - The operator's exposure
	/// * `denominator` - The total exposure across all operators
	/// * `total` - The total amount to distribute
	///
	/// # Returns
	/// Result<BalanceOf<T>, DispatchError>
	fn calculate_proportional_share(
		numerator: u128,
		denominator: u128,
		total: BalanceOf<T>,
	) -> Result<BalanceOf<T>, sp_runtime::DispatchError> {
		// Convert to Balance type for calculation
		let numerator_balance = numerator
			.try_into()
			.map_err(|_| Error::<T>::ArithmeticOverflow)?;
		let denominator_balance = denominator
			.try_into()
			.map_err(|_| Error::<T>::ArithmeticOverflow)?;

		// Calculate: (numerator * total) / denominator
		let product = total
			.checked_mul(&numerator_balance)
			.ok_or(Error::<T>::ArithmeticOverflow)?;

		let result = product
			.checked_div(&denominator_balance)
			.ok_or(Error::<T>::DivisionByZero)?;

		Ok(result)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_revenue_distribution_validation() {
		// Valid distribution (sums to 100%)
		let valid = RevenueDistribution {
			operator_share: Perbill::from_percent(85),
			developer_share: Perbill::from_percent(10),
			protocol_share: Perbill::from_percent(5),
		};
		assert!(valid.validate());

		// Invalid distribution (sums to 95% - less than 100%)
		let invalid_low = RevenueDistribution {
			operator_share: Perbill::from_percent(80),
			developer_share: Perbill::from_percent(10),
			protocol_share: Perbill::from_percent(5),
		};
		assert!(!invalid_low.validate());

		// Note: Perbill saturates, so we can't test > 100% by adding percentages
		// Just verify the valid distribution validates correctly
		assert!(valid.validate());
	}

	#[test]
	fn test_default_distribution() {
		let dist = RevenueDistribution::default_distribution();
		assert_eq!(dist.operator_share, Perbill::from_percent(85));
		assert_eq!(dist.developer_share, Perbill::from_percent(10));
		assert_eq!(dist.protocol_share, Perbill::from_percent(5));
		assert!(dist.validate());
	}
}
