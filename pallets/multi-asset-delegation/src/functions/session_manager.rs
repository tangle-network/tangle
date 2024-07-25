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
use crate::{types::*, Pallet};

use frame_support::pallet_prelude::DispatchResult;

impl<T: Config> Pallet<T> {
	pub fn handle_round_change() -> DispatchResult {
		// Increment the current round
		CurrentRound::<T>::mutate(|round| *round += 1);
		let current_round = Self::current_round();

		// Iterate through all operators and build their snapshots
		for (operator, metadata) in Operators::<T>::iter() {
			// Create the operator snapshot
			let snapshot =
				OperatorSnapshot { bond: metadata.bond, delegations: metadata.delegations.clone() };

			// Store the snapshot in AtStake storage
			AtStake::<T>::insert(current_round, operator.clone(), snapshot);
		}

		Ok(())
	}
}
