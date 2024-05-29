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
use crate::types::*;
use crate::Pallet;
use frame_support::ensure;
use frame_support::pallet_prelude::DispatchResult;
use frame_support::traits::Get;
use frame_support::traits::ReservableCurrency;
use sp_runtime::DispatchError;

impl<T: Config> Pallet<T> {
	pub fn process_deposit(
		who: T::AccountId,
		asset_id: Option<T::AssetId>,
		amount: BalanceOf<T>,
	) -> DispatchResult {
		// Check if the user is already a delegator
		ensure!(!Delegators::<T>::contains_key(&who), Error::<T>::AlreadyDelegator);

		ensure!(amount >= T::MinOperatorBondAmount::get(), Error::<T>::BondTooLow);

		// Reserve the amount
		if let Some(asset_id) = asset_id {
			T::Fungibles::reserve(asset_id, &who, amount)?;

			// Update storage
			Delegators::<T>::mutate(&who, |maybe_metadata| {
				let metadata = maybe_metadata.get_or_insert_with(Default::default);
				metadata.deposits.entry(asset_id).and_modify(|e| *e += amount).or_insert(amount);
			});
		}

		// TODO : handle if TNT deposit
		Ok(())
	}

	fn process_schedule_unstake(
		who: &T::AccountId,
		asset_id: Option<T::AssetId>,
		amount: BalanceOf<T>,
	) -> DispatchResult {
		// Ensure there is enough deposited balance
		Delegators::<T>::mutate(&who, |maybe_metadata| {
			let metadata = maybe_metadata.as_mut().ok_or(Error::<T>::InsufficientBalance)?;
			let balance =
				metadata.deposits.get_mut(&asset_id).ok_or(Error::<T>::InsufficientBalance)?;
			ensure!(*balance >= amount, Error::<T>::InsufficientBalance);

			// Reduce the balance in deposits
			*balance -= amount;
			if *balance == Zero::zero() {
				metadata.deposits.remove(&asset_id.unwrap());
			}

			// Schedule the unstake
			let current_block = <frame_system::Pallet<T>>::block_number();
			UnstakingSchedules::<T>::mutate(&who, |schedules| {
				schedules.push((
					asset_id,
					amount,
					current_block + T::BlockNumber::from(2 * 7 * 14400u32),
				)); // assuming 2 weeks delay with 14400 blocks/day // TODO : Move to config
			});
		});

		Ok(())
	}

	fn process_execute_unstake(who: &T::AccountId) -> DispatchResult {
		let current_block = <frame_system::Pallet<T>>::block_number();

		UnstakingSchedules::<T>::mutate(&who, |schedules| {
			let mut remaining_schedules = vec![];
			let mut total_unstaked = BTreeMap::new();

			for (asset_id, amount, scheduled_block) in schedules.iter() {
				if &current_block >= scheduled_block {
					// Track the total unstaked amount for each asset
					*total_unstaked.entry(*asset_id).or_insert(Zero::zero()) += *amount;
				} else {
					remaining_schedules.push((*asset_id, *amount, *scheduled_block));
				}
			}

			// Update the schedules
			*schedules = remaining_schedules;

			// Unreserve the amounts
			for (asset_id, amount) in total_unstaked.into_iter() {
				T::Asset::unreserve(asset_id, &who, amount)?;
			}
		});

		Ok(())
	}

	fn process_cancel_unstake(who: &T::AccountId, asset_id: T::AssetId) -> DispatchResult {
		UnstakingSchedules::<T>::mutate(&who, |schedules| {
			let mut remaining_schedules = vec![];
			let mut total_cancelled = Zero::zero();

			for (scheduled_asset_id, amount, scheduled_block) in schedules.iter() {
				if *scheduled_asset_id == asset_id {
					total_cancelled += *amount;
				} else {
					remaining_schedules.push((*scheduled_asset_id, *amount, *scheduled_block));
				}
			}

			// Update the schedules
			*schedules = remaining_schedules;

			// Re-add the cancelled amount back to deposits
			Delegators::<T>::mutate(&who, |maybe_metadata| {
				let metadata = maybe_metadata.get_or_insert_with(Default::default);
				metadata
					.deposits
					.entry(asset_id)
					.and_modify(|e| *e += total_cancelled)
					.or_insert(total_cancelled);
			});
		});

		Ok(())
	}
}
