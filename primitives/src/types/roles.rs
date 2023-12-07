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
use parity_scale_codec::alloc::string::ToString;
use scale_info::prelude::string::String;
use sp_arithmetic::Percent;
use sp_std::ops::Add;

/// Role type to be used in the system.
#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq, TypeInfo, PartialOrd, Ord)]
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
	pub authority_key: Vec<u8>,
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

/// Represents the reward distribution percentages for validators in a key generation process.
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

#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq, TypeInfo)]
pub struct Record {
	pub metadata: RoleTypeMetadata,
	pub amount: Option<u64>,
}

#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq, TypeInfo)]
pub struct IndependentReStakeProfile {
	pub records: Vec<Record>,
}

#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq, TypeInfo)]
pub struct SharedReStakeProfile {
	pub records: Vec<Record>,
	pub amount: u64,
}

#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq, TypeInfo)]
pub enum Profile {
	Independent(IndependentReStakeProfile),
	Shared(SharedReStakeProfile),
}

impl Profile {
	pub fn is_independent(&self) -> bool {
		matches!(self, Profile::Independent(_))
	}

	pub fn is_shared(&self) -> bool {
		matches!(self, Profile::Shared(_))
	}

	pub fn get_total_profile_stake(&self) -> u64 {
		match self {
			Profile::Independent(profile) =>
				profile.records.iter().fold(0, |acc, record| acc + record.amount.unwrap_or(0)),
			Profile::Shared(profile) => profile.amount,
		}
	}

	pub fn get_records(&self) -> Vec<Record> {
		match self {
			Profile::Independent(profile) => profile.records.clone(),
			Profile::Shared(profile) => profile.records.clone(),
		}
	}

	pub fn get_roles(&self) -> Vec<RoleType> {
		match self {
			Profile::Independent(profile) =>
				profile.records.iter().map(|record| record.metadata.get_role_type()).collect(),
			Profile::Shared(profile) =>
				profile.records.iter().map(|record| record.metadata.get_role_type()).collect(),
		}
	}

	pub fn has_role(&self, role_type: RoleType) -> bool {
		match self {
			Profile::Independent(profile) => profile
				.records
				.iter()
				.any(|record| record.metadata.get_role_type() == role_type),
			Profile::Shared(profile) => profile
				.records
				.iter()
				.any(|record| record.metadata.get_role_type() == role_type),
		}
	}

	pub fn remove_role_from_profile(&mut self, role_type: RoleType) {
		match self {
			Profile::Independent(profile) => {
				profile.records.retain(|record| record.metadata.get_role_type() != role_type);
			},
			Profile::Shared(profile) => {
				profile.records.retain(|record| record.metadata.get_role_type() != role_type);
			},
		}
	}

	pub fn is_profile_empty(&self) -> bool {
		match self {
			Profile::Independent(profile) => profile.records.is_empty(),
			Profile::Shared(profile) => profile.records.is_empty(),
		}
	}

	pub fn has_duplicate_roles(&self) -> bool {
		let records = self.get_records();
		let mut role_types = Vec::new();
		for record in records {
			if role_types.contains(&record.metadata.get_role_type()) {
				return true
			}
			role_types.push(record.metadata.get_role_type());
		}
		false
	}

	// check if roles have changed for any records in updated profile.
	pub fn has_roles_changed(&self, updated_profile: &Profile) -> bool {
		// use get_records to get the records from the profile.
		let records = self.get_records();
		let updated_records = updated_profile.get_records();
		// check if roles have changed for any records in updated profile.
		records.iter().any(|record| {
			let updated_record = updated_records.iter().find(|updated_record| {
				updated_record.metadata.get_role_type() == record.metadata.get_role_type()
			});
			updated_record.is_none()
		})
	}

	// Get records removed in updated profile.
	pub fn get_removed_records(&self, updated_profile: &Profile) -> Vec<Record> {
		// use get_records to get the records from the profile.
		let records = self.get_records();
		let updated_records = updated_profile.get_records();
		// check if roles have changed for any records in updated profile.
		records
			.iter()
			.filter_map(|record| {
				let updated_record = updated_records.iter().find(|updated_record| {
					updated_record.metadata.get_role_type() == record.metadata.get_role_type()
				});
				if updated_record.is_none() {
					Some(record.clone())
				} else {
					None
				}
			})
			.collect()
	}

	// Get new records added to updated profile.
	pub fn get_newly_added_records(&self, updated_profile: &Profile) -> Vec<Record> {
		// use get_records to get the records from the profile.
		let records = self.get_records();
		let updated_records = updated_profile.get_records();
		// check if roles have changed for any records in updated profile.
		updated_records
			.iter()
			.filter_map(|updated_record| {
				let record = records.iter().find(|record| {
					updated_record.metadata.get_role_type() == record.metadata.get_role_type()
				});
				if record.is_none() {
					Some(updated_record.clone())
				} else {
					None
				}
			})
			.collect()
	}
}
