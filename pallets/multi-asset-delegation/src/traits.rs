// This file is part of Tangle.
// Copyright (C) 2022-2024 Webb Technologies Inc.
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
use super::*;
use crate::Config;
use frame_support::traits::Currency;
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_runtime::RuntimeDebug;

pub trait ServiceManager<AccountId, Balance> {
	/// List active services for the given account ID.
	fn list_active_services(account: &AccountId) -> Vec<Service>;

	/// List service rewards for the given account ID.
	fn list_service_reward(account: &AccountId) -> Balance;

	/// Check if the given account ID can exit.
	fn can_exit(account: &AccountId) -> bool;
}

// Example struct representing a Service
#[derive(Encode, Decode, Clone, RuntimeDebug, PartialEq, Eq)]
pub struct Service {
	pub service_id: u32,
	pub name: Vec<u8>,
	pub status: ServiceStatus,
}

// Example enum representing Service Status
#[derive(Encode, Decode, Clone, RuntimeDebug, PartialEq, Eq)]
pub enum ServiceStatus {
	Active,
	Inactive,
}
