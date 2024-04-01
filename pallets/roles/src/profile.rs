// This file is part of Webb.
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

use crate::{BalanceOf, Config};
use frame_support::pallet_prelude::*;
use sp_runtime::traits::Zero;
use sp_runtime::Saturating;
use sp_std::vec::Vec;
use tangle_primitives::roles::RoleType;

#[derive(
	PartialEqNoBound,
	EqNoBound,
	CloneNoBound,
	Encode,
	Decode,
	RuntimeDebugNoBound,
	TypeInfo,
	MaxEncodedLen,
)]
#[scale_info(skip_type_params(T))]
pub struct Record<T: Config> {
	pub role: RoleType,
	pub amount: Option<BalanceOf<T>>,
}

#[derive(
	PartialEqNoBound,
	EqNoBound,
	CloneNoBound,
	Encode,
	Decode,
	RuntimeDebugNoBound,
	TypeInfo,
	MaxEncodedLen,
)]
#[scale_info(skip_type_params(T))]
pub struct IndependentRestakeProfile<T: Config> {
	pub records: BoundedVec<Record<T>, T::MaxRolesPerAccount>,
}

#[derive(
	PartialEqNoBound,
	EqNoBound,
	CloneNoBound,
	Encode,
	Decode,
	RuntimeDebugNoBound,
	TypeInfo,
	MaxEncodedLen,
)]
#[scale_info(skip_type_params(T))]
pub struct SharedRestakeProfile<T: Config> {
	pub records: BoundedVec<Record<T>, T::MaxRolesPerAccount>,
	pub amount: BalanceOf<T>,
}

#[derive(
	PartialEqNoBound,
	EqNoBound,
	CloneNoBound,
	Encode,
	Decode,
	RuntimeDebugNoBound,
	TypeInfo,
	MaxEncodedLen,
)]
#[scale_info(skip_type_params(T))]
pub enum Profile<T: Config> {
	Independent(IndependentRestakeProfile<T>),
	Shared(SharedRestakeProfile<T>),
}

impl<T: Config> Profile<T> {
	/// Checks if the profile is an independent profile.
	pub fn is_independent(&self) -> bool {
		matches!(self, Profile::Independent(_))
	}

	/// Checks if the profile is a shared profile.
	pub fn is_shared(&self) -> bool {
		matches!(self, Profile::Shared(_))
	}

	/// Returns the total profile restake.
	pub fn get_total_profile_restake(&self) -> BalanceOf<T> {
		match self {
			Profile::Independent(profile) => {
				profile.records.iter().fold(Zero::zero(), |acc, record| {
					acc.saturating_add(record.amount.unwrap_or_default())
				})
			},
			Profile::Shared(profile) => profile.amount,
		}
	}

	/// Returns staking record details containing role metadata and restake amount.
	pub fn get_records(&self) -> Vec<Record<T>> {
		match self {
			Profile::Independent(profile) => profile.records.clone().into_inner(),
			Profile::Shared(profile) => profile.records.clone().into_inner(),
		}
	}

	/// Returns roles in the profile.
	pub fn get_roles(&self) -> Vec<RoleType> {
		match self {
			Profile::Independent(profile) => {
				profile.records.iter().map(|record| record.role).collect()
			},
			Profile::Shared(profile) => profile.records.iter().map(|record| record.role).collect(),
		}
	}

	/// Checks if the profile contains given role.
	pub fn has_role(&self, role_type: RoleType) -> bool {
		match self {
			Profile::Independent(profile) => {
				profile.records.iter().any(|record| record.role == role_type)
			},
			Profile::Shared(profile) => {
				profile.records.iter().any(|record| record.role == role_type)
			},
		}
	}

	/// Removes given role from the profile.
	pub fn remove_role_from_profile(&mut self, role_type: RoleType) {
		match self {
			Profile::Independent(profile) => {
				profile.records.retain(|record| record.role != role_type);
			},
			Profile::Shared(profile) => {
				profile.records.retain(|record| record.role != role_type);
			},
		}
	}

	/// Return roles from current profile removed in updated profile.
	pub fn get_removed_roles(&self, updated_profile: &Profile<T>) -> Vec<RoleType> {
		// Get the roles from the current profile.
		let roles = self.get_roles();
		let updated_roles = updated_profile.get_roles();
		// Returns roles in current profile that have been removed in updated profile.
		roles
			.iter()
			.filter_map(|role| {
				let updated_role = updated_roles.iter().find(|updated_role| *updated_role == role);
				if updated_role.is_none() {
					Some(role.clone())
				} else {
					None
				}
			})
			.collect()
	}
}
