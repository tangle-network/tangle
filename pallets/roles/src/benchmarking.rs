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

//! Role pallet benchmarking.

use super::*;
use crate::{
	profile::{Profile, Record, SharedRestakeProfile},
	Pallet as Roles,
};
use frame_benchmarking::v2::*;
use frame_support::BoundedVec;
use frame_system::RawOrigin;
use sp_core::sr25519;
use sp_runtime::Perbill;
use tangle_primitives::roles::RoleType;

fn assert_last_event<T: Config>(generic_event: <T as Config>::RuntimeEvent) {
	frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}

pub fn shared_profile<T: Config>() -> Profile<T> {
	let amount: T::CurrencyBalance = 3000_u64.into();
	let profile = SharedRestakeProfile {
		records: BoundedVec::try_from(vec![
			Record { role: RoleType::Tss(Default::default()), amount: None },
			Record { role: RoleType::ZkSaaS(Default::default()), amount: None },
		])
		.unwrap(),
		amount,
	};
	Profile::Shared(profile)
}

pub fn updated_profile<T: Config>() -> Profile<T> {
	let amount: T::CurrencyBalance = 5000_u64.into();
	let profile = SharedRestakeProfile {
		records: BoundedVec::try_from(vec![
			Record { role: RoleType::Tss(Default::default()), amount: None },
			Record { role: RoleType::ZkSaaS(Default::default()), amount: None },
		])
		.unwrap(),
		amount,
	};
	Profile::Shared(profile)
}

fn bond_amount<T: Config>() -> BalanceOf<T> {
	T::Currency::minimum_balance().saturating_mul(10_000u32.into())
}

fn create_validator_account<T: Config>(seed: &'static str) -> T::AccountId {
	let stash: T::AccountId = account(seed, 0, 0);
	let reward_destination = pallet_staking::RewardDestination::Staked;
	let amount = bond_amount::<T>();
	// add twice as much balance to prevent the account from being killed.
	let free_amount = amount.saturating_mul(2u32.into());
	T::Currency::make_free_balance_be(&stash, free_amount);
	pallet_staking::Pallet::<T>::bond(
		RawOrigin::Signed(stash.clone()).into(),
		amount,
		reward_destination.clone(),
	)
	.unwrap();

	let validator_prefs = pallet_staking::ValidatorPrefs {
		commission: Perbill::from_percent(50),
		..Default::default()
	};
	pallet_staking::Pallet::<T>::validate(RawOrigin::Signed(stash.clone()).into(), validator_prefs)
		.unwrap();
	stash
}

#[benchmarks(
    where
		T::RoleKeyId: From<ecdsa::Public>,
		T::AccountId : From<sr25519::Public>,
)]

mod benchmarks {
	use super::*;
	// Create profile.
	#[benchmark]
	fn create_profile() {
		let shared_profile = shared_profile::<T>();

		let caller: T::AccountId = create_validator_account::<T>("Alice");

		#[extrinsic_call]
		_(RawOrigin::Signed(caller.clone().into()), shared_profile.clone());

		let ledger = Roles::<T>::ledger(caller).unwrap();
		assert!(ledger.profile == shared_profile);
	}

	// Update profile.
	#[benchmark]
	fn update_profile() {
		let caller: T::AccountId = create_validator_account::<T>("Alice");
		let shared_profile = shared_profile::<T>();
		let ledger = RoleStakingLedger::<T>::new(caller.clone(), shared_profile.clone(), vec![]);
		Ledger::<T>::insert(caller.clone(), ledger);
		// Updating shared stake from 3000 to 5000 tokens
		let updated_profile = updated_profile::<T>();

		#[extrinsic_call]
		_(RawOrigin::Signed(caller.clone()), updated_profile.clone());

		let ledger = Roles::<T>::ledger(caller).unwrap();
		assert!(ledger.profile == updated_profile);
	}

	// Delete profile
	#[benchmark]
	fn delete_profile() {
		let caller: T::AccountId = create_validator_account::<T>("Alice");
		let shared_profile = shared_profile::<T>();
		let ledger = RoleStakingLedger::<T>::new(caller.clone(), shared_profile.clone(), vec![]);
		Ledger::<T>::insert(caller.clone(), ledger);

		#[extrinsic_call]
		_(RawOrigin::Signed(caller.clone()));

		assert_last_event::<T>(Event::ProfileDeleted { account: caller.clone() }.into());
		let ledger = Roles::<T>::ledger(caller);
		assert!(ledger.is_none())
	}

	#[benchmark]
	fn chill() {
		let caller: T::AccountId = create_validator_account::<T>("Alice");

		#[extrinsic_call]
		_(RawOrigin::Signed(caller.clone()));
	}

	#[benchmark]
	fn unbound_funds() {
		let caller: T::AccountId = create_validator_account::<T>("Alice");
		let amount: T::CurrencyBalance = 2000_u64.into();

		#[extrinsic_call]
		_(RawOrigin::Signed(caller.clone()), amount);
	}

	#[benchmark]
	fn withdraw_unbonded() {
		let caller: T::AccountId = create_validator_account::<T>("Alice");

		#[extrinsic_call]
		_(RawOrigin::Signed(caller.clone()));
	}

	// Define the module and associated types for the benchmarks
	impl_benchmark_test_suite!(Roles, crate::mock::new_test_ext(vec![]), crate::mock::Runtime,);
}
