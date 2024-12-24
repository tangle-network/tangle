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

use crate::Config;
use frame_support::traits::Currency;
use frame_system::pallet_prelude::BlockNumberFor;
use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;
use sp_std::{collections::btree_map::BTreeMap, vec::Vec};
use tangle_primitives::{services::Asset, types::RoundIndex};

/// Represents different types of rewards a user can earn
#[derive(Clone, Encode, Decode, RuntimeDebug, TypeInfo, PartialEq, Eq)]
pub struct UserRewards<Balance, BlockNumber> {
	/// Rewards earned from restaking (in TNT)
	pub restaking_rewards: Balance,
	/// Boost rewards information
	pub boost_rewards: BoostInfo<Balance, BlockNumber>,
	/// Service rewards in their respective assets
	pub service_rewards: Balance,
}

pub type UserRewardsOf<T> = UserRewards<BalanceOf<T>, BlockNumberFor<T>>;

/// Information about a user's boost rewards
#[derive(Clone, Encode, Decode, RuntimeDebug, TypeInfo, PartialEq, Eq)]
pub struct BoostInfo<Balance, BlockNumber> {
	/// Amount of boost rewards
	pub amount: Balance,
	/// Multiplier for the boost (e.g. OneMonth = 1x, TwoMonths = 2x)
	pub multiplier: LockMultiplier,
	/// Block number when the boost expires
	pub expiry: BlockNumber,
}

impl<Balance: Default, BlockNumber: Default> Default for BoostInfo<Balance, BlockNumber> {
	fn default() -> Self {
		Self {
			amount: Balance::default(),
			multiplier: LockMultiplier::OneMonth,
			expiry: BlockNumber::default(),
		}
	}
}

impl<Balance: Default, BlockNumber: Default> Default for UserRewards<Balance, BlockNumber> {
	fn default() -> Self {
		Self {
			restaking_rewards: Balance::default(),
			boost_rewards: BoostInfo::default(),
			service_rewards: Balance::default(),
		}
	}
}

pub type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

/// Lock multiplier for rewards, representing months of lock period
#[derive(Clone, Copy, Encode, Decode, RuntimeDebug, TypeInfo, PartialEq, Eq)]
pub enum LockMultiplier {
	/// One month lock period (1x multiplier)
	OneMonth = 1,
	/// Two months lock period (2x multiplier)
	TwoMonths = 2,
	/// Three months lock period (3x multiplier)
	ThreeMonths = 3,
	/// Six months lock period (6x multiplier)
	SixMonths = 6,
}

impl Default for LockMultiplier {
	fn default() -> Self {
		Self::OneMonth
	}
}

impl LockMultiplier {
	/// Get the multiplier value
	pub fn value(&self) -> u32 {
		*self as u32
	}
}
