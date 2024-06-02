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
use sp_std::collections::btree_map::BTreeMap;

pub mod delegator;
pub mod operator;
pub mod rewards;

pub use delegator::*;
pub use operator::*;
pub use rewards::*;

pub type RoundIndex = u32;

pub type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

pub type OperatorMetadataOf<T: Config> =
	OperatorMetadata<<T as frame_system::Config>::AccountId, BalanceOf<T>, T::AssetId>;

pub type OperatorSnapshotOf<T: Config> =
	OperatorSnapshot<<T as frame_system::Config>::AccountId, BalanceOf<T>, T::AssetId>;

pub type DelegatorMetadataOf<T: Config> =
	DelegatorMetadata<<T as frame_system::Config>::AccountId, BalanceOf<T>, T::AssetId>;
