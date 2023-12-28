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
use tangle_primitives::roles::RoleTypeMetadata;

fn assert_last_event<T: Config>(generic_event: <T as Config>::RuntimeEvent) {
	frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}

pub fn shared_profile<T: Config>() -> Profile<T> {
	let amount: T::CurrencyBalance = 3000_u64.into();
	let profile = SharedRestakeProfile {
		records: BoundedVec::try_from(vec![
			Record { metadata: RoleTypeMetadata::Tss(Default::default()), amount: None },
			Record { metadata: RoleTypeMetadata::ZkSaas(Default::default()), amount: None },
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
			Record { metadata: RoleTypeMetadata::Tss(Default::default()), amount: None },
			Record { metadata: RoleTypeMetadata::ZkSaas(Default::default()), amount: None },
		])
		.unwrap(),
		amount,
	};
	Profile::Shared(profile)
}

pub fn mock_pub_key(i: u8) -> sr25519::Public {
	sr25519::Public::from_raw([i; 32])
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
		let caller: T::AccountId = mock_pub_key(1).into();

		#[extrinsic_call]
		_(RawOrigin::Signed(caller.clone().into()), shared_profile.clone());

		let ledger = Roles::<T>::ledger(caller).unwrap();
		assert!(ledger.profile == shared_profile);
	}

	// Update profile.
	#[benchmark]
	fn update_profile() {
		let caller: T::AccountId = mock_pub_key(1).into();
		let shared_profile = shared_profile::<T>();
		let ledger = RoleStakingLedger::<T>::new(caller.clone(), shared_profile.clone());
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
		let caller: T::AccountId = mock_pub_key(1).into();
		let shared_profile = shared_profile::<T>();
		let ledger = RoleStakingLedger::<T>::new(caller.clone(), shared_profile.clone());
		Ledger::<T>::insert(caller.clone(), ledger);

		#[extrinsic_call]
		_(RawOrigin::Signed(caller.clone()));

		assert_last_event::<T>(Event::ProfileDeleted { account: caller.clone() }.into());
		let ledger = Roles::<T>::ledger(caller);
		assert!(ledger.is_none())
	}

	#[benchmark]
	fn chill() {
		let caller: T::AccountId = mock_pub_key(1).into();

		#[extrinsic_call]
		_(RawOrigin::Signed(caller.clone()));
	}

	#[benchmark]
	fn unbound_funds() {
		let caller: T::AccountId = mock_pub_key(1).into();
		let amount: T::CurrencyBalance = 2000_u64.into();

		#[extrinsic_call]
		_(RawOrigin::Signed(caller.clone()), amount);
	}

	#[benchmark]
	fn withdraw_unbonded() {
		let caller: T::AccountId = mock_pub_key(1).into();

		#[extrinsic_call]
		_(RawOrigin::Signed(caller.clone()));
	}

	// Define the module and associated types for the benchmarks
	impl_benchmark_test_suite!(
		Roles,
		crate::mock::new_test_ext(vec![1, 2, 3, 4]),
		crate::mock::Runtime,
	);
}
