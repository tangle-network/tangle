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
use tangle_primitives::types::rewards::UserRewards;
use tangle_primitives::{services::Asset, types::RoundIndex};

pub type UserRewardsOf<T> = UserRewards<
	BalanceOf<T>,
	BlockNumberFor<T>,
	<T as Config>::AssetId,
	<T as Config>::MaxUniqueServiceRewards,
>;

pub type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;