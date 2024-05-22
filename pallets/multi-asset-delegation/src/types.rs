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
use frame_support::traits::Currency;
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_runtime::RuntimeDebug;

pub type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

pub type FeeInfoOf<T> = FeeInfo<BalanceOf<T>>;

/// This struct represents the preset fee to charge for different DKG job types
#[derive(PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Clone, MaxEncodedLen, Default)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct FeeInfo<Balance: MaxEncodedLen> {
	/// The base fee for all jobs.
	pub base_fee: Balance,

	/// The fee for handling the Circuit.
	pub circuit_fee: Balance,

	/// The fee for Proof generation.
	pub prove_fee: Balance,

	/// The storage fee per byte
	pub storage_fee_per_byte: Balance,
}

impl<Balance: MaxEncodedLen> FeeInfo<Balance> {
	/// Get the base fee.
	pub fn get_base_fee(self) -> Balance {
		self.base_fee
	}

	/// Get the circuit fee.
	pub fn get_circuit_fee(self) -> Balance {
		self.circuit_fee
	}

	/// Get the proof generation fee.
	pub fn get_prove_fee(self) -> Balance {
		self.prove_fee
	}

	/// Get the storage fee per byte
	pub fn get_storage_fee_per_byte(self) -> Balance {
		self.storage_fee_per_byte
	}
}


pub struct CountedDelegations<T: Config> {
	pub uncounted_stake: BalanceOf<T>,
	pub rewardable_delegations: Vec<Bond<T::AccountId, BalanceOf<T>>>,
}

#[derive(Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct Bond<AccountId, Balance> {
	pub owner: AccountId,
	pub amount: Balance,
}

impl<A: Decode, B: Default> Default for Bond<A, B> {
	fn default() -> Bond<A, B> {
		Bond {
			owner: A::decode(&mut sp_runtime::traits::TrailingZeroInput::zeroes())
				.expect("infinite length input; no invalid inputs for type; qed"),
			amount: B::default(),
		}
	}
}

impl<A, B: Default> Bond<A, B> {
	pub fn from_owner(owner: A) -> Self {
		Bond {
			owner,
			amount: B::default(),
		}
	}
}

impl<AccountId: Ord, Balance> Eq for Bond<AccountId, Balance> {}

impl<AccountId: Ord, Balance> Ord for Bond<AccountId, Balance> {
	fn cmp(&self, other: &Self) -> Ordering {
		self.owner.cmp(&other.owner)
	}
}

impl<AccountId: Ord, Balance> PartialOrd for Bond<AccountId, Balance> {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}

impl<AccountId: Ord, Balance> PartialEq for Bond<AccountId, Balance> {
	fn eq(&self, other: &Self) -> bool {
		self.owner == other.owner
	}
}

#[derive(Copy, Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
/// The activity status of the Validator
pub enum ValidatorStatus {
	/// Committed to be online and producing valid blocks (not equivocating)
	Active,
	/// Temporarily inactive and excused for inactivity
	Idle,
	/// Bonded until the inner round
	Leaving(RoundIndex),
}

impl Default for ValidatorStatus {
	fn default() -> ValidatorStatus {
		ValidatorStatus::Active
	}
}

#[derive(Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct BondWithAutoCompound<AccountId, Balance> {
	pub owner: AccountId,
	pub amount: Balance,
	pub auto_compound: Percent,
}

impl<A: Decode, B: Default> Default for BondWithAutoCompound<A, B> {
	fn default() -> BondWithAutoCompound<A, B> {
		BondWithAutoCompound {
			owner: A::decode(&mut sp_runtime::traits::TrailingZeroInput::zeroes())
				.expect("infinite length input; no invalid inputs for type; qed"),
			amount: B::default(),
			auto_compound: Percent::zero(),
		}
	}
}

#[derive(Encode, Decode, RuntimeDebug, TypeInfo)]
/// Snapshot of Validator state at the start of the round for which they are selected
pub struct ValidatorSnapshot<AccountId, Balance> {
	/// The total value locked by the Validator.
	pub bond: Balance,

	/// The rewardable delegations. This list is a subset of total delegators, where certain
	/// delegators are adjusted based on their scheduled
	/// [DelegationChange::Revoke] or [DelegationChange::Decrease] action.
	pub delegations: Vec<BondWithAutoCompound<AccountId, Balance>>,

	/// The total counted value locked for the Validator, including the self bond + total staked by
	/// top delegators.
	pub total: Balance,
}

impl<A: PartialEq, B: PartialEq> PartialEq for ValidatorSnapshot<A, B> {
	fn eq(&self, other: &Self) -> bool {
		let must_be_true = self.bond == other.bond && self.total == other.total;
		if !must_be_true {
			return false;
		}
		for (
			BondWithAutoCompound {
				owner: o1,
				amount: a1,
				auto_compound: c1,
			},
			BondWithAutoCompound {
				owner: o2,
				amount: a2,
				auto_compound: c2,
			},
		) in self.delegations.iter().zip(other.delegations.iter())
		{
			if o1 != o2 || a1 != a2 || c1 != c2 {
				return false;
			}
		}
		true
	}
}

impl<A, B: Default> Default for ValidatorSnapshot<A, B> {
	fn default() -> ValidatorSnapshot<A, B> {
		ValidatorSnapshot {
			bond: B::default(),
			delegations: Vec::new(),
			total: B::default(),
		}
	}
}

#[derive(Clone, Default, Encode, Decode, RuntimeDebug, TypeInfo)]
/// Info needed to make delayed payments to stakers after round end
pub struct DelayedPayout<Balance> {
	/// Total round reward (result of compute_issuance() at round end)
	pub round_issuance: Balance,
	/// The total inflation paid this round to stakers (e.g. less parachain bond fund)
	pub total_staking_reward: Balance,
	/// Snapshot of Validator commission rate at the end of the round
	pub Validator_commission: Perbill,
}

#[derive(PartialEq, Encode, Decode, RuntimeDebug, TypeInfo)]
/// Capacity status for top or bottom delegations
pub enum CapacityStatus {
	/// Reached capacity
	Full,
	/// Empty aka contains no delegations
	Empty,
	/// Partially full (nonempty and not full)
	Partial,
}