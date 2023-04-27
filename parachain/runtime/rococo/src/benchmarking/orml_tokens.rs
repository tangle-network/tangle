// Copyright 2022 Webb Technologies Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//
use crate::{AccountId, Runtime, Tokens};
use frame_benchmarking::{account, whitelisted_caller};
use frame_system::RawOrigin;
use orml_benchmarking::runtime_benchmarks;
use orml_traits::MultiCurrency;
use sp_runtime::traits::StaticLookup;
use sp_std::prelude::*;

const SEED: u32 = 0;
const CURRENCYID: webb_primitives::AssetId = 1;

runtime_benchmarks! {
	{ Runtime, orml_tokens }

	transfer {
		let from: AccountId = whitelisted_caller();
		let _ = Tokens::deposit(CURRENCYID, &from, 100);
		let to: AccountId = account("to", 0, SEED);
		let to_lookup = <Runtime as frame_system::Config>::Lookup::unlookup(to.clone());
	}: _(RawOrigin::Signed(from), to_lookup, CURRENCYID, 90)
	verify {
		assert_eq!(<Tokens as MultiCurrency<_>>::total_balance(CURRENCYID, &to), 90);
	}

	transfer_all {
		let from: AccountId = whitelisted_caller();
		let _ = Tokens::deposit(CURRENCYID, &from, 100);

		let to: AccountId = account("to", 0, SEED);
		let to_lookup = <Runtime as frame_system::Config>::Lookup::unlookup(to.clone());
	}: _(RawOrigin::Signed(from.clone()), to_lookup, CURRENCYID, false)
	verify {
		assert_eq!(<Tokens as MultiCurrency<_>>::total_balance(CURRENCYID, &from), 0);
	}

	transfer_keep_alive {
		let from: AccountId = whitelisted_caller();
		let _ = Tokens::deposit(CURRENCYID, &from, 100);

		let to: AccountId = account("to", 0, SEED);
		let to_lookup = <Runtime as frame_system::Config>::Lookup::unlookup(to.clone());
	}: _(RawOrigin::Signed(from), to_lookup, CURRENCYID, 90)
	verify {
		assert_eq!(<Tokens as MultiCurrency<_>>::total_balance(CURRENCYID, &to), 90);
	}

	force_transfer {
		let from: AccountId = account("from", 0, SEED);
		let from_lookup = <Runtime as frame_system::Config>::Lookup::unlookup(from.clone());
		let _ = Tokens::deposit(CURRENCYID, &from, 100);

		let to: AccountId = account("to", 0, SEED);
		let to_lookup = <Runtime as frame_system::Config>::Lookup::unlookup(to.clone());
	}: _(RawOrigin::Root, from_lookup, to_lookup, CURRENCYID, 100)
	verify {
		assert_eq!(<Tokens as MultiCurrency<_>>::total_balance(CURRENCYID, &to), 100);
	}

	set_balance {
		let who: AccountId = account("who", 0, SEED);
		let who_lookup = <Runtime as frame_system::Config>::Lookup::unlookup(who.clone());
	}: _(RawOrigin::Root, who_lookup, CURRENCYID, 100, 0)
	verify {
		assert_eq!(<Tokens as MultiCurrency<_>>::total_balance(CURRENCYID, &who), 100);
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::benchmarking::utils::tests::new_test_ext;
	use orml_benchmarking::impl_benchmark_test_suite;

	impl_benchmark_test_suite!(new_test_ext(),);
}
