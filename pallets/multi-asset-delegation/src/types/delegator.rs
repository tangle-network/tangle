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

#[derive(Clone, PartialEq, Encode, Decode, RuntimeDebug, TypeInfo, Default)]
pub enum DelegatorStatus {
	#[default]
	Active,
	/// Schedule exit to revoke all ongoing delegations
	LeavingScheduled(RoundIndex),
}

#[derive(Clone, PartialEq, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct UnstakeRequest<AssetId, Balance> {
	pub asset_id: AssetId,
	pub amount: Balance,
	pub requested_round: RoundIndex,
}

#[derive(Clone, PartialEq, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct UndelegateRequest<AssetId, Balance> {
	pub asset_id: AssetId,
	pub amount: Balance,
	pub requested_round: RoundIndex,
}

#[derive(Encode, Decode, RuntimeDebug, TypeInfo)]
/// Delegator state
pub struct DelegatorMetadata<AccountId, Balance, AssetId: Encode + Decode + TypeInfo> {
	pub deposits: BTreeMap<AssetId, Balance>,
	pub unstake_request: Option<UnstakeRequest<AssetId, Balance>>, // only one withdraw request at a time
	/// All current delegations
	pub delegations: Vec<Bond<AccountId, Balance, AssetId>>,
	/// Outstanding unstake
	pub undelegate_request: Option<UndelegateRequest<AssetId, Balance>>, // only one undelegate request at a time
	/// Status for this delegator
	pub status: DelegatorStatus,
}

impl<AccountId, Balance, AssetId: Encode + Decode + TypeInfo> Default
	for DelegatorMetadata<AccountId, Balance, AssetId>
{
	fn default() -> Self {
		DelegatorMetadata {
			deposits: BTreeMap::new(),
			delegations: Vec::new(),
			status: DelegatorStatus::default(),
			unstake_request: None,
			undelegate_request: None,
		}
	}
}

#[derive(Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct Deposit<AssetId, Balance> {
	pub amount: Balance,
	pub asset_id: AssetId,
}

#[derive(Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct Bond<AccountId, AssetId, Balance> {
	pub owner: AccountId,
	pub amount: Balance,
	pub asset_id: AssetId,
}
