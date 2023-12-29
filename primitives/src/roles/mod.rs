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

use frame_support::pallet_prelude::*;
use parity_scale_codec::alloc::string::ToString;
use scale_info::prelude::string::String;
use sp_arithmetic::Percent;
use sp_std::{ops::Add, vec::Vec};

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

pub mod traits;
pub mod tss;
pub mod zksaas;

pub use tss::*;
pub use zksaas::*;

/// Role type to be used in the system.
#[derive(Encode, Decode, Copy, Clone, Debug, PartialEq, Eq, TypeInfo, PartialOrd, Ord)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum RoleType {
	/// TSS role type.
	Tss(ThresholdSignatureRoleType),
	/// Zk-SaaS role type.
	ZkSaaS(ZeroKnowledgeRoleType),
	/// Light client relaying role type.
	LightClientRelaying,
}

impl RoleType {
	pub fn is_dkg_tss(&self) -> bool {
		match self {
			RoleType::Tss(_) => true,
			_ => false,
		}
	}

	pub fn is_zksaas(&self) -> bool {
		match self {
			RoleType::ZkSaaS(_) => true,
			_ => false,
		}
	}

	pub fn is_light_client_relaying(&self) -> bool {
		match self {
			RoleType::LightClientRelaying => true,
			_ => false,
		}
	}

	pub fn to_u16(&self) -> u16 {
		let be_bytes: [u8; 2] = match self {
			RoleType::Tss(ThresholdSignatureRoleType::TssGG20) => [0, 1],
			RoleType::Tss(ThresholdSignatureRoleType::TssCGGMP) => [0, 2],
			RoleType::Tss(ThresholdSignatureRoleType::TssFrostSr25519) => [0, 3],
			RoleType::Tss(ThresholdSignatureRoleType::TssFrostP256) => [0, 4],
			RoleType::Tss(ThresholdSignatureRoleType::TssFrostSecp256k1) => [0, 5],
			RoleType::Tss(ThresholdSignatureRoleType::TssFrostRistretto255) => [0, 6],
			RoleType::Tss(ThresholdSignatureRoleType::TssFrostBabyJubJub) => [0, 7],
			RoleType::Tss(ThresholdSignatureRoleType::TssFrostEd25519) => [0, 8],
			RoleType::Tss(ThresholdSignatureRoleType::TssEdDSABabyJubJub) => [0, 9],
			RoleType::Tss(ThresholdSignatureRoleType::TssBls381) => [0, 10],
			RoleType::ZkSaaS(ZeroKnowledgeRoleType::ZkSaaSGroth16) => [1, 0],
			RoleType::ZkSaaS(ZeroKnowledgeRoleType::ZkSaaSMarlin) => [1, 1],
			RoleType::LightClientRelaying => [2, 0],
		};

		u16::from_be_bytes(be_bytes)
	}
}

/// Metadata associated with a role type.
#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum RoleTypeMetadata {
	Tss(TssRoleMetadata),
	ZkSaas(ZkSaasRoleMetadata),
	LightClientRelaying,
}

impl RoleTypeMetadata {
	/// Return type of role.
	pub fn get_role_type(&self) -> RoleType {
		match self {
			RoleTypeMetadata::Tss(metadata) => RoleType::Tss(metadata.role_type),
			RoleTypeMetadata::ZkSaas(metadata) => RoleType::ZkSaaS(metadata.role_type),
			RoleTypeMetadata::LightClientRelaying => RoleType::LightClientRelaying,
		}
	}

	pub fn get_authority_key(&self) -> Vec<u8> {
		match self {
			RoleTypeMetadata::Tss(metadata) => metadata.authority_key.clone(),
			RoleTypeMetadata::ZkSaas(metadata) => metadata.authority_key.clone(),
			_ => Vec::new(),
		}
	}
}

/// Represents the reward distribution percentages for validators in a key generation process.
#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct ValidatorRewardDistribution {
	/// The percentage share of the reward allocated for TSS
	tss_share: Percent,
	/// The percentage share of the reward allocated for the ZK-SaaS
	zksaas_share: Percent,
}

impl ValidatorRewardDistribution {
	pub fn try_new(tss_share: Percent, zksaas_share: Percent) -> Result<Self, String> {
		if !tss_share.add(zksaas_share).is_one() {
			return Err("Shares must add to One".to_string())
		}

		Ok(Self { tss_share, zksaas_share })
	}

	pub fn get_reward_distribution(self) -> (Percent, Percent) {
		(self.tss_share, self.zksaas_share)
	}
}
