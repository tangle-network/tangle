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

use crate::{BalanceOf, Config, Error, Event, Operators, OperatorsProfile, Pallet};
use frame_support::{
	dispatch::DispatchResult,
	pallet_prelude::*,
	traits::{Currency, ExistenceRequirement},
};
use sp_std::vec::Vec;
use tangle_primitives::services::{Field, OperatorPreferences, OperatorProfile};

impl<T: Config> Pallet<T> {
	pub fn do_register(
		operator: &T::AccountId,
		blueprint_id: u64,
		preferences: OperatorPreferences,
		registration_args: Vec<Field<T::Constraints, T::AccountId>>,
		value: BalanceOf<T>,
	) -> DispatchResult {
		let (_, blueprint) = Self::blueprints(blueprint_id)?;

		blueprint
			.type_check_registration(&registration_args)
			.map_err(Error::<T>::TypeCheck)?;

		// Transfer the registration value to the pallet
		T::Currency::transfer(
			operator,
			&Self::pallet_account(),
			value,
			ExistenceRequirement::KeepAlive,
		)?;

		let (allowed, _weight) = Self::on_register_hook(
			&blueprint,
			blueprint_id,
			&preferences,
			&registration_args,
			value,
		)
		.map_err(|e| {
			log::error!("Error in on_register_hook: {:?}", e);
			Error::<T>::OnRegisterHookFailed
		})?;

		ensure!(allowed, Error::<T>::InvalidRegistrationInput);

		Operators::<T>::insert(blueprint_id, &operator, preferences);

		OperatorsProfile::<T>::try_mutate(&operator, |profile| {
			match profile {
				Ok(p) => {
					p.blueprints
						.try_insert(blueprint_id)
						.map_err(|_| Error::<T>::MaxBlueprintsPerOperatorExceeded)?;
				},
				Err(_) => {
					let mut blueprints = BoundedBTreeSet::new();
					blueprints
						.try_insert(blueprint_id)
						.map_err(|_| Error::<T>::MaxBlueprintsPerOperatorExceeded)?;
					*profile = Ok(OperatorProfile { blueprints, ..Default::default() });
				},
			};
			Result::<_, Error<T>>::Ok(())
		})?;

		Self::deposit_event(Event::Registered {
			provider: operator.clone(),
			blueprint_id,
			preferences,
			registration_args,
		});

		Ok(())
	}
}
