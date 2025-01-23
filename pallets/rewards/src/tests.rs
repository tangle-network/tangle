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
// along with Tangle. If not, see <http://www.gnu.org/licenses/>.
use crate::{mock::*, types::*, AssetAction, Error, Pallet as RewardsPallet};
use frame_support::assert_err;
use sp_runtime::{DispatchError, Percent};
use tangle_primitives::services::Asset;

pub mod apy_calc;
pub mod claim;
pub mod reward_calc;
pub mod vault;
