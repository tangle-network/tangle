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

//! Payment-related traits and interfaces.

use sp_runtime::traits::{Zero, Saturating, CheckedMul, CheckedAdd};
use sp_std::ops::Div;
use frame_support::pallet_prelude::*;
use sp_runtime::DispatchResult;
use sp_std::vec::Vec;

use super::super::types::PricingModel;
use super::types::{BillingCalculation, ServiceBillingState};
use crate::services::types::ServiceId;

/// Helper trait for billing operations
pub trait BillingOperations<BlockNumber, Balance> {
    /// Calculate the billing amount based on the pricing model and current state
    fn calculate_billing(
        &self,
        current_block: BlockNumber,
        billing_state: &ServiceBillingState<BlockNumber>,
    ) -> BillingCalculation<Balance>;

    /// Process billing and update the billing state
    fn process_billing(
        &self,
        current_block: BlockNumber,
        billing_state: &mut ServiceBillingState<BlockNumber>,
    ) -> Option<Balance>;
}

impl<BlockNumber, Balance> BillingOperations<BlockNumber, Balance> for PricingModel<BlockNumber, Balance>
where
    BlockNumber: Copy + PartialOrd + Saturating + Zero + Div<Output = BlockNumber>,
    Balance: Copy + Zero + Saturating + CheckedMul + CheckedAdd + PartialOrd + From<u64>,
{
    fn calculate_billing(
        &self,
        current_block: BlockNumber,
        billing_state: &ServiceBillingState<BlockNumber>,
    ) -> BillingCalculation<Balance> {
        use super::types::{BillingTrigger, BillingSkipReason};
        
        match self {
            PricingModel::PayOnce { .. } => {
                self.calculate_pay_once_billing(billing_state.last_billed)
                    .unwrap_or_else(|| BillingCalculation {
                        amount: Balance::zero(),
                        trigger: BillingTrigger::Activation,
                        should_bill: false,
                        skip_reason: Some(BillingSkipReason::AlreadyBilled),
                    })
            }
            PricingModel::Subscription { .. } => {
                self.calculate_subscription_billing(current_block, billing_state.last_billed)
                    .unwrap_or_else(|| BillingCalculation {
                        amount: Balance::zero(),
                        trigger: BillingTrigger::BlockInterval,
                        should_bill: false,
                        skip_reason: Some(BillingSkipReason::SubscriptionEnded),
                    })
            }
            PricingModel::EventDriven { .. } => {
                self.calculate_event_driven_billing(&billing_state.event_log)
                    .unwrap_or_else(|| BillingCalculation {
                        amount: Balance::zero(),
                        trigger: BillingTrigger::EventSubmission,
                        should_bill: false,
                        skip_reason: Some(BillingSkipReason::NoEvents),
                    })
            }
        }
    }

    fn process_billing(
        &self,
        current_block: BlockNumber,
        billing_state: &mut ServiceBillingState<BlockNumber>,
    ) -> Option<Balance> {
        let calculation = self.calculate_billing(current_block, billing_state);
        
        if calculation.should_bill && calculation.amount > Balance::zero() {
            // Update billing state
            billing_state.mark_billed(current_block);
            
            // Reset counters based on pricing model
            match self {
                PricingModel::EventDriven { .. } => {
                    billing_state.clear_events();
                }
                _ => {}
            }
            
            Some(calculation.amount)
        } else {
            None
        }
    }
}

/// Trait for payment validation and verification
pub trait PaymentValidator<Balance> {
    /// Validate that a payment amount is within acceptable limits
    fn validate_payment_amount(&self, amount: Balance) -> bool;
    
    /// Validate that the payment timing is appropriate
    fn validate_payment_timing(&self) -> bool;
}

/// Trait for payment fee calculation
pub trait FeeCalculator<Balance> {
    /// Calculate transaction fees for a payment
    fn calculate_transaction_fee(&self, amount: Balance) -> Balance;
    
    /// Calculate platform fees for a service
    fn calculate_platform_fee(&self, amount: Balance) -> Balance;
}

/// Trait for payment history tracking
pub trait PaymentHistory<AccountId, ServiceId, Balance, BlockNumber> {
    /// Record a payment in the history
    fn record_payment(
        &mut self,
        operator: AccountId,
        service_id: ServiceId,
        amount: Balance,
        block: BlockNumber,
    );
    
    /// Get payment history for an operator
    fn get_operator_payments(&self, operator: &AccountId) -> Vec<(ServiceId, Balance, BlockNumber)>;
    
    /// Get payment history for a service
    fn get_service_payments(&self, service_id: ServiceId) -> Vec<(AccountId, Balance, BlockNumber)>;
}

/// Trait for payment scheduling
pub trait PaymentScheduler<BlockNumber> {
    /// Schedule a payment for a future block
    fn schedule_payment(&mut self, block: BlockNumber);
    
    /// Check if a payment is scheduled for the current block
    fn is_payment_scheduled(&self, block: BlockNumber) -> bool;
    
    /// Get the next scheduled payment block
    fn next_payment_block(&self) -> Option<BlockNumber>;
} 