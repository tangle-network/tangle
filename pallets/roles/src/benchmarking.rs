use frame_benchmarking::{benchmarks, impl_benchmark_test_suite};
use frame_support::BoundedVec;
use frame_system::RawOrigin;
use profile::{Record, SharedRestakeProfile};

use super::*;

#[allow(unused)]
use crate::Pallet;

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

benchmarks! {

	where_clause {
		where
			T::RoleKeyId: From<ecdsa::Public>,
			T::AccountId : From<u64>
	}
	// Create profile.
	create_profile {
		let shared_profile = shared_profile::<T>();
		let caller : T::AccountId = 1_u64.into();

	}: _(RawOrigin::Signed(caller.clone()), shared_profile.clone())
	verify {
		let ledger = Pallet::<T>::ledger(caller).unwrap();
		assert!(ledger.profile == shared_profile );
	}

	// Update profile.
	update_profile {
		let caller : T::AccountId = 1_u64.into();
		let shared_profile = shared_profile::<T>();
		let ledger = RoleStakingLedger::<T>::new(caller.clone(), shared_profile.clone());
		Ledger::<T>::insert(caller.clone(), ledger);
		// Updating shared stake from 3000 to 5000 tokens
		let updated_profile = updated_profile::<T>();

	}: _(RawOrigin::Signed(caller.clone()), updated_profile.clone())
	verify {
		let ledger = Pallet::<T>::ledger(caller).unwrap();
		assert!(ledger.profile == updated_profile );

	}

	// Delete profile
	delete_profile {
		let caller : T::AccountId = 1_u64.into();
		let shared_profile = shared_profile::<T>();
		let ledger = RoleStakingLedger::<T>::new(caller.clone(), shared_profile.clone());
		Ledger::<T>::insert(caller.clone(), ledger);


	}: _(RawOrigin::Signed(caller.clone()))
	verify {
		assert_last_event::<T>(Event::ProfileDeleted { account: caller.clone() }.into());
		let ledger = Pallet::<T>::ledger(caller);
		assert!(ledger.is_none())

	}

	chill {
		let caller : T::AccountId = 1_u64.into();

	}: _(RawOrigin::Signed(caller.clone()))


	unbound_funds {
		let caller : T::AccountId = 1_u64.into();
		let amount: T::CurrencyBalance = 2000_u64.into();
	}: _(RawOrigin::Signed(caller.clone()), amount)


	withdraw_unbonded {
		let caller : T::AccountId = 1_u64.into();
	}: _(RawOrigin::Signed(caller.clone()))

}

// Define the module and associated types for the benchmarks
impl_benchmark_test_suite!(
	Pallet,
	crate::mock::new_test_ext(vec![1, 2, 3, 4]),
	crate::mock::Runtime,
);
