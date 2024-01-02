use crate::{Call, Config, Pallet};
use frame_benchmarking::{account, benchmarks, impl_benchmark_test_suite};
use frame_support::traits::Currency;
use frame_system::RawOrigin;
use sp_runtime::traits::{Bounded, Zero};
use sp_std::vec;

benchmarks! {
// Benchmark submit_job function
set_fee {
	let caller: T::AccountId = account("caller", 0, 0);
}: _(RawOrigin::Signed(caller.clone()), Default::default())

}

// Define the module and associated types for the benchmarks
impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Runtime,);
