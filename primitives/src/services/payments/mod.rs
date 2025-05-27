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

//! Payment system for the Services pallet.
//!
//! This module provides a comprehensive payment system that supports multiple pricing models:
//! - PayOnce: One-time payment services
//! - Subscription: Recurring payment services
//! - EventDriven: Payment based on events processed
//!
//! The system is organized into several modules:
//! - `types`: Core payment types and enums
//! - `billing`: Billing calculation logic and state management
//! - `traits`: Traits for payment operations and reward recording

pub mod billing;
pub mod traits;
pub mod types;

// Re-export commonly used types and traits
pub use traits::*;
pub use types::*;

// Re-export pricing models from parent types module for convenience
pub use super::types::PricingModel;
