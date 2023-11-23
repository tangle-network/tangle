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
use frame_support::{dispatch::Vec, pallet_prelude::*};

/// Role type to be used in the system.
#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq, TypeInfo)]
pub enum RoleType {
	Tss,
	ZkSaas,
}

impl RoleType {
	/// Checks if the role type is a TSS role.
	pub fn is_tss(&self) -> bool {
		matches!(self, RoleType::Tss)
	}

	/// Checks if the role type is a Zk-Saas role.
	pub fn is_zksaas(&self) -> bool {
		matches!(self, RoleType::ZkSaas)
	}
}

/// Metadata associated with a role type.
#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq, TypeInfo)]
pub enum RoleTypeMetadata {
	Tss(TssRoleMetadata),
	ZkSaas(ZkSaasRoleMetadata),
}

impl RoleTypeMetadata {
	/// Return type of role.
	pub fn get_role_type(&self) -> RoleType {
		match self {
			RoleTypeMetadata::Tss(_) => RoleType::Tss,
			RoleTypeMetadata::ZkSaas(_) => RoleType::ZkSaas,
		}
	}

	pub fn get_authority_key(&self) -> Vec<u8> {
		match self {
			RoleTypeMetadata::Tss(metadata) => metadata.authority_key.clone(),
			RoleTypeMetadata::ZkSaas(metadata) => metadata.authority_key.clone(),
		}
	}
}

/// Associated metadata needed for a DKG role
#[derive(Encode, Decode, Clone, Debug, PartialEq, Default, Eq, TypeInfo)]
pub struct TssRoleMetadata {
	/// The authority key associated with the role.
	authority_key: Vec<u8>,
}

/// Associated metadata needed for a zkSaas role
#[derive(Encode, Decode, Clone, Debug, PartialEq, Default, Eq, TypeInfo)]
pub struct ZkSaasRoleMetadata {
	/// The authority key associated with the role.
	// TODO : Expand this
	authority_key: Vec<u8>,
}

/// Role type to be used in the system.
#[derive(Encode, Decode, Clone, Copy, Debug, PartialEq, Eq, TypeInfo)]
pub enum ReStakingOption {
	// Re-stake all the staked funds for selected role.
	Full,
	// Re-stake only the given amount of funds for selected role.
	Custom(u64),
}
