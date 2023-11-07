// This file is part of Webb.
// Copyright (C) 2022 Webb Technologies Inc.
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

use frame_support::pallet_prelude::*;
use crate::jobs::JobKey;

/// Role type to be used in the system.
#[derive(Encode, Decode, Clone, Copy, Debug, PartialEq, Eq, TypeInfo)]
pub enum RoleType {
	Tss,
	ZkSaas,
}

impl RoleType {
	/// Checks if the role type is a TSS role.
	pub fn is_tss(self) -> bool {
		self == RoleType::Tss
	}

	/// Checks if the role type is a Zk-Saas role.
	pub fn is_zksaas(self) -> bool {
		self == RoleType::ZkSaas
	}
}

impl From<JobKey> for RoleType {
	fn from(job_key: JobKey) -> Self {
        match job_key {
			JobKey::DKG => RoleType::Tss,
			JobKey::DKGSignature => RoleType::Tss,
			JobKey::ZkSaasPhaseOne => RoleType::ZkSaas,
			JobKey::ZkSaasPhaseTwo => RoleType::ZkSaas,
		}
    }
}
