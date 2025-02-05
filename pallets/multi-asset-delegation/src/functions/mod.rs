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

use crate::{Config, Pallet};
use frame_system::RawOrigin;
use sp_runtime::traits::BadOrigin;

pub mod delegate;
pub mod deposit;
pub mod evm;
pub mod operator;
pub mod session_manager;
pub mod slash;

/// Ensure that the origin `o` represents the current pallet (i.e. transaction).
/// Returns `Ok` if the origin is the current pallet, `Err` otherwise.
pub fn ensure_pallet<T: Config, OuterOrigin>(o: OuterOrigin) -> Result<T::AccountId, BadOrigin>
where
	OuterOrigin: Into<Result<RawOrigin<T::AccountId>, OuterOrigin>>,
{
	match o.into() {
		Ok(RawOrigin::Signed(t)) if t == Pallet::<T>::pallet_account() => Ok(t),
		_ => Err(BadOrigin),
	}
}
